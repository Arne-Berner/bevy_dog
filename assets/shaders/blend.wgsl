#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

const PI: f32 = 3.14159265359;

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

    // this uses max color, but I could also mix with min color
    if config.enable_hatch == 1 {
        output = vec3(1.0);
        let hatchUV = in.uv * 2.0 -1.0; 
        if config.enable_layers.x == 1.0 {
            let radians = config.hatch_rotations.r * PI / 180.0;
            let rot: mat2x2<f32> = mat2x2<f32>(
                cos(radians), -sin(radians), 
                sin(radians), cos(radians), 
            );

            let s1 = textureSample(
                hatch_texture, hatch_sampler, (hatchUV * rot) * config.hatch_resolutions.x * 0.5 + 0.5).rgb;
            output = vec3(mix(s1, config.max_color, D.r));
        }

        // every enabled layer will just add the respective hatching texture to the output
        if  config.enable_layers.y != 0.0 {
            let radians = config.hatch_rotations.g * PI / 180.0;
            let rot: mat2x2<f32> = mat2x2<f32>(
                cos(radians), -sin(radians), 
                sin(radians), cos(radians), 
            );
            let s2 = textureSample(hatch_texture, hatch_sampler, rot * hatchUV * config.hatch_resolutions.y * 0.5 + 0.5).rgb;
            output = vec3(mix(s2, config.max_color, D.g)) * output.rgb;
        }
        if  config.enable_layers.z != 0.0 {
            let radians = config.hatch_rotations.b * PI / 180.0;
            let rot: mat2x2<f32> = mat2x2<f32>(
                cos(radians), -sin(radians), 
                sin(radians), cos(radians), 
            );
            let s3 = textureSample(hatch_texture, hatch_sampler, rot * hatchUV * config.hatch_resolutions.z * 0.5 + 0.5).rgb;
            output = vec3(mix(s3, config.max_color, D.b)) * output.rgb;
        }
        if  config.enable_layers.w != 0.0 {
            let radians = config.hatch_rotations.a * PI / 180.0;
            let rot: mat2x2<f32> = mat2x2<f32>(
                cos(radians), -sin(radians), 
                sin(radians), cos(radians), 
            );
            let s4 = textureSample(hatch_texture, hatch_sampler, rot * hatchUV * config.hatch_resolutions.w * 0.5 + 0.5).rgb;
            output = vec3(mix(s4, config.max_color, D.a)) * output.rgb;
        }
    }

    return saturate(vec4(mix(col, output, config.blend_strength), 1.0));
    // return saturate(vec4(mix(col, output, 0.0), 1.0));
}

