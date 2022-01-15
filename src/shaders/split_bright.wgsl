// Draw fragments over brightness threshold on to separate texture

[[group(0), binding(0)]]
var tex_sampler: sampler;

[[group(0), binding(1)]]
var surface: texture_2d<f32>;

struct MultiFrag {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] bright_color: vec4<f32>;
};

[[stage(fragment)]]
fn main([[location(0)]] coords: vec2<f32>) -> MultiFrag {
    var out: MultiFrag;

    let sampled = textureSample(surface, tex_sampler, coords).rgb;

    out.color = vec4<f32>(sampled, 1.0);

    let bright: f32 = dot(sampled, vec3<f32>(0.2126, 0.7152, 0.0722));

    if (bright > 1.0) {
        out.bright_color = vec4<f32>(sampled, 1.0);
    } else {
        out.bright_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    };

    return out;
}