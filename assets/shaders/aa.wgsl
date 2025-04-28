#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

const PI: f32 = 3.14159265359;

fn gaussian(sigma: f32, pos: f32)->f32{
    return (1. / sqrt(2. * PI * sigma * sigma)) * exp(-(pos * pos) / (2. * sigma * sigma));
}

struct DoGSettings {
    thresholding: i32,
    blend_mode: i32,
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
    line_conv_step_sizes: vec2i,
    edge_smooth_step_sizes: vec2i,
    min_color: vec3f,
    max_color: vec3f,
    enable_hatch: i32,
    enable_layers: vec4f,
    hatch_resolutions: vec4f,
    hatch_rotations: vec4f,
    thresholds: vec4f,
}




@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> view: View;
@group(0) @binding(3) var<uniform> config: DoGSettings;

@group(1) @binding(0) var tfm_texture: texture_2d<f32>;
@group(1) @binding(1) var point_clamp_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let x = 1/view.viewport.z;
    let y = 1/view.viewport.w;
    let texel_size = vec2(x,y);
    let kernel_size = config.sigma_a * 2.0;
    var G = textureSample(screen_texture, texture_sampler, in.uv);
    var w = 1.0;
    let v = textureSample(tfm_texture, point_clamp_sampler, in.uv).xy * texel_size;
    var st0 = vec2(in.uv);
    var v0 = v;
    for (var d = 1.0; d < kernel_size; d += 1.0) {
        st0 += v0 * f32(config.edge_smooth_step_sizes.x);
        let c = textureSample(screen_texture,texture_sampler, st0);
        let gauss1 = gaussian(config.sigma_a, d);

        G += gauss1 * c;
        w += gauss1;

        v0 = textureSample(tfm_texture, point_clamp_sampler, st0).xy * texel_size;
    }

    var st1 = in.uv;
    var v1 = v;

    for (var d = 1.0; d < kernel_size; d += 1.0) {
        st1 -= v1 * f32(config.edge_smooth_step_sizes.y);
        let c = textureSample(screen_texture, texture_sampler, st1);
        let gauss1 = gaussian(config.sigma_a, d);

        G += gauss1 * c;
        w += gauss1;

        v1 = textureSample(tfm_texture, point_clamp_sampler, st1).xy * texel_size;
    }

    return G / w;
}
