// Vertex stage

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] idx: u32) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coord = vec2<f32>(select(0.0, 2.0, idx == u32(2)), select(0.0, 2.0, idx == u32(1)));
    out.position = vec4<f32>(((out.tex_coord * vec2<f32>(2.0, -2.0)) + vec2<f32>(-1.0, 1.0)), 1.0, 1.0);

    return out;

}