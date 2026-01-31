use std::sync::Arc;

use anyhow::Result;

use crate::{
    models::cubeworld::{chunk::Chunk, chunk_mesh::ChunkMesh, chunks::Chunks},
    render::{drawable::Drawable, renderer::TextureRef},
    texture::{Texture, TextureConfig},
};

pub struct CubeRenderer {
    texture_atlas: TextureRef,
    chunks: Chunks,
    chunk_meshes: Vec<ChunkMesh>,
}

impl CubeRenderer {
    pub fn new(chunks: Chunks) -> Result<Self> {
        let atlas = Arc::new(Texture::with_config(
            "assets/textures/blocks/TextureAtlas.png",
            TextureConfig {
                texture_filtering: gl::NEAREST as i32,
                wrap_s: gl::REPEAT as i32,
                wrap_t: gl::REPEAT as i32,
                ..Default::default()
            },
        )?);
        let mut chunk_meshes = Vec::with_capacity(chunks.get_volume());
        for i in 0..chunks.get_volume() {
            chunk_meshes.push(ChunkMesh::from_chunk(&chunks.chunks[i], atlas.clone())?);
        }
        Ok(Self {
            chunks,
            texture_atlas: atlas,
            chunk_meshes,
        })
    }
}

impl Drawable for CubeRenderer {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        for chunk in &self.chunk_meshes {
            chunk.draw(glfw, state);
        }
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
