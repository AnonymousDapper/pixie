// Frame trail mix shader (first step)

[[group(0), binding(0)]]
var tex_sampler: sampler;

[[group(0), binding(1)]]
var surface: texture_2d<f32>;

[[group(0), binding(2)]]
var cut_surface: texture_2d<f32>;

[[stage(fragment)]]
fn main([[location(0)]] coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    let surface_color = textureSample(surface, tex_sampler, coords);

    let cut_color = textureSample(cut_surface, tex_sampler, coords);

    return vec4<f32>(surface_color.rgb + cut_color.rgb, 1.0);
}