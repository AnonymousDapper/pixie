// Frame trail cut shader (second step)

[[group(0), binding(0)]]
var tex_sampler: sampler;

[[group(0), binding(1)]]
var surface: texture_2d<f32>;

struct Fragment {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] cut: vec4<f32>;
};

[[stage(fragment)]]
fn main([[location(0)]] coords: vec2<f32>) -> Fragment {
    var out: Fragment;
    let color = textureSample(surface, tex_sampler, coords);

    out.color = color;

    out.cut = vec4<f32>(color.rgb / vec3<f32>(1.25), min(color.a, 0.95));

    return out;
}