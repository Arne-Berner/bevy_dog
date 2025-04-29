use bevy::{
    math::{Vec2, Vec3, Vec4},
    prelude::*,
    reflect::Reflect,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum Thresholding {
    NoThreshold,
    Tanh,
    Quantization,
    SmoothQuantization,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    NoBlend,
    Interpolate,
    TwoPointInterpolate,
}

#[derive(Reflect, Debug, Component, Clone, Copy, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct DoGSettings {
    pub thresholding: i32,
    pub blend_mode: i32,
    pub invert: i32,
    pub calc_diff_before_convolution: i32,
    pub sigma_c: f32,
    pub sigma_e: f32,
    pub sigma_m: f32,
    pub sigma_a: f32,
    pub quantizer_step: f32,
    pub k: f32,
    pub tau: f32,
    pub phi: f32,
    pub blend_strength: f32,
    pub dog_strength: f32,
    pub line_conv_step_sizes: Vec2,
    pub edge_smooth_step_sizes: Vec2,
    pub min_color: Vec3,
    pub max_color: Vec3,
    pub enable_hatch: i32,
    pub enable_layers: Vec4,
    pub hatch_resolutions: Vec4,
    pub hatch_rotations: Vec4,
    pub thresholds: Vec4,
}

impl Default for DoGSettings {
    fn default() -> Self {
        DoGSettings {
            thresholding: Thresholding::NoThreshold as i32,
            blend_mode: BlendMode::NoBlend as i32,
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
            line_conv_step_sizes: Vec2::ONE,
            edge_smooth_step_sizes: Vec2::ONE,
            min_color: Vec3::ZERO,
            max_color: Vec3::ONE,
            enable_hatch: 0,
            enable_layers: Vec4::from_array([1., 0., 0., 0.]),
            hatch_resolutions: Vec4::ONE,
            hatch_rotations: Vec4::from_array([15., 60., 105., 170.]),
            thresholds: Vec4::from_array([90.0, 20.0, 30.0, 40.0]),
        }
    }
}

impl DoGSettings {
    pub const DEFAULT: Self = Self {
        thresholding: Thresholding::NoThreshold as i32,
        blend_mode: BlendMode::NoBlend as i32,
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
        line_conv_step_sizes: Vec2::ONE,
        edge_smooth_step_sizes: Vec2::ONE,
        min_color: Vec3::ZERO,
        max_color: Vec3::ONE,
        enable_hatch: 0,
        enable_layers: Vec4::from_array([1., 0., 0., 0.]),
        hatch_resolutions: Vec4::ONE,
        hatch_rotations: Vec4::from_array([15., 60., 105., 170.]),
        thresholds: Vec4::from_array([90.0, 20.0, 30.0, 40.0]),
    };
    pub const OUTLINE_DITHER: Self = Self {
        k: 0.5,
        tau: 32.,
        phi: 0.8,
        thresholding: Thresholding::Tanh as i32,
        thresholds: Vec4::from_array([12., 6., 3., 0.5]),
        min_color: Vec3::from_array([0.5, 0.4, 0.4]),
        max_color: Vec3::from_array([0.8, 1.0, 0.9]),
        ..Self::DEFAULT
    };
    pub const CROSSHATCH: Self = Self {
        tau: 4.0,
        blend_strength: 0.9,
        max_color: Vec3::from_array([0.8, 0.85, 0.81]),
        phi: 2.0,
        thresholding: Thresholding::Tanh as i32,
        enable_layers: Vec4::from_array([1., 1., 1., 1.]),
        thresholds: Vec4::from_array([0.2, 1.3, 0.7, 0.5]),
        enable_hatch: 1,
        hatch_resolutions: Vec4::splat(6.2),
        ..Self::DEFAULT
    };
    pub const OUTLINE: Self = Self {
        tau: 15.0,
        ..Self::DEFAULT
    };
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
