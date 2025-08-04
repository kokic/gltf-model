#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;
@group(2) @binding(2) var<uniform> material_uniforms: MaterialUniforms;

struct MaterialUniforms {
    texture_count: u32,
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let texture_count_f32 = f32(material_uniforms.texture_count);
    let texture_index = u32(mesh.uv.x * texture_count_f32);    
    let internal_uv = vec2<f32>(fract(mesh.uv.x * texture_count_f32), mesh.uv.y);
    return textureSample(textures[texture_index], nearest_sampler, internal_uv);
}