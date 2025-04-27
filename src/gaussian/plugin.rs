use super::{pipeline::DoGPipelines, settings::DoGSettings};
use crate::gaussian::{
    node::{prepare_dog_bind_groups, DoGNode},
    pipeline::{prepare_gaussian_pipelines, DoGSpecializedRenderPipelines},
    settings::PassesSettings,
    textures::prepare_dog_textures,
};
use bevy::{
    asset::{load_internal_asset, weak_handle, RenderAssetUsages},
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{RenderGraphApp, RenderLabel, ViewNodeRunner},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        Render, RenderApp, RenderSet,
    },
};

pub const CROSSHATCH_TEXTURE_HANDLE: Handle<Image> =
    weak_handle!("3bc8be12-aa9d-481f-bce3-56ad52cdfea4");
pub const RGB2LAB_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("ce20ba6c-9bd1-4a62-bfe8-ba51750dbdc3");
pub const TFM_SHADER_HANDLE: Handle<Shader> = weak_handle!("34197afe-e54f-4d38-9fc6-7c467524cc59");
pub const FDOG_SHADER_HANDLE: Handle<Shader> = weak_handle!("8588af6b-9061-4514-bb16-95deb4e06818");
pub const AA_SHADER_HANDLE: Handle<Shader> = weak_handle!("9d4d296b-cb2d-4e25-8125-c6854184f959");
pub const DOG_SHADER_HANDLE: Handle<Shader> = weak_handle!("bedbea43-8967-4cdd-95b6-d3a4d630c436");
pub const BLEND_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("b86e54d8-858a-41e5-84d5-62a34f455a77");

/// It is generally encouraged to set up post processing effects as a plugin
pub struct DoGPlugin;

impl Plugin for DoGPlugin {
    fn build(&self, app: &mut App) {
        // so I seem to register it too?
        app.register_type::<DoGSettings>().add_plugins((
            ExtractComponentPlugin::<DoGSettings>::default(),
            UniformComponentPlugin::<DoGSettings>::default(),
        ));
        app.register_type::<PassesSettings>().add_plugins((
            ExtractComponentPlugin::<PassesSettings>::default(),
            UniformComponentPlugin::<PassesSettings>::default(),
        ));

        let diffuse_bytes = include_bytes!("../../assets/textures/crosshatch.png");

        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();
        // this is for the texture
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let crosshatch_image = Image::new_fill(
            size,
            TextureDimension::D2,
            bytemuck::cast_slice(&diffuse_rgba),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );

        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .insert(CROSSHATCH_TEXTURE_HANDLE.id(), crosshatch_image);

        load_internal_asset!(
            app,
            RGB2LAB_SHADER_HANDLE,
            "../../assets/shaders/rgb2lab.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TFM_SHADER_HANDLE,
            "../../assets/shaders/tfm.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            FDOG_SHADER_HANDLE,
            "../../assets/shaders/fdog.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            DOG_SHADER_HANDLE,
            "../../assets/shaders/dog.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            AA_SHADER_HANDLE,
            "../../assets/shaders/aa.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            BLEND_SHADER_HANDLE,
            "../../assets/shaders/blend.wgsl",
            Shader::from_wgsl
        );

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DoGSpecializedRenderPipelines>()
            .add_systems(
                Render,
                (
                    prepare_gaussian_pipelines.in_set(RenderSet::Prepare),
                    prepare_dog_textures.in_set(RenderSet::PrepareResources),
                    prepare_dog_bind_groups.in_set(RenderSet::PrepareBindGroups),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<DoGNode>>(
                // Specify the label of the graph, in this case we want the graph for 3d
                Core3d, // It also needs the label of the node
                DoGLabel,
            )
            .add_render_graph_edges(
                Core3d,
                // Specify the node ordering.
                // This will automatically create all required node dogs to enforce the given ordering.
                (
                    Node3d::Tonemapping,
                    DoGLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            println!("could not get renderapp");
            return;
        };

        render_app.init_resource::<DoGPipelines>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct DoGLabel;
