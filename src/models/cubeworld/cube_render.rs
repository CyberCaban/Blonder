use std::sync::Arc;

use anyhow::Result;
use glfw::Glfw;

use crate::{
    models::cubeworld::{
        chunk::{Chunk, Voxel},
        chunk_mesh::ChunkMesh,
        chunks::Chunks,
    },
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
                wrap_s: gl::CLAMP_TO_EDGE as i32,
                wrap_t: gl::CLAMP_TO_EDGE as i32,
                mipmap_filtering: gl::LINEAR_MIPMAP_NEAREST as i32,
                ..Default::default()
            },
        )?);
        let mut chunk_meshes = Vec::with_capacity(chunks.get_volume());
        for chunk in &chunks.chunks {
            chunk_meshes.push(ChunkMesh::from_chunk(chunk, &chunks, atlas.clone())?);
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
        unsafe {
            gl::Enable(gl::POLYGON_OFFSET_FILL);
            gl::PolygonOffset(0.0, 2.0);
        }
        for chunk in &self.chunk_meshes {
            chunk.draw(glfw, state);
        }
        unsafe {
            gl::Disable(gl::POLYGON_OFFSET_FILL);
        }
    }
    fn update(&mut self, state: &crate::state::State) {
        let raycast = self.chunks.raycast(
            state.camera.position.into(),
            state.camera.front.into(),
            10.0,
        );
        if raycast.is_hit {
            if state.mouse_left_just_pressed() {
                self.chunks.set_voxel(
                    raycast.hit_coords[0],
                    raycast.hit_coords[1],
                    raycast.hit_coords[2],
                    Voxel { id: 0 },
                );
            }
            if state.mouse_right_just_pressed() {
                self.chunks.set_voxel(
                    raycast.hit_coords[0] + raycast.normal[0] as i32,
                    raycast.hit_coords[1] + raycast.normal[1] as i32,
                    raycast.hit_coords[2] + raycast.normal[2] as i32,
                    Voxel { id: 1 },
                );
            }
        }
        for (i, chunk) in self.chunks.chunks.iter().enumerate() {
            if !chunk.is_modified() {
                continue;
            }
            chunk.reset_modified();
            self.chunk_meshes[i] =
                ChunkMesh::from_chunk(chunk, &self.chunks, self.texture_atlas.clone()).unwrap();
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
