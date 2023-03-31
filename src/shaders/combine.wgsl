// Bloom recombining pass

[[group(0), binding(0)]]
var tex_sampler: sampler;

[[group(0), binding(1)]]
var surface: texture_2d<f32>;

[[group(0), binding(2)]]
var mix_surface: texture_2d<f32>;


[[stage(fragment)]]
fn main([[location(0)]] coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    let frame = textureSample(surface, tex_sampler, coords);

    let mix = textureSample(mix_surface, tex_sampler, coords);

    return frame + mix + (1.0 - frame.a);
    //return vec4<f32>(frame + mix, 1.0);
}