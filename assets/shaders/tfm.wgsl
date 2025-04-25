#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

const PI: f32 = 3.14159265359;
fn gaussian(sigma: f32, pos: f32)->f32{
    return (1. / sqrt(2. * PI * sigma * sigma)) * exp(-(pos * pos) / (2. * sigma * sigma));
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> view: View;
@group(0) @binding(3) var<uniform> config: DoGSettings;

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
    var out = vec4(1.0);
#ifdef EIGENVECTOR
    let x = 1/view.viewport.z;
    let y = 1/view.viewport.w;
    return vec4(1.);

/*
    let Sx = vec3((
        1.0f * textureSample(screen_texture, in.uv + vec2(-x, -y)).rgb +
        2.0f * textureSample(screen_texture, in.uv + vec2(-x,  0.0)).rgb +
        1.0f * textureSample(screen_texture, in.uv + vec2(-x,  y)).rgb +
        -1.0f * textureSample(screen_texture, in.uv + vec2(x, -y)).rgb +
        -2.0f * textureSample(screen_texture, in.uv + vec2(x,  0.0)).rgb +
        -1.0f * textureSample(screen_texture, in.uv + vec2(x,  y)).rgb
    ) / 4.0f);

    let Sy = vec3((
        1.0f * textureSample(screen_texture, in.uv + vec2(-x, -y)).rgb +
        2.0f * textureSample(screen_texture, in.uv + vec2( 0.0, -y)).rgb +
        1.0f * textureSample(screen_texture, in.uv + vec2( x, -y)).rgb +
        -1.0f * textureSample(screen_texture, in.uv + vec2(-x, y)).rgb +
        -2.0f * textureSample(screen_texture, in.uv + vec2( 0.0, y)).rgb +
        -1.0f * textureSample(screen_texture, in.uv + vec2( x, y)).rgb
    ) / 4.0f);

    
    out = vec4(dot(Sx, Sx), dot(Sy, Sy), dot(Sx, Sy),1.0);
    */
#endif
    return out;
}


#ifdef HORIZONTAL
// horizontal blur pass
@fragment
fn horizontal_blur_pass(in: FullscreenVertexOutput) -> @location(0) vec4f {
    var out = vec4(1.0);
    let kernelRadius = max(1.0, floor(sigma_c * 2.45));
    var col = vec4<f32>(0.0);
    var kernelSum = 0.0;
    
    for (var x = -kernelRadius; x <= kernelRadius; x = x+1.) {
        let samplePos = uv + vec2(x, 0.0) * texel_size.xy;
        let c = textureSample(mainTex, s, samplePos);
        let gauss = gaussian(sigma_c, x);
        
        col += c * gauss;
        kernelSum += gauss;
    }
    
    out = col / kernelSum;
    return out;
}
#endif

#ifdef VERTICAL
// vertical blur pass
@fragment
fn vertical_blur_pass(in: FullscreenVertexOutput) -> @location(0) vec4f {
    var out = vec4(1.0);
    let kernelRadius = max(1.0, floor(sigma_c * 2.45));
    var col = vec4<f32>(0.0);
    var kernelSum = 0.0;
    
    for (var y = -kernelRadius; y <= kernelRadius; y = y+1.) {
        let samplePos = uv + vec2(0.0, y) * texel_size.xy;
        let c = textureSample(mainTex, s, samplePos);
        let gauss = gaussian(sigma_c, y);
        
        col += c * gauss;
        kernelSum += gauss;
    }

    let g = vec3(col.rgb / kernelSum);

    let lambda1 = 0.5 * (g.y + g.x + sqrt(g.y * g.y - 2.0 * g.x * g.y + g.x * g.x + 4.0 * g.z * g.z));
    let d = vec2(g.x - lambda1, g.z);
    
    
    out = select(vec4(0.,1.,0.,1.), vec4(normalize(d), sqrt(lamda1), 1.), length(d) != 0);
    return out;
}
#endif
