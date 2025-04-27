#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

const PI: f32 = 3.14159265359;

fn gaussian(sigma: f32, pos: f32)->f32{
    return (1. / sqrt(2. * PI * sigma * sigma)) * exp(-(pos * pos) / (2. * sigma * sigma));
}

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

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> view: View;
@group(0) @binding(3) var<uniform> config: DoGSettings;

@group(1) @binding(0) var tfm_texture: texture_2d<f32>;
@group(1) @binding(1) var point_clamp_sampler: sampler;


@fragment
fn fdog_blur_pass(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let x = 1/view.viewport.z;
    let y = 1/view.viewport.w;
    let xy = vec2(x,y);
    let tfm = textureSample(tfm_texture, point_clamp_sampler, in.uv); 
    // direction of the vector flow
    let tensor = tfm.xy;
    var n = vec2(tensor.y, -tensor.x);
    let n_abs = abs(n);
    // changed from a ternary to this for readability
    let ds = 1.0 / (max(n_abs.x, n_abs.y));
    n *= xy;
    var col = textureSample(screen_texture, texture_sampler, in.uv).xx; // x = lightness in lab
    var kernel_sum = vec2(1.0);
    let kernel_size = select(1.0, floor(config.sigma_e * 2.0), config.sigma_e * 2.0 > 1.0);

    for (var x = ds; x <= kernel_size; x += 1.0){
        let gauss1 = gaussian(config.sigma_e, x);
        let gauss2 = gaussian(config.sigma_e * config.k, x);

        // why only x?
        let c1 = textureSample(screen_texture, texture_sampler, in.uv - x * n).r;
        let c2 = textureSample(screen_texture, texture_sampler, in.uv + x * n).r;

        col.r += (c1 + c2) * gauss1; // adds surrounding lightness times gaussian to current lightness
        kernel_sum.x += 2.0 * gauss1;

        col.g += (c1 + c2) * gauss2;
        kernel_sum.y +=  2.0 * gauss2;
    }
    // 
    col /= kernel_sum;
    

    return vec4(col, (1 + config.tau) * (col.r * 100.0) - config.tau * (col.g * 100.0), 1.0);
}

@fragment
fn fdog_blur_and_difference(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let x = 1/view.viewport.z;
    let y = 1/view.viewport.w;
    let xy = vec2(x,y);
    let kernel_size = config.sigma_m * 2.0;

    var w = vec2(1.0);
    var c = textureSample(screen_texture, texture_sampler, in.uv).rgb;
    var G = select(c.rg, vec2(c.b, 0.0), config.calc_diff_before_convolution != 0);
    let tfm_frag = textureSample(tfm_texture, point_clamp_sampler, in.uv).xy * xy;
    var st0 = in.uv;
    var v0 = tfm_frag;

    for (var d = 1.0; d < kernel_size; d += 1.0) {
        st0 += v0 * vec2(f32(config.line_conv_step_sizes.x));
        c = textureSample(screen_texture, texture_sampler, st0).rgb;
        let gauss1 = gaussian(config.sigma_m, d);

        if (config.calc_diff_before_convolution != 0) {
            G.r += gauss1 * c.b;
            w.x += gauss1;
        } else {
            let gauss2 = gaussian(config.sigma_m * config.k, d);

            G.r += gauss1 * c.r;
            w.x += gauss1;

            G.g += gauss2 * c.g;
            w.y += gauss2;
        }

        v0 = textureSample(tfm_texture, point_clamp_sampler, st0).xy * xy;
    }

    var st1 = in.uv;
    var v1 = tfm_frag;

    for (var d = 1.0; d < kernel_size; d+=1.0) {
        st1 -= v1 * f32(config.line_conv_step_sizes.y);
        let c = textureSample(screen_texture, texture_sampler, st1).rgb;
        let gauss1 = gaussian(config.sigma_m, d);


        if (config.calc_diff_before_convolution != 0) {
            G.r += gauss1 * c.b;
            G.r += gauss1 * c.b;
            w.x += gauss1;
        } else {
            let gauss2 = gaussian(config.sigma_m * config.k, d);

            G.r += gauss1 * c.r;
            w.x += gauss1;

            G.g += gauss2 * c.g;
            w.y += gauss2;
        }

        v1 = textureSample(tfm_texture, point_clamp_sampler, st1).xy * xy;
    }

    G /= w;

    var D = 0.0;
    if (config.calc_diff_before_convolution != 0) {
        D = G.x;
    } else {
        D = (1 + config.tau) * (G.r * 100.0) - config.tau * (G.g * 100.0);
    }

    var output = vec4(0.0);

    if (config.thresholding == 1) {
        output.r = select(1+tanh(config.phi * (D-config.thresholds.x)),1.0,D>= config.thresholds.x);
        output.r = select(1+tanh(config.phi * (D-config.thresholds.y)),1.0,D>=config.thresholds.y);
        output.r = select(1+tanh(config.phi * (D-config.thresholds.z)),1.0,D>=config.thresholds.z);
        output.r = select(1+tanh(config.phi * (D-config.thresholds.w)),1.0,D>=config.thresholds.w);
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

        // TODO this was not vec4 in the original code
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

