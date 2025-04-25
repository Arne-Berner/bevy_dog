#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

// he uses DogTex to represent the result of all passes before the (last) blending pass
// I only need to implement one blendmode for now, but it might be easy to implement them all at once?
fn gaussian(sigma: f32, pos: f32)->f32{
    return (1. / sqrt(2. * PI * sigma * sigma)) * exp(-(pos * pos) / (2. * sigma * sigma));
}

// 2nd pass
fn horizontal_tfn_blur(in:vec2f, main_tex: texture_2d, s: sampler, texel_size:vec4f, sigma_c:f32) -> vec4f {
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
    
    return col / kernelSum;
}


// 3rd pass
// I think this is the vertical pass? but it is different at the end.
fn vertical_tfn_blur(in:vec2f, main_tex: texture_2d, s: sampler, texel_size:vec4f, sigma_c:f32) -> vec4f {
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
    
    
    return select(vec4(0.,1.,0.,1.), vec4(normalize(d), sqrt(lamda1), 1.), length(d) != 0);
}

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

