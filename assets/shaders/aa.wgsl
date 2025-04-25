#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(3) var<uniform> view: View;
@group(0) @binding(4) var<uniform> config: DoGSettings;

struct DoGSettings {
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
    line_conv_step_sizes: vec2i,
    edge_smooth_step_sizes: vec2i,
    min_color: vec3f,
    max_color: vec3f,
    enable_layers: vec4i,
    hatch_resolution: vec4f,
    thresholds: vec4f,
}


@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    // let main = textureSample(screen_texture, texture_sampler, uv);
    // let lab = rgb2lab(screen_fragment.rgb);
    // return vec4(lab, 1.);
    return vec4(1.);
}

