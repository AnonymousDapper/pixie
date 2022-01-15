// HDR tonemap fragment shader

struct Uniforms {
    horizontal: u32;
};

[[group(0), binding(0)]]
var tex_sampler: sampler;

[[group(0), binding(1)]]
var surface: texture_2d<f32>;

[[group(1), binding(0)]]
var<uniform> data: Uniforms;

let weight = array<f32, 6>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216, 0.003385);


[[stage(fragment)]]
fn main([[location(0)]] coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    //return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let offset = 1.0 / vec2<f32>(textureDimensions(surface));

    var result: vec3<f32> = textureSample(surface, tex_sampler, coords).rgb * weight[0];

    if (data.horizontal == 1u) {

            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(offset.x * 1.0, 0.0)).rgb * weight[1];
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(offset.x * 2.0, 0.0)).rgb * weight[2];
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(offset.x * 3.0, 0.0)).rgb * weight[3];
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(offset.x * 4.0, 0.0)).rgb * weight[4];
            //result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(offset.x * 5.0, 0.0)).rgb * weight[5];

            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(offset.x * 1.0, 0.0)).rgb * weight[1];
            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(offset.x * 2.0, 0.0)).rgb * weight[2];
            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(offset.x * 3.0, 0.0)).rgb * weight[3];
            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(offset.x * 4.0, 0.0)).rgb * weight[4];
            //result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(offset.x * 5.0, 0.0)).rgb * weight[5];
        
    } else {
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(0.0, offset.y * 1.0)).rgb * weight[1];
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(0.0, offset.y * 2.0)).rgb * weight[2];
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(0.0, offset.y * 3.0)).rgb * weight[3];
            result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(0.0, offset.y * 4.0)).rgb * weight[4];
            //result = result + textureSample(surface, tex_sampler, coords + vec2<f32>(0.0, offset.y * 5.0)).rgb * weight[5];

            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(0.0, offset.y * 1.0)).rgb * weight[1];
            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(0.0, offset.y * 2.0)).rgb * weight[2];
            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(0.0, offset.y * 3.0)).rgb * weight[3];
            result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(0.0, offset.y * 4.0)).rgb * weight[4];
            //result = result + textureSample(surface, tex_sampler, coords - vec2<f32>(0.0, offset.y * 5.0)).rgb * weight[5];
    };

    return vec4<f32>(result, 1.0);

}