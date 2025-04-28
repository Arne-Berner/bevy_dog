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
fn first_gaussian_blur_pass(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let x = 1/view.viewport.z;
    let y = 1/view.viewport.w;
    let texel_size = vec2(x,y);
    var col = vec2(0.0);
    var kernel_sum1 = 0.0;
    var kernel_sum2 = 0.0;

    let kernel_size = select(2.0, floor(config.sigma_e * 2.), config.sigma_e *2. > 2.);

    for (var x = -kernel_size; x <= kernel_size; x += 1.0) {
        let c = textureSample(screen_texture, texture_sampler, in.uv + vec2(x, 0.) * texel_size.xy).r;
        let gauss1 = gaussian(config.sigma_e, x);
        let gauss2 = gaussian(config.sigma_e * config.k, x);

        col.r += c * gauss1;
        kernel_sum1 += gauss1;

        col.g += c * gauss2;
        kernel_sum2 += gauss2;
    }

    return vec4(col.r / kernel_sum1, col.g / kernel_sum2, 0., 0.);
}

@fragment
fn second_gaussian_blur_pass(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let x = 1/view.viewport.z;
    let y = 1/view.viewport.w;
    let texel_size = vec2(x,y);
    var col = vec2(0.);
    var kernel_sum1 = 0.0;
    var kernel_sum2 = 0.0;

    let kernel_size = select(2.0, floor(config.sigma_e * 2.), (config.sigma_e * 2. > 2.));

    for (var y = -kernel_size; y <= kernel_size; y+= 1.0) {
        let c = textureSample(screen_texture, texture_sampler, in.uv + vec2(0., y) * texel_size.xy).rg;
        let gauss1 = gaussian(config.sigma_e, y);
        let gauss2 = gaussian(config.sigma_e * config.k, y);

        col.r += c.r * gauss1;
        kernel_sum1 += gauss1;

        col.g += c.g * gauss2;
        kernel_sum2 += gauss2;
    }

    let G = vec2(col.r / kernel_sum1, col.g / kernel_sum2);

    let D = (1. + config.tau) * (G.r * 100.0) - config.tau * (G.g * 100.0);

    var output = vec4(0.0);

    if (config.thresholding == 1) {
        output.r = select(1+tanh(config.phi * (D-config.thresholds.x)),1.0,D>= config.thresholds.x);
        output.g = select(1+tanh(config.phi * (D-config.thresholds.y)),1.0,D>=config.thresholds.y);
        output.b = select(1+tanh(config.phi * (D-config.thresholds.z)),1.0,D>=config.thresholds.z);
        output.a = select(1+tanh(config.phi * (D-config.thresholds.w)),1.0,D>=config.thresholds.w);
    } else if (config.thresholding == 2) {
        let a = 1.0 / config.quantizer_step;
        let b = config.thresholds.x / 100.0;
        let x = D / 100.0;

        // TODO this was not vec4 in the original code
        output = vec4(select(a* floor((pow(x,config.phi) - (a*b/2.)) / (a*b)+0.5), 1.0, x>= b));
    } else if (config.thresholding == 2) {
        let a = 1.0 / config.quantizer_step;
        let b = config.thresholds.x / 100.0;
        let x = D / 100.0;

        output = vec4(select(a* floor((pow(x,config.phi) - (a*b/2.)) / (a*b)+0.5), 1.0, x>= b));
    } else if (config.thresholding == 3) {
        let x = D / 100.;
        let qn = floor(x * config.quantizer_step + 0.5) / config.quantizer_step;
        let qs = smoothstep(-2.0, 2.0, config.phi * (x - qn) * 10.0) - 0.5;
        output = vec4(qn + qs / config.quantizer_step);
    } else {
        output = vec4(D / 100.0);
    }

    if config.invert == 1 {
        output = 1.0 - output;
    }

    return saturate(output);
}

