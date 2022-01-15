// HDR tonemap fragment shader

[[group(0), binding(0)]]
var tex_sampler: sampler;

[[group(0), binding(1)]]
var surface: texture_2d<f32>;

let gamma: f32 = 2.2;
let exposure: f32 = 1.0;

[[stage(fragment)]]
fn main([[location(0)]] coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    let hdr = textureSample(surface, tex_sampler, coords).rgb;

    // Reinhard
    //var mapped: vec3<f32> = hdr / (hdr + vec3<f32>(1.0));

    // exposure
    var mapped: vec3<f32> = vec3<f32>(1.0) - exp(-hdr * exposure);

    //mapped = pow(mapped, vec3<f32>(1.0 / gamma));

    //mapped = pow(mapped, vec3<f32>(gamma));

    return vec4<f32>(mapped, 1.0);

}