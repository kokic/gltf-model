// Adapted from: https://github.com/bevyengine/bevy/blob/main/examples/shader/texture_binding_array.rs

use std::num::NonZero;

use bevy::{
    asset::{Asset, Handle},
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        alpha::AlphaMode, mesh::MeshVertexBufferLayoutRef, render_asset::RenderAssets, render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            AsBindGroup, AsBindGroupError, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BindGroupLayoutEntry, BindingResources, BufferInitDescriptor,
            BufferUsages, PreparedBindGroup, RenderPipelineDescriptor, SamplerBindingType,
            ShaderRef, ShaderStages, ShaderType, SpecializedMeshPipelineError, TextureSampleType,
            UnpreparedBindGroup,
        }, renderer::RenderDevice, texture::{FallbackImage, GpuImage}
    },
};
use bevy_image::Image;

use crate::built_block_mesh::MAX_BLOCK_TEXTURE_COUNT;

const SHADER_ASSET_PATH: &str = "shaders/block_texture_binding_array.wgsl";

#[derive(Clone, Debug)]
pub struct MaterialUniforms {
    pub texture_count: u32,
}

impl ShaderType for MaterialUniforms {
    type ExtraMetadata = ();

    const METADATA: bevy::render::render_resource::encase::private::Metadata<Self::ExtraMetadata> =
        u32::METADATA;
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct BindlessMaterial {
    pub textures: Vec<Handle<Image>>,
    pub uniforms: MaterialUniforms,
}

const BIND_GROUP_INDEX_OFFSET: u32 = 10;

impl AsBindGroup for BindlessMaterial {
    type Data = ();

    type Param = (SRes<RenderAssets<GpuImage>>, SRes<FallbackImage>);

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        (image_assets, fallback_image): &mut SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        // retrieve the render resources from handles
        let mut images = vec![];
        for handle in self.textures.iter().take(MAX_BLOCK_TEXTURE_COUNT) {
            match image_assets.get(handle) {
                Some(image) => images.push(image),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }

        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_BLOCK_TEXTURE_COUNT];

        // convert bevy's resource types to WGPU's references
        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

        // fill in up to the first `MAX_TEXTURE_COUNT` textures and samplers to the arrays
        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        // Create uniform buffer for material uniforms
        let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("material_uniforms_buffer"),
            contents: &self.uniforms.texture_count.to_ne_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = render_device.create_bind_group(
            "bindless_material_bind_group",
            layout,
            &BindGroupEntries::with_indices((
                (BIND_GROUP_INDEX_OFFSET, &textures[..]),
                (BIND_GROUP_INDEX_OFFSET + 1, &fallback_image.sampler),
                (
                    BIND_GROUP_INDEX_OFFSET + 2,
                    uniform_buffer.as_entire_binding(),
                ),
            )),
        );

        Ok(PreparedBindGroup {
            bindings: BindingResources(vec![]),
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _layout: &BindGroupLayout,
        _render_device: &RenderDevice,
        _param: &mut SystemParamItem<'_, '_, Self::Param>,
        _force_no_bindless: bool,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        // We implement `as_bind_group`` directly because bindless texture
        // arrays can't be owned.
        // Or rather, they can be owned, but then you can't make a `&'a [&'a
        // TextureView]` from a vec of them in `get_binding()`.
        Err(AsBindGroupError::CreateBindGroupDirectly)
    }

    fn bind_group_layout_entries(_: &RenderDevice, _: bool) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (
                    BIND_GROUP_INDEX_OFFSET,
                    texture_2d(TextureSampleType::Float { filterable: true })
                        .count(NonZero::<u32>::new(MAX_BLOCK_TEXTURE_COUNT as u32).unwrap()),
                ),
                (
                    BIND_GROUP_INDEX_OFFSET + 1,
                    sampler(SamplerBindingType::Filtering),
                ),
                (
                    BIND_GROUP_INDEX_OFFSET + 2,
                    uniform_buffer::<MaterialUniforms>(false),
                ),
            ),
        )
        .to_vec()
    }
}

impl Material for BindlessMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(ref mut depth_stencil) = descriptor.depth_stencil {
            depth_stencil.depth_write_enabled = true;
        }

        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}
