#import bevy_pbr::{
    pbr_types::{PbrInput, pbr_input_new},
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::view,
    mesh_bindings::mesh,
    utils::{coords_to_viewport_uv},
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
}
#endif

@group(2) @binding(10) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(11) var nearest_sampler: sampler;
@group(2) @binding(12) var<uniform> material_uniforms: MaterialUniforms;

struct MaterialUniforms {
    texture_count: u32,
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Sample texture based on UV coordinates
    let texture_count_f32 = f32(material_uniforms.texture_count);
    let texture_index = u32(in.uv.x * texture_count_f32);    
    let internal_uv = vec2<f32>(fract(in.uv.x * texture_count_f32), in.uv.y);
    let base_color = textureSample(textures[texture_index], nearest_sampler, internal_uv);
    
    var pbr_input = pbr_input_new();
    
    pbr_input.material.base_color = base_color;
    pbr_input.material.perceptual_roughness = 0.8;
    pbr_input.material.metallic = 0.0;
    pbr_input.material.flags = 0u;
    
    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = in.world_normal;
    
    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    pbr_input.N = normalize(in.world_normal);
    pbr_input.V = V;
    
    if (!is_front) {
        pbr_input.N = -pbr_input.N;
    }
    
    // Alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // Apply PBR lighting calculations
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}