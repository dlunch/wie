@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var scale_filter: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var output: VertexOutput;
    output.position = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    output.uv = uv;
    return output;
}

@fragment
fn fragment_srgb(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(source, scale_filter, input.uv).rgb, 1.0);
}

@fragment
fn fragment_unorm(input: VertexOutput) -> @location(0) vec4<f32> {
    let linear = textureSample(source, scale_filter, input.uv).rgb;
    let encoded = select(1.055 * pow(linear, vec3<f32>(1.0 / 2.4)) - 0.055, 12.92 * linear, linear <= vec3<f32>(0.0031308));
    return vec4<f32>(encoded, 1.0);
}
