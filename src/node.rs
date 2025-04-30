use crate::{
    pipeline::{DoGPipelines, GaussianPipelineIDs},
    plugin::CROSSHATCH_TEXTURE_HANDLE,
    settings::{DoGSettings, PassesSettings},
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
        &'static PassesSettings,
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
            passes_settings,
            settings_index,
            view_pipelines,
            textures,
            bind_groups,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let dog_pipeline = world.resource::<DoGPipelines>();

        let err = pipeline_cache
            .get_render_pipeline_state(view_pipelines.rgb2lab_pipeline_id)
            .unwrap();

        /*
        if let Some(err) = match err {
            bevy::render::render_resource::CachedPipelineState::Queued => None,
            bevy::render::render_resource::CachedPipelineState::Creating(_) => None,
            bevy::render::render_resource::CachedPipelineState::Ok(_) => None,
            bevy::render::render_resource::CachedPipelineState::Err(pipeline_cache_error) => {
                Some(pipeline_cache_error)
            }
        } {
            let err = match err {
                bevy::render::render_resource::PipelineCacheError::ShaderNotLoaded(_) => None,
                bevy::render::render_resource::PipelineCacheError::ProcessShaderError(
                    composer_error,
                ) => Some(composer_error),
                bevy::render::render_resource::PipelineCacheError::ShaderImportNotYetAvailable => {
                    None
                }
                bevy::render::render_resource::PipelineCacheError::CreateShaderModule(_) => None,
            };
            if let Some(state) = err {
                println!("{:?}", state.inner);
            }
        }
        */

        let (
            Some(rgb2lab_pipeline),
            Some(eigenvector_pipeline),
            Some(vertical_pipeline),
            Some(horizontal_pipeline),
            Some(first_fdog_pipeline),
            Some(second_fdog_pipeline),
            Some(first_dog_pipeline),
            Some(second_dog_pipeline),
            Some(aa_pipeline),
            Some(blend_pipeline),
        ) = (
            pipeline_cache.get_render_pipeline(view_pipelines.rgb2lab_pipeline_id),
            pipeline_cache
                .get_render_pipeline(view_pipelines.tfm_pipeline_ids.eigenvector_pipeline_id),
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
            println!("Pipeline cache has not prepared the pipelines yet");
            return Ok(());
        };
        let postprocess = view_target.post_process_write();
        let (source, destination) = (postprocess.source, postprocess.destination);
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

        {
            let postprocess_bind_group = render_context.render_device().create_bind_group(
                "post_process_bind_group",
                &dog_pipeline.rgba2lab.postprocess_bind_group_layout,
                &BindGroupEntries::sequential((
                    source,
                    &dog_pipeline.rgba2lab.sampler,
                    view_uniforms.clone(),
                    settings_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("rgb2lab_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    // view: &destination,
                    view: &textures.lab_texture.default_view,
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
        }

        if (passes_settings.aa == 1) && (passes_settings.tfm == 1) {
            // PASS 1 Eigenvector
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "eigenvector_process_bind_group",
                    &dog_pipeline.tfm.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.lab_texture.default_view,
                        &dog_pipeline.tfm.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("eigenvector_pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.eigen_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(eigenvector_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                // render_pass.set_bind_group(1, &view_smaa_bind_groups.edge_detection_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // PASS 2 Vertical
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "horizontal_process_bind_group",
                    &dog_pipeline.tfm.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.eigen_texture.default_view,
                        &dog_pipeline.tfm.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("eigenvector_pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.horizontal_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(horizontal_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                // render_pass.set_bind_group(1, &view_smaa_bind_groups.edge_detection_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // PASS 3 Vertical + bringing it together
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "horizontal_process_bind_group",
                    &dog_pipeline.tfm.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.horizontal_texture.default_view,
                        &dog_pipeline.tfm.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("vertical_pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.vertical_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(vertical_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                // render_pass.set_bind_group(1, &view_smaa_bind_groups.edge_detection_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        if passes_settings.tfm == 1 {
            // PASS 4 first FDOG blur
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "horizontal_process_bind_group",
                    &dog_pipeline.fdog.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.lab_texture.default_view,
                        &dog_pipeline.fdog.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("first fdog pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.first_dog_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(first_fdog_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                render_pass.set_bind_group(1, &bind_groups.tfm_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // PASS 5 second FDOG blur
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "second_fdog_bind_group",
                    &dog_pipeline.fdog.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.first_dog_texture.default_view,
                        &dog_pipeline.fdog.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("second fdog pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.second_dog_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(second_fdog_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                render_pass.set_bind_group(1, &bind_groups.tfm_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        } else {
            // PASS 4 first DOG blur
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "horizontal_process_bind_group",
                    &dog_pipeline.dog.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.lab_texture.default_view,
                        &dog_pipeline.dog.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("first dog pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.first_dog_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(first_dog_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                render_pass.draw(0..3, 0..1);
            }

            // PASS 5 second DOG blur
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "second_dog_bind_group",
                    &dog_pipeline.dog.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.first_dog_texture.default_view,
                        &dog_pipeline.dog.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("second dog pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.second_dog_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(second_dog_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                render_pass.draw(0..3, 0..1);
            }
        }

        if passes_settings.aa == 1 {
            // PASS 6 AA
            {
                let postprocess_bind_group = render_context.render_device().create_bind_group(
                    "aa_bind_group",
                    &dog_pipeline.aa.postprocess_bind_group_layout,
                    &BindGroupEntries::sequential((
                        &textures.second_dog_texture.default_view,
                        &dog_pipeline.aa.sampler,
                        view_uniforms.clone(),
                        settings_binding.clone(),
                    )),
                );

                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("aa pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &textures.aa_texture.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render_pass.set_render_pipeline(aa_pipeline);
                render_pass.set_bind_group(
                    0,
                    &postprocess_bind_group,
                    &[view_uniform_offset.offset, settings_index.index()],
                );
                render_pass.set_bind_group(1, &bind_groups.tfm_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        // FINAL PASS Blend
        {
            let postprocess_bind_group = render_context.render_device().create_bind_group(
                "blend_process_bind_group",
                &dog_pipeline.blend.postprocess_bind_group_layout,
                &BindGroupEntries::sequential((
                    source,
                    &dog_pipeline.blend.sampler,
                    view_uniforms,
                    settings_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("blend_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: destination,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(blend_pipeline);
            render_pass.set_bind_group(
                0,
                &postprocess_bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            if passes_settings.aa == 1 {
                render_pass.set_bind_group(1, &bind_groups.aa_blend_bind_group, &[]);
            } else {
                render_pass.set_bind_group(1, &bind_groups.blend_bind_group, &[]);
            }
            render_pass.draw(0..3, 0..1);
        }

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
    pub aa_blend_bind_group: BindGroup,
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
                    &dog_textures.vertical_texture.default_view,
                    &point_clamp_sampler,
                )),
            ),

            aa_blend_bind_group: render_device.create_bind_group(
                Some("blend texture bind group"),
                &dog_pipelines.blend.blend_bind_group_layout,
                &BindGroupEntries::sequential((
                    &dog_textures.aa_texture.default_view,
                    &cross_hatch.texture_view,
                    &cross_hatch_sampler,
                )),
            ),
            blend_bind_group: render_device.create_bind_group(
                Some("blend texture bind group"),
                &dog_pipelines.blend.blend_bind_group_layout,
                &BindGroupEntries::sequential((
                    &dog_textures.second_dog_texture.default_view,
                    &cross_hatch.texture_view,
                    &cross_hatch_sampler,
                )),
            ),
        });
    }
}
/*
// Fetch the render pipelines.
 */
