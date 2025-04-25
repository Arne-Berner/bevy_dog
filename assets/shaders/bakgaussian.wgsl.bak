#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import "shaders/common.wgsl"::luminance
#import "shaders/common.wgsl"::rgb2lab
#import "shaders/common.wgsl"::calculate_eigenvector
#import "shaders/common.wgsl"::luminance
#import "shaders/common.wgsl"::luminance
#import "shaders/common.wgsl"::luminance


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
    // return length(d) ? vec4(normalize(d), sqrt(lambda1), 1.0) : vec4(0.0, 1.0, 0.0, 1.0);
}

// 4th pass
fn FDoGBlurPass(in: vec2f, main: texture_2d, tfm: vec4f, s: sampler, texel_size: vec4f, tau: f32) -> vec4f{
    let tangent = tfm.xy;
    var normal = vec2(tangent.y, -tangent.x);
    let normal_abs = abs(normal);
    // changed from a ternary to this for readability
    let ds = 1.0 / (max(normal_abs.x, normal_abs.y));
    normal *= texel_size.xy;
    var col = textureSample(main, s, i.uv).xx;
    var kernel_sum = 1.0;
    let kernel_size = select(floor(sigma_e * 2), 1, sigmae_e * 2 > 1);

    for (var x = ds; x <= kernel_size; x += 1){
        let gauss1 = gaussian(_SigmaE, x);
        let gauss2 = gaussian(_SigmaE * _K, x);

        let c1 = textureSample(main, s, i.uv - x * n).r;
        let c2 = textureSample(main, s, i.uv + x * n).r;

        col.r += (c1 + c2) * gauss1;
        kernel_sum.x += 2.0 * gauss1;

        col.g += (c1 + c2) * gauss2;
        kernel_sum.y +=  2.0 * gauss2;
    }
    col /= kernelSum;

    return vec4(col, (1 + tau) * (col.r * 100.0) - tau * (col.g * 100.0), 1.0);
}

// 5th pass
// TODO I need different render passes, because it uses the texture written to in the previous frame... otherwise I need an intermediate texture that I can reuse. Can I just write something to a texture in wgpu? I don't think so.
fn fdog_blur_with_threshold(in: vec2f, sigma_m: f32, main: texture_2d, s: sampler, tfm: texture_2d, point_clamp_sampler: sampler, calc_diff_before_convolution: i32, texel_size: vec4f, line_conv_step_size: vec4, k:f32, thresholding:i32, threshold: f32, phi:f32, invert:i32, quantizer_step: f32) -> vec4f {
    float kernel_size = sigma_m * 2;

    var w = vec2(1.0);
    // what is the texture at this point? If Calcdiff is used, then there is a third entry
    // in c, otherwise there is not
    // not used anywhere???
    var c = textureSample(main, s, in.uv).rgb;
    // TODO is it 0 or <= 0?
    var G = select(c.rg, vec2(c.b, 0.0), calc_diff_before_convolution != 0);
    // TODO does this work, since it's vec2 times vec4?
    let tfm_frag = textureSample(tfm, point_clamp_sampler, i.uv).xy * texel_size;
    var st0 = in.uv;
    var v0 = tfm_frag;

    for (var d = 1; d < kernel_size; d++) {
        st0 += v0 * line_conv_step_size.x;
        let c = textureSample(main, s, st0).rgb;
        let gauss1 = gaussian(sigma_m, d);

        if (calc_diff_before_convolution != 0) {
            G.r += gauss1 * c.b;
            w.x += gauss1;
        } else {
            let gauss2 = gaussian(sigma_m * k, d);

            G.r += gauss1 * c.r;
            w.x += gauss1;

            G.g += gauss2 * c.g;
            w.y += gauss2;
        }

        v0 = textureSample(tfm, point_clamp_sampler, st0).xy * texel_size.xy;
    }

    var st1 = in.uv;
    var v1 = tfm_frag;

    for (var d = 1; d < kernel_size; d++) {
        st1 -= v1 * line_conv_step_size.y;
        let c = tex2D(main, s, st1).rgb;
        let gauss1 = gaussian(sigma_m, d);


        if (calc_diff_before_convolution != 0) {
            G.r += gauss1 * c.b;
            G.r += gauss1 * c.b;
            w.x += gauss1;
        } else {
            let gauss2 = gaussian(sigma_m * k, d);

            G.r += gauss1 * c.r;
            w.x += gauss1;

            G.g += gauss2 * c.g;
            w.y += gauss2;
        }

        v1 = textureSample(tfm, point_clamp_sampler, st1).xy * texel_size.xy;
    }

    G /= w;

    var D = 0.0;
    if (calc_diff_before_convolution != 0) {
        D = G.x;
    } else {
        // D here is described in the paper as S.
        // so maybe DoG?
        D = (1 + _Tau) * (G.r * 100.0) - _Tau * (G.g * 100.0);
    }

    var output = vec4(0.0);

    if (thresholding == 1) {
        output.r = select(1+tanh(phi * (D-threshold)),1,D>=threshold);
        output.r = select(1+tanh(phi * (D-threshold2)),1,D>=threshold2);
        output.r = select(1+tanh(phi * (D-threshold3)),1,D>=threshold3);
        output.r = select(1+tanh(phi * (D-threshold4)),1,D>=threshold4);
    } else if (thresholding == 2) {
        let a = 1.0 / quantizer_step;
        let b = threshold / 100.0;
        let x = D / 100.0;

        // TODO this was not vec4 in the original code
        output = vec4(select(a* floor((pow(x,phi) - (a*b/2.)) / (a*b)+0.5), 1, x>= b));
    } else if (thresholding == 3) {
        let x = D / 100.;
        let qn = floor(x * float(quantizer_step) + 0.5) / float(quantizer_step);
        let qs = smoothstep(-2.0, 2.0, phi * (x - qn) * 10.0) - 0.5;
        
        output = qn + qs / float(quantizer_step);
    } else {
        output = D / 100.0;
    }

    if (invert) {
        output = 1 - output;
    }

    return saturate(output);
}

// 6th pass
fn non_fdog_blur(in: vec2f, main: texture_2d, s: sampler, sigma_e: f32, texel_size: vec4f, k: f32)-> vec4f {
    let col = vec2(0);
    let kernel_sum1 = 0.0;
    let kernel_sum2 = 0.0;

    let kernel_size = select(2., floor(sigma_e * 2.), sigma_e *2 > 2);

    for (var x = -kernel_size; x <= kernel_size; x++) {
        let c = textureSample(main, s, in.uv + vec2(x, 0) * texel_size.xy).r;
        let gauss1 = gaussian(sigma_e, x);
        let gauss2 = gaussian(sigma_e * k, x);

        col.r += c * gauss1;
        kernel_sum1 += gauss1;

        col.g += c * gauss2;
        kernel_sum2 += gauss2;
    }

    return vec4(col.r / kernel_sum1, col.g / kernel_sum2, 0, 0);
}

// 7th pass
fn non_fdog_blur2(in: vec2f, main: texture_2d, s: sampler, sigma_e: f32, texel_size: vec4f, k: f32, tau: f32, quantizer_step: f32) -> vec4f{
    var col = 0;
    var kernel_sum1 = 0.0;
    var kernel_sum2 = 0.0;

    let kernel_size = select(2, floor(sigma_e * 2.), (sigma_e * 2. > 2.));

    for (let y = -kernel_size; y <= kernel_size; y++) {
        let c = textureSample(main, s, in.uv + vec2(0, y) * texel_size.xy).rg;
        let gauss1 = gaussian(sigma_e, y);
        let gauss2 = gaussian(sigma_e * k, y);

        col.r += c.r * gauss1;
        kernel_sum1 += gauss1;

        col.g += c.g * gauss2;
        kernel_sum2 += gauss2;
    }

    float2 G = float2(col.r / kernel_sum1, col.g / kernel_sum2);

    float D = (1. + tau) * (G.r * 100.0) - tau * (G.g * 100.0);

    var output = vec4(0.0);

    if (thresholding == 1) {
        output.r = select(1+tanh(phi * (D-threshold)),1,D>=threshold);
        output.r = select(1+tanh(phi * (D-threshold2)),1,D>=threshold2);
        output.r = select(1+tanh(phi * (D-threshold3)),1,D>=threshold3);
        output.r = select(1+tanh(phi * (D-threshold4)),1,D>=threshold4);
    } else if (thresholding == 2) {
        let a = 1.0 / quantizer_step;
        let b = threshold / 100.0;
        let x = D / 100.0;

        // TODO this was not vec4 in the original code
        output = vec4(select(a* floor((pow(x,phi) - (a*b/2.)) / (a*b)+0.5), 1, x>= b));
    } else if (thresholding == 3) {
        let x = D / 100.;
        let qn = floor(x * float(quantizer_step) + 0.5) / float(quantizer_step);
        let qs = smoothstep(-2.0, 2.0, phi * (x - qn) * 10.0) - 0.5;
        
        output = qn + qs / float(quantizer_step);
    } else {
        output = D / 100.0;
    }

    if (invert) {
        output = 1 - output;
    }

    return saturate(output);
}

// 8th pass
fn anti_aliasing(in: vec2, sigma_a: f32, main: texture2d, s:sampler, tfm: texture_2d, point_clamp_sampler: sampler, texel_size: i32, edge_smooth_step_size: vec4f)->vec4f{
    let kernel_size = sigma_a * 2;
    let g = textureSample(main, s, in.uv);
    var w = 1.0;
    let v = textureSample(tfm, point_clamp_sampler, in.uv).xy * texel_size;
    var st0 = vec2(in.uv);
    var v0 = v;
    for (var d = 1; d < kernelSize; d++) {
        st0 += v0 * edge_smooth_step_size.x;
        let c = textureSample(main,s, st0);
        let gauss1 = gaussian(sigma_a, d);

        G += gauss1 * c;
        w += gauss1;

        v0 = textureSample(tfm, point_clamp_sampler, st0).xy * texel_size.xy;
    }

    var st1 = in.uv;
    var v1 = v;

    for (var d = 1; d < kernelSize; d++) {
        st1 -= v1 * edge_smooth_step_size.y;
        let c = textureSample(main, s, st1);
        let gauss1 = gaussian(sigma_a, d);

        G += gauss1 * c;
        w += gauss1;

        v1 = textureSample(tfm, point_clamp_sampler, st1).xy * texel_size.xy;
    }

    return G / w;
}

// 9th pass
fn blend(in:vec2f, dog: texture_2d, main:vec4f, s: sampler, dog_strength: f32, blend_mode:i32, min_color: vec3f, max_color: vec3f, hatching_enabled: i32, hatch: texture_2d, second_layer: i32, third_layer: i32, hatch_res: vec4f, blend_strength: f32) -> vec4f {
    let D = textureSample(dog, s, in.uv) * dog_strength;
    let col = main.rgb;
    var output = 0.;

    if blend_mode == 0 {
        output.rgb = lerp(min_color, max_color, D.r);
    } else if blend_mode == 1 {
        output.rgb = lerp(min_color, col, D.r);
    } else if blend_mode == 2 {
        if D.r < 0.5 {
            output.rgb = lerp(min_color, col, D.r * 2.);
        } else {
            output.rgb = lerp(col, max_color, (D.r - 0.5) * 2.);
        }
    }

    if hatching_enabled {
        let hatchUV = in.uv * 2 -1; // TODO why not starting at 0?
        let s1 = textureSample(hex, s, hatchUV * hatch_res.r * 0.5 + 0.5).rgb;
        output.rgb = lerp(vec3(s1.r), max_color, D.r);

        // every enabled layer will just add the respective hatching texture to the output
        if  second_layer != 0 {
            output.rgb *= lerp(vec3(s1.g), max_color, D.g);
        }
        if  third_layer != 0 {
            output.rgb *= lerp(vec3(s1.b), max_color, D.b);
        }
    }

    return saturate(vec4(lerp(col, output, blend_strength), 1.0));
}

struct Config {
// threshold 1
// threshold 2
// threshold 3
// threshold 4
// texel_size
// blendmode
// sigma_c

    crosshatch_threshold: f32,
    edge_color: vec4f,
    debug: u32,
    enabled: u32,
};

// TFM texture
// DoG texture
@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var cross_hatch_texture: texture_2d<f32>;
@group(0) @binding(3) var<uniform> view: View;
@group(0) @binding(4) var<uniform> config: Config;

// TODO: I should be able to sample the main texture once here and forward this into the functions
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    // FIRST PIPELINE
    // 0th pass: rgb2lab
    let main = textureSample(screen_texture, texture_sampler, uv);
    let lab = rgb2lab(screen_fragment.rgb);


    // TAKES RGBTOLAB
    // SECOND PIPELINE
    // #ifdef USEFLOW OR SMOOTHEDGEDS
    // first pass: get eigenvector
    calculate_eigenvector(in, config.texel_size, screen_texture, texture_sampler);

    // second pass: tensor float map blur 1
    // calculate tensor float map blur horizontal
    let horizontal = horizontal_tfn_blur(in, screen_texture,  texture_sampler, texel_size, config.sigma_c);

    // third pass: tensor float map blur 2
    let vertical = vertical_tfn_blur(in, screen_texture, texture_sampler, texel_size, config.sigma_c);

    // SET VERTICAL TO TEXTURE TFM
    // END
    

    // TAKES RGBTOLAB
    // THIRD PIPELINE
    // IFEDF USEFLOW
    // fourth pass: FDog Blur
    let fdog = FDoGBlurPass(in, screen_texture, tfm, texture_sampler, config.texel_size, config.tau);

    // TODO pass only config
    // fifth pass: FDog Blur 2
    let fdog2 = fdog_blur_with_threshold(in, config.sigma_m, screen_texture,
        texture_sampler, tfm_texture, point_clamp_sampler, 
        config.calc_diff_before_convolution, config.texel_size,
        config.integral_conv_step_size, config.k, config.thresholding,
        config.threshold, config.phi, config.invert, config.quantizer_step);
    // ELSE 
    // sixth pass: Gaussian Blur
    let non_fdog = non_fdog_blur(in, screen_texture, texture_sampler, config.sigma_e, config.texel_size, config.k);

    // seventh pass: Gaussian Blur
    let non_fdog2 = non_fdog_blur2(in, screen_texture, texture_sampler, config.sigma_e, config.texel_size, config.k, config.tau, config.quantizer_step);

    // END

    // IF SMOOTH EDGES
    // eigth pass: AA
    let aa = 
        anti_aliasing(in, config.sigma_a, screen_texture, texture_sampler, tfm,
        point_clamp_sampler, config.texel_size, config.integral_convolution_size);
    // set aa to DoG texture
    // ELSE
    // set non_fdog2 or fdog2 to DoG texture
    // END

    // FOURTH PIPELINE
    // ninth pass: Blend
    let out = blend(in, dog, main, texture_sampler, config.dog_strength,
        config.blend_mode, config.min_color, config.max_color, config.hatching_enabled2,
        hatch_texture, config.second_layer, config.third_layer, config.hatch_res,
        config.blend_strength);




    // old stuff
    let ratio = view.viewport.z/view.viewport.w;
    let uv = vec2(in.uv.x * ratio*.5, in.uv.y*.5);
    let color = textureSample(cross_hatch_texture, texture_sampler, uv);

    return color;

}

