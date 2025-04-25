#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

fn luminance(color: vec3f) -> f32 {
    return dot(color, vec3(0.299f, 0.587f, 0.114f));
}

// Color conversions from https://gist.github.com/mattatz/44f081cac87e2f7c8980
fn rgb2xyz(c: vec3f) -> vec3f{
    var tmp = vec3(0.);

    tmp.x = select(c.r/12.92, pow((c.r+0.055)/1.055, 2.4), c.r>0.04045);
    tmp.x = select(c.g/12.92, pow((c.g+0.055)/1.055, 2.4), c.g>0.04045);
    tmp.x = select(c.b/12.92, pow((c.b+0.055)/1.055, 2.4), c.b>0.04045);
    
    let mat: mat3x3<f32> = mat3x3<f32>(
        0.4124, 0.3576, 0.1805,
        0.2126, 0.7152, 0.0722,
        0.0193, 0.1192, 0.9505
    );

    return 100.0 * (tmp * mat);
}

fn xyz2lab(c:vec3f) -> vec3f {
    let n = c / vec3(95.047, 100, 108.883);
    var v = vec3(0.);

    v.x = select((7.787 * n.x) + (16.0 / 116.0), pow(n.x, 1.0 / 3.0), n.x > 0.008856);
    v.x = select((7.787 * n.y) + (16.0 / 116.0), pow(n.y, 1.0 / 3.0), n.y > 0.008856);
    v.x = select((7.787 * n.z) + (16.0 / 116.0), pow(n.z, 1.0 / 3.0), n.z > 0.008856);

    return vec3((116.0 * v.y) - 16.0, 500.0 * (v.x - v.y), 200.0 * (v.y - v.z));
}

fn rgb2lab(c: vec3f) -> vec3f {
    let lab = xyz2lab(rgb2xyz(c));

    return vec3(lab.x / 100.0f, 0.5 + 0.5 * (lab.y / 127.0), 0.5 + 0.5 * (lab.z / 127.0));
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
    // let main = textureSample(screen_texture, texture_sampler, uv);
    // let lab = rgb2lab(screen_fragment.rgb);
    // return vec4(lab, 1.);
    return vec4(1.);
}

