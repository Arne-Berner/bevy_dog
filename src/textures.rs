use crate::settings::DoGSettings;
use bevy::{
    image::BevyDefault,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
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
}

pub fn prepare_dog_textures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
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

        commands.entity(entity).insert(DoGTextures {
            lab_texture,
            eigen_texture,
            horizontal_texture,
            vertical_texture,
            first_dog_texture,
            second_dog_texture,
            aa_texture,
        });
    }
}
