#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

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
    enable_layers: vec4f,
    hatch_resolution: vec4f,
    thresholds: vec4f,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> view: View;
@group(0) @binding(3) var<uniform> config: DoGSettings;

@group(1) @binding(0) var dog_texture: texture_2d<f32>;
@group(1) @binding(1) var hatch_texture: texture_2d<f32>;
@group(1) @binding(2) var hatch_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let main = textureSample(screen_texture, texture_sampler, in.uv);
    let D = textureSample(dog_texture, texture_sampler, in.uv) * config.dog_strength;
    let col = main.rgb;
    var output = vec3(0.);

    if config.blend_mode == 0 {
        output = vec3(mix(config.min_color, config.max_color, D.r));
    } else if config.blend_mode == 1 {
        output = vec3(mix(config.min_color, col, D.r));
    } else if config.blend_mode == 2 {
        if D.r < 0.5 {
            output = vec3(mix(config.min_color, col, D.r * 2.));
        } else {
            output = vec3(mix(col, config.max_color, (D.r - 0.5) * 2.));
        }
    }

    if config.enable_layers.x == 1.0 {
        let hatchUV = in.uv * 2.0 -1.0; 
        let s1 = textureSample(hatch_texture, hatch_sampler, hatchUV * config.hatch_resolution.r * 0.5 + 0.5).rgb;
        output = vec3(mix(vec3(s1.r), config.max_color, D.r));
        output = vec3(s1.r);


        // every enabled layer will just add the respective hatching texture to the output
        if  config.enable_layers.y != 0.0 {
            output = vec3(mix(vec3(s1.g), config.max_color, D.g)) * output.rgb;
        }
        if  config.enable_layers.z != 0.0 {
            output = vec3(mix(vec3(s1.b), config.max_color, D.b)) * output.rgb;
        }
    }

    return saturate(vec4(mix(col, output, config.blend_strength), 1.0));
}

