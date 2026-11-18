#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var scene_image: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> tape: vec4<f32>;

fn grain_at(pixel: vec2<f32>, time_slice: f32) -> f32 {
    let p = fract(pixel.xyx * vec3<f32>(0.1031, 0.1030, 0.0973) + vec3<f32>(time_slice * 0.0317));
    let q = p + vec3<f32>(dot(p, p.yzx + vec3<f32>(33.33)));
    return fract((q.x + q.y) * q.z) - 0.5;
}

@fragment
fn fragment(input: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let strength = tape.x;
    let time = tape.y;
    let uv = input.uv;
    let pixel = 1.0 / vec2<f32>(textureDimensions(scene_image));
    let line = floor(uv.y * 260.0);
    // A slow head-switching trace near the foot of the frame; no cuts or flashes.
    let head = smoothstep(0.963, 1.0, uv.y);
    let drift = sin(time * 2.5 + line * 0.105) * 0.0011 + head * sin(line * 1.05 + time) * 0.0034;
    let tracked = uv + vec2<f32>(strength * drift, 0.0);
    let chroma = vec2<f32>(strength * max(pixel.x, 0.001) * 2.0, 0.0);
    let body = textureSampleLevel(scene_image, scene_sampler, tracked, 0.0);
    let smear = textureSampleLevel(scene_image, scene_sampler, tracked - chroma, 0.0).rgb;
    let bleed = textureSampleLevel(scene_image, scene_sampler, tracked + chroma, 0.0).rgb;
    let separated = vec3<f32>(smear.r, body.g, bleed.b);
    let signal = sqrt(max(mix(body.rgb, separated, 0.82), vec3<f32>(0.0)));
    let luma = dot(signal, vec3<f32>(0.299, 0.587, 0.114));
    let wash = mix(signal, vec3<f32>(luma), strength * 0.22);
    let scan = 0.010 + 0.021 * (0.5 + 0.5 * cos(uv.y * 1507.96));
    let grain = grain_at(floor(uv * vec2<f32>(580.0, 320.0)), floor(time * 10.0));
    let centre = uv - vec2<f32>(0.5);
    let corners = clamp(dot(centre, centre) * 2.0, 0.0, 1.0);
    let flicker_free = wash * (1.0 - strength * (scan + corners * 0.17))
        + strength * (vec3<f32>(0.012, 0.014, 0.019) + vec3<f32>((0.045 + head * 0.064) * grain));
    let limited = max(flicker_free, vec3<f32>(0.0));
    return vec4<f32>(limited * limited, body.a);
}
