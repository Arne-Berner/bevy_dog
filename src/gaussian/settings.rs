use bevy::{
    math::{Vec2, Vec3, Vec4},
    prelude::*,
    reflect::Reflect,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Reflect, Debug, Clone, Copy)]
enum Thresholding {
    NoThreshold,
    Tanh,
    Quantization,
    SmoothQuantization,
}

#[derive(Reflect, Debug, Clone, Copy)]
enum BlendMode {
    NoBlend,
    Interpolate,
    TwoPointInterpolate,
}

#[derive(Reflect, Debug, Component, Clone, Copy, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct DoGSettings {
    thresholding: i32,
    blend_mode: i32,
    hatching_enabled: i32,
    invert: i32,
    calc_diff_before_convolution: i32,
    sigma_c: f32,
    sigma_e: f32,
    sigma_m: f32,
    sigma_a: f32,
    quantizer_step: f32,
    k: f32,
    tau: f32,
    phi: f32,
    blend_strength: f32,
    dog_strength: f32,
    brightness_offset: f32,
    saturation: f32,
    line_conv_step_sizes: Vec2,
    edge_smooth_step_sizes: Vec2,
    min_color: Vec3,
    max_color: Vec3,
    enable_layers: Vec4,
    hatch_resolution: Vec4,
    thresholds: Vec4,
}

impl Default for DoGSettings {
    fn default() -> Self {
        DoGSettings {
            thresholding: Thresholding::NoThreshold as i32,
            blend_mode: BlendMode::NoBlend as i32,
            hatching_enabled: 0,
            invert: 0,
            calc_diff_before_convolution: 0,
            sigma_c: 2.0,
            sigma_e: 2.0,
            sigma_m: 2.0,
            sigma_a: 1.0,
            quantizer_step: 2.0,
            k: 1.6,
            tau: 1.0,
            phi: 1.0,
            blend_strength: 1.0,
            dog_strength: 1.0,
            brightness_offset: 0.0,
            saturation: 1.0,
            line_conv_step_sizes: Vec2::ONE,
            edge_smooth_step_sizes: Vec2::ONE,
            min_color: Vec3::ZERO,
            max_color: Vec3::ONE,
            enable_layers: Vec4::from_array([1., 0., 0., 0.]),
            hatch_resolution: Vec4::ONE,
            thresholds: Vec4::from_array([50.0, 20.0, 30.0, 40.0]),
        }
    }
}

#[derive(Reflect, Debug, Component, Clone, Copy, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct PassesSettings {
    pub aa: i32,
    pub tfm: i32,
}
impl Default for PassesSettings {
    fn default() -> Self {
        PassesSettings { aa: 0, tfm: 0 }
    }
}
