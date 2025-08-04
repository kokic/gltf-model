#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_bindings,
    pbr_types,
}

// Custom vertex input that includes block type
struct CustomVertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) block_type: f32, // Custom attribute for block type
}

struct CustomVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) block_type: f32,
}

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;

@vertex
fn vertex(vertex: CustomVertexInput) -> CustomVertexOutput {
    var out: CustomVertexOutput;
    
    // Transform position to world space
    let world_position = mesh_position_local_to_world(
        pbr_bindings::mesh[vertex.instance_index].model,
        vec4<f32>(vertex.position, 1.0)
    );
    
    out.clip_position = mesh_position_world_to_clip(world_position);
    out.world_position = world_position;
    out.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    out.uv = vertex.uv;
    out.block_type = vertex.block_type;
    
    return out;
}

@fragment
fn fragment(
    mesh: CustomVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    // Use the block_type vertex attribute to select texture directly
    let texture_index = u32(mesh.block_type) % 4u;
    
    // Sample the texture
    var base_color = textureSample(textures[texture_index], nearest_sampler, mesh.uv);
    
    // Create PBR input
    var pbr_input = pbr_types::pbr_input_new();
    
    pbr_input.material.base_color = base_color;
    pbr_input.material.reflectance = 0.04;
    pbr_input.material.perceptual_roughness = 0.8;
    pbr_input.material.metallic = 0.0;
    
    pbr_input.frag_coord = mesh.clip_position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = pbr_functions::prepare_world_normal(
        mesh.world_normal,
        false,
        is_front,
    );
    
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = normalize(pbr_bindings::view.world_position.xyz - pbr_input.world_position.xyz);
    
    var output_color = apply_pbr_lighting(pbr_input);
    output_color = main_pass_post_lighting_processing(pbr_input, output_color);
    
    return output_color;
}
