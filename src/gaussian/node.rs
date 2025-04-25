use super::{
    pipeline::{DoGPipelines, GaussianPipelineIDs},
    plugin::CROSSHATCH_TEXTURE_HANDLE,
    settings::DoGSettings,
    textures::DoGTextures,
};
use bevy::{
    ecs::{query::QueryItem, system::Commands, world::World},
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_asset::RenderAssets,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::{
            AddressMode, BindGroup, BindGroupEntries, FilterMode, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, SamplerDescriptor,
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        view::{ExtractedView, ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

#[derive(Default)]
pub struct DoGNode;

impl ViewNode for DoGNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static DoGSettings,
        &'static DynamicUniformIndex<DoGSettings>,
        &'static GaussianPipelineIDs,
        &'static DoGTextures,
        &'static DoGBindGroups,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            view_target,
            view_uniform_offset,
            _post_process_settings,
            settings_index,
            view_pipelines,
            textures,
            bind_groups,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let dog_pipeline = world.resource::<DoGPipelines>();

        let rgb2lab_pipeline = pipeline_cache
            .get_render_pipeline(view_pipelines.rgb2lab_pipeline_id)
            .unwrap();

        let _eigenvector_tfm_pipeline = pipeline_cache
            .get_render_pipeline_state(view_pipelines.tfm_pipeline_ids.eigenvector_pipeline_id)
            .unwrap();
        /*
        let err = match eigenvector_tfm_pipeline_state {
            bevy::render::render_resource::CachedPipelineState::Queued => None,
            bevy::render::render_resource::CachedPipelineState::Creating(_) => None,
            bevy::render::render_resource::CachedPipelineState::Ok(_) => None,
            bevy::render::render_resource::CachedPipelineState::Err(pipeline_cache_error) => {
                Some(pipeline_cache_error)
            }
        };
        let err = match err.unwrap() {
            bevy::render::render_resource::PipelineCacheError::ShaderNotLoaded(_) => None,
            bevy::render::render_resource::PipelineCacheError::ProcessShaderError(
                composer_error,
            ) => Some(composer_error),
            bevy::render::render_resource::PipelineCacheError::ShaderImportNotYetAvailable => None,
            bevy::render::render_resource::PipelineCacheError::CreateShaderModule(_) => None,
        };
        if let Some(state) = err {
            println!("{:?}", state.inner);
        }
        */
        // let vertical_tfm_pipeline = vertical_tfm_pipeline_state.unwrap();
        /*
        // Fetch the render pipelines.
        let (
            Some(rgb2lab_pipeline),
            Some(vertical_tfm_pipeline),
            Some(horizontal_tfm_pipeline),
            Some(first_fdog_pipeline),
            Some(second_fdog_pipeline),
            Some(first_dog_pipeline),
            Some(second_dog_pipeline),
            Some(aa_pipeline),
            Some(blend_pipeline),
        ) = (
            pipeline_cache
                .get_render_pipeline(view_pipelines.tfm_pipeline_ids.vertical_pipeline_id),
            pipeline_cache
                .get_render_pipeline(view_pipelines.tfm_pipeline_ids.horizontal_pipeline_id),
            pipeline_cache.get_render_pipeline(view_pipelines.fdog_pipeline_ids.first),
            pipeline_cache.get_render_pipeline(view_pipelines.fdog_pipeline_ids.second),
            pipeline_cache.get_render_pipeline(view_pipelines.dog_pipeline_ids.first),
            pipeline_cache.get_render_pipeline(view_pipelines.dog_pipeline_ids.second),
            pipeline_cache.get_render_pipeline(view_pipelines.aa_pipeline_id),
            pipeline_cache.get_render_pipeline(view_pipelines.blend_pipeline_id),
        )
        else {
            println!("cache not workng");
            return Ok(());
        };
            */

        // Fetch the framebuffer textures.
        let postprocess = view_target.post_process_write();
        let (source, destination) = (postprocess.source, postprocess.destination);
        // I should get the prepared pipelines
        // textures
        // (views)
        // bind groups
        //
        // Then I create functions for each pass (except tfm and fdog+aa will be brought together I think)
        //
        //
        // In each of those passes I will create the postprocess bind group with the sampler from
        // the pipeline
        // Then I will set the textures accordingly.
        //
        // then I will do the draw calls

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        // Get the settings uniform binding
        let view_uniforms = world.resource::<ViewUniforms>();
        let Some(view_uniforms) = view_uniforms.uniforms.binding() else {
            println!("view uniforms");
            return Ok(());
        };
        let settings_uniforms = world.resource::<ComponentUniforms<DoGSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            println!("settings binding");
            return Ok(());
        };

        let postprocess_bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &dog_pipeline.rgba2lab.postprocess_bind_group_layout,
            &BindGroupEntries::sequential((
                source,
                &dog_pipeline.rgba2lab.sampler,
                view_uniforms,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("rgb2lab_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &destination,
                // view: &textures.lab_texture.default_view,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(rgb2lab_pipeline);
        render_pass.set_bind_group(
            0,
            &postprocess_bind_group,
            &[view_uniform_offset.offset, settings_index.index()],
        );
        // render_pass.set_bind_group(1, &view_smaa_bind_groups.edge_detection_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

// fn rgb2lab_render_pass() {}

/// A render world component that stores the bind groups necessary to perform
/// SMAA.
///
/// This is stored on each view.
#[derive(Component)]
pub struct DoGBindGroups {
    pub tfm_bind_group: BindGroup,
    pub blend_bind_group: BindGroup,
}

pub fn prepare_dog_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    dog_pipelines: Res<DoGPipelines>,
    images: Res<RenderAssets<GpuImage>>,
    view_targets: Query<(Entity, &DoGTextures), (With<ExtractedView>, With<DoGSettings>)>,
) {
    // Fetch the two lookup textures. These are bundled in this library.
    let Some(cross_hatch) = images.get(&CROSSHATCH_TEXTURE_HANDLE) else {
        return;
    };

    // for every camera with dog
    for (entity, dog_textures) in &view_targets {
        let cross_hatch_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("common sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });

        let point_clamp_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("common sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..default()
        });

        // these don't change, while the post_process_bind_group does for each pass
        commands.entity(entity).insert(DoGBindGroups {
            tfm_bind_group: render_device.create_bind_group(
                Some("TFM bind group"),
                &dog_pipelines.fdog.tfm_bind_group_layout,
                &BindGroupEntries::sequential((
                    &dog_textures.tfm_texture.default_view,
                    &point_clamp_sampler,
                )),
            ),
            blend_bind_group: render_device.create_bind_group(
                Some("blend texture bind group"),
                &dog_pipelines.blend.blend_bind_group_layout,
                &BindGroupEntries::sequential((
                    &dog_textures.dog_texture.default_view,
                    &cross_hatch.texture_view,
                    &cross_hatch_sampler,
                )),
            ),
        });
    }
}
