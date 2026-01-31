use std::sync::Arc;

use anyhow::Result;

use crate::{
    models::cubeworld::{chunk::Chunk, chunk_mesh::ChunkMesh},
    render::{drawable::Drawable, renderer::TextureRef},
    texture::{Texture, TextureConfig},
};

pub struct CubeRenderer {
    texture_atlas: TextureRef,
    chunk: Chunk,
    chunk_mesh: ChunkMesh,
}

impl CubeRenderer {
    pub fn new(chunk: Chunk) -> Result<Self> {
        let atlas = Arc::new(Texture::with_config(
            "assets/textures/blocks/TextureAtlas.png",
            TextureConfig {
                texture_filtering: gl::NEAREST as i32,
                wrap_s: gl::CLAMP_TO_EDGE as i32,
                wrap_t: gl::CLAMP_TO_EDGE as i32,
                ..Default::default()
            },
        )?);
        let chunk_mesh = ChunkMesh::from_chunk(&chunk, atlas.clone())?;
        Ok(Self {
            chunk,
            texture_atlas: atlas,
            chunk_mesh,
        })
    }
}

impl Drawable for CubeRenderer {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        self.chunk_mesh.draw(glfw, state);
    }
    fn get_blend_mode(&self) -> crate::render::blend_mode::BlendMode {
        crate::render::blend_mode::BlendMode::Opaque
    }
    fn get_shader_name(&self) -> Option<crate::render::shader::ShaderInfo> {
        None
    }
    fn get_texture_config(&self) -> Option<crate::texture::TextureConfig> {
        None
    }
    fn get_texture_name(&self) -> Option<String> {
        None
    }
}
