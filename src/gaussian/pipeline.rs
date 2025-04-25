use super::plugin::{
    AA_SHADER_HANDLE, BLEND_SHADER_HANDLE, DOG_SHADER_HANDLE, FDOG_SHADER_HANDLE,
    RGB2LAB_SHADER_HANDLE, TFM_SHADER_HANDLE,
};
use crate::gaussian::settings::DoGSettings;
use bevy::render::render_resource::{
    AddressMode, FilterMode, Sampler, SpecializedRenderPipeline, SpecializedRenderPipelines,
};
use bevy::{
    asset::DirectAssetAccessExt,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::world::{FromWorld, World},
    image::BevyDefault,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderStages,
            TextureFormat, TextureSampleType,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/cross_hatch.wgsl";

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
pub struct DoGPipelines {
    // Pass 1: RGBA to LAB
    pub rgba2lab: RGB2LABPipeline,
    // Passes 2&3 (Optional): Get Tensor Flow Map for texture
    pub tfm: TFMPipeline,
    // Passes 4&5: Flow Based-Difference of Gaussians
    pub fdog: FDoGPipeline,
    // Passes 4&5: Difference of Gaussians
    pub dog: DoGPipeline,
    // Passes 6: Anti Alliasing
    pub aa: AntiAlliasingPipeline,
    // Pass 7: Blending with potential hatch texture
    pub blend: BlendPipeline,
}

pub struct RGB2LABPipeline {
    /// The bind group layout common to all passes.
    pub postprocess_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

pub struct TFMPipeline {
    /// The bind group layout common to all passes.
    pub postprocess_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
}

#[derive(Component)]
pub struct TFMPipelineIDs {
    pub eigenvector_pipeline_id: CachedRenderPipelineId,
    pub horizontal_pipeline_id: CachedRenderPipelineId,
    pub vertical_pipeline_id: CachedRenderPipelineId,
}

pub struct FDoGPipeline {
    /// The bind group layout common to all passes.
    pub postprocess_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    /// The bind group layout for data specific to this pass.
    pub tfm_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub struct FDoGPipelineIDs {
    /// blurring the image by using dog
    pub first: CachedRenderPipelineId,
    /// blurring the image by using dog again and calculating the difference
    pub second: CachedRenderPipelineId,
}

pub struct DoGPipeline {
    /// The bind group layout common to all passes.
    pub postprocess_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
}

#[derive(Component)]
pub struct DoGPipelineIDs {
    /// blurring the image by using dog
    pub first: CachedRenderPipelineId,
    /// blurring the image by using dog again and calculating the difference
    pub second: CachedRenderPipelineId,
}

pub struct AntiAlliasingPipeline {
    /// The bind group layout common to all passes.
    pub postprocess_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    /// The bind group layout for data specific to this pass.
    pub tfm_bind_group_layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
}

pub struct BlendPipeline {
    /// The bind group layout common to all passes.
    pub postprocess_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    /// The bind group layout for data specific to this pass.
    pub blend_bind_group_layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
}

/// A render world component that holds the pipeline IDs for all passes needed in this effect.
///
/// There are nine separate DoG passes, each with a different shader and 4 bind
/// group layouts, so we need nine IDs. The maximum of different bind groups possible by wgpu are
/// 4.
#[derive(Component)]
pub struct GaussianPipelineIDs {
    /// The pipeline ID to turn rgb into lab
    pub rgb2lab_pipeline_id: CachedRenderPipelineId,
    /// The pipeline IDs for the horizontal and vertical TFM pass
    pub tfm_pipeline_ids: TFMPipelineIDs,
    /// The pipeline IDs for the dog passes
    pub dog_pipeline_ids: DoGPipelineIDs,
    pub fdog_pipeline_ids: FDoGPipelineIDs,
    /// The pipeline ID for optional AA
    pub aa_pipeline_id: CachedRenderPipelineId,
    /// The pipeline ID for blending the optional texture
    pub blend_pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for DoGPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // bind group 1
        // first making one for rgb2lab -> doesn't need any texture to read from
        let postprocess_bind_group_layout = render_device.create_bind_group_layout(
            "DoG postprocess_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // source
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    // view
                    uniform_buffer::<ViewUniform>(true),
                    // settings specific to DoG
                    uniform_buffer::<DoGSettings>(true),
                ),
            ),
        );

        // bind group 2
        let tfm_bind_group_layout = render_device.create_bind_group_layout(
            "tfm_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // tfm
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // point clamp sampler
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        // bind group 3
        // first making one for blend -> reads from dog and hatch
        let blend_bind_group_layout = render_device.create_bind_group_layout(
            "blend_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // dog
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // hatch
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // sampler for hatch
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("rgba2lab_pipeline".into()),
            layout: vec![postprocess_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: RGB2LAB_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("common sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });
        let rgba2lab = RGB2LABPipeline {
            postprocess_bind_group_layout: postprocess_bind_group_layout.clone(),
            sampler: sampler.clone(),
            pipeline_id,
        };

        let tfm = TFMPipeline {
            postprocess_bind_group_layout: postprocess_bind_group_layout.clone(),
            sampler: sampler.clone(),
        };

        let fdog = FDoGPipeline {
            postprocess_bind_group_layout: postprocess_bind_group_layout.clone(),
            sampler: sampler.clone(),
            tfm_bind_group_layout: tfm_bind_group_layout.clone(),
        };

        let dog = DoGPipeline {
            postprocess_bind_group_layout: postprocess_bind_group_layout.clone(),
            sampler: sampler.clone(),
        };

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("anti_aliasing_pipeline".into()),
            layout: vec![postprocess_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: AA_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let aa = AntiAlliasingPipeline {
            postprocess_bind_group_layout: postprocess_bind_group_layout.clone(),
            sampler: sampler.clone(),
            tfm_bind_group_layout,
            pipeline_id,
        };

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("blending_pipeline".into()),
            layout: vec![postprocess_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: BLEND_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let blend = BlendPipeline {
            postprocess_bind_group_layout,
            sampler,
            blend_bind_group_layout,
            pipeline_id,
        };

        Self {
            rgba2lab,
            tfm,
            fdog,
            dog,
            aa,
            blend,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum TFMPipelineKeys {
    Eigenvector,
    Vertical,
    Horizontal,
}

impl SpecializedRenderPipeline for TFMPipeline {
    type Key = TFMPipelineKeys;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // let shader_defs = vec![preset.shader_def()];

        // I don't think I have a couple of different ones
        // Those are the defs shown in the shader to use special parses
        let shader_defs = match key {
            TFMPipelineKeys::Eigenvector => vec!["EIGENVECTOR".into()],
            TFMPipelineKeys::Vertical => vec!["VERTICAL".into()],
            TFMPipelineKeys::Horizontal => vec!["HORIZONTAL".into()],
        };

        RenderPipelineDescriptor {
            label: Some("RGBA2LAB".into()),
            layout: vec![self.postprocess_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: TFM_SHADER_HANDLE,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct FDoGPipelineKeys {
    first: bool,
}

impl SpecializedRenderPipeline for FDoGPipeline {
    type Key = FDoGPipelineKeys;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // let shader_defs = vec![preset.shader_def()];

        let entry_point = if key.first {
            "first".into()
        } else {
            "second".into()
        };

        // I don't think I have a couple of different ones
        // Those are the defs shown in the shader to use special parses
        let shader_defs = if key.first {
            vec!["FIRST".into()]
        } else {
            vec!["SECOND".into()]
        };

        RenderPipelineDescriptor {
            label: Some("FDOG".into()),
            layout: vec![self.postprocess_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: FDOG_SHADER_HANDLE,
                shader_defs,
                entry_point,
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct DoGPipelineKeys {
    first: bool,
}

impl SpecializedRenderPipeline for DoGPipeline {
    type Key = DoGPipelineKeys;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let entry_point = if key.first {
            "first_gaussian_blur_pass".into()
        } else {
            "second_gaussian_blur_pass".into()
        };

        // I don't think I have a couple of different ones
        // Those are the defs shown in the shader to use special parses
        let shader_defs = if key.first {
            vec!["FIRST".into()]
        } else {
            vec!["SECOND".into()]
        };

        RenderPipelineDescriptor {
            label: Some("DOG".into()),
            layout: vec![self.postprocess_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: DOG_SHADER_HANDLE,
                shader_defs,
                entry_point,
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct DoGSpecializedRenderPipelines {
    tfm: SpecializedRenderPipelines<TFMPipeline>,
    fdog: SpecializedRenderPipelines<FDoGPipeline>,
    dog: SpecializedRenderPipelines<DoGPipeline>,
}

pub fn prepare_gaussian_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    dog_pipelines: Res<DoGPipelines>,
    mut specialized_render_pipelines: ResMut<DoGSpecializedRenderPipelines>,
    views: Query<(Entity, &DoGSettings)>,
) {
    for (entity, _dog) in &views {
        let rgb2lab_pipeline_id = dog_pipelines.rgba2lab.pipeline_id;

        let eigenvector_pipeline_id = specialized_render_pipelines.tfm.specialize(
            &pipeline_cache,
            &dog_pipelines.tfm,
            TFMPipelineKeys::Eigenvector,
        );

        let vertical_pipeline_id = specialized_render_pipelines.tfm.specialize(
            &pipeline_cache,
            &dog_pipelines.tfm,
            TFMPipelineKeys::Vertical,
        );

        let horizontal_pipeline_id = specialized_render_pipelines.tfm.specialize(
            &pipeline_cache,
            &dog_pipelines.tfm,
            TFMPipelineKeys::Horizontal,
        );

        let first_fdog = specialized_render_pipelines.fdog.specialize(
            &pipeline_cache,
            &dog_pipelines.fdog,
            FDoGPipelineKeys { first: true },
        );

        let second_fdog = specialized_render_pipelines.fdog.specialize(
            &pipeline_cache,
            &dog_pipelines.fdog,
            FDoGPipelineKeys { first: false },
        );

        let first_dog = specialized_render_pipelines.dog.specialize(
            &pipeline_cache,
            &dog_pipelines.dog,
            DoGPipelineKeys { first: true },
        );

        let second_dog = specialized_render_pipelines.dog.specialize(
            &pipeline_cache,
            &dog_pipelines.dog,
            DoGPipelineKeys { first: false },
        );

        let aa_pipeline_id = dog_pipelines.aa.pipeline_id;
        let blend_pipeline_id = dog_pipelines.blend.pipeline_id;

        commands.entity(entity).insert(GaussianPipelineIDs {
            rgb2lab_pipeline_id,
            tfm_pipeline_ids: TFMPipelineIDs {
                eigenvector_pipeline_id,
                vertical_pipeline_id,
                horizontal_pipeline_id,
            },
            dog_pipeline_ids: DoGPipelineIDs {
                first: first_dog,
                second: second_dog,
            },
            fdog_pipeline_ids: FDoGPipelineIDs {
                first: first_fdog,
                second: second_fdog,
            },
            aa_pipeline_id,
            blend_pipeline_id,
        });
    }
}
