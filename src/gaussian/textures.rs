use super::settings::DoGSettings;
use crate::gaussian::plugin::CROSSHATCH_TEXTURE_HANDLE;
use bevy::{
    image::BevyDefault,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, GpuImage, TextureCache},
        view::ExtractedView,
    },
};

#[derive(Component)]
pub struct DoGTextures {
    pub lab_texture: CachedTexture,
    pub eigen_texture: CachedTexture,
    pub horizontal_texture: CachedTexture,
    pub vertical_texture: CachedTexture,
    pub first_dog_texture: CachedTexture,
    pub second_dog_texture: CachedTexture,
    pub aa_texture: CachedTexture,
    pub crosshatch_texture: Texture,
}

pub fn prepare_dog_textures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    // render_queue: Res<RenderQueue>,
    images: Res<RenderAssets<GpuImage>>,
    mut texture_cache: ResMut<TextureCache>,
    view_targets: Query<(Entity, &ExtractedCamera), (With<ExtractedView>, With<DoGSettings>)>,
) {
    for (entity, camera) in &view_targets {
        let Some(texture_size) = camera.physical_target_size else {
            continue;
        };

        let texture_size = Extent3d {
            width: texture_size.x,
            height: texture_size.y,
            depth_or_array_layers: 1,
        };

        // Create the two-channel RG texture for phase 1 (edge detection).
        let lab_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("lab color texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        let eigen_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("Eigenvector Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );
        let horizontal_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("Horizontal Blur Pass Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );
        let vertical_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("Vertical Blur Pass Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        let first_dog_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("first dog texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        let second_dog_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("second dog texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        let aa_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("AA Pass Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        // I don't need to create this myself, but I should be able to use load asset
        // Fetch the two lookup textures. These are bundled in this library.
        let Some(crosshatch_image) = images.get(&CROSSHATCH_TEXTURE_HANDLE) else {
            return;
        };

        /*
        let diffuse_bytes = include_bytes!("../../assets/textures/crosshatch.jpg");
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

        // I should probably create 2 more textures, because DoG needs one and TFM
        let crosshatch_texture = render_device.create_texture_with_data(
            &render_queue,
            &TextureDescriptor {
                label: Some("crosshatch-texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::default(),
            bytemuck::cast_slice(&diffuse_rgba),
        );
        */
        let crosshatch_texture = crosshatch_image.texture.clone();

        commands.entity(entity).insert(DoGTextures {
            lab_texture,
            eigen_texture,
            horizontal_texture,
            vertical_texture,
            first_dog_texture,
            second_dog_texture,
            aa_texture,
            crosshatch_texture,
        });
    }
}
