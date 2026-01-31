use std::{ptr, sync::Arc};

use anyhow::Result;

use crate::{
    models::cubeworld::{
        chunk::{Chunk, Voxel},
        consts::{ATLAS_SIDE, C, CHUNK_D, CHUNK_H, CHUNK_W, UV_SIZE},
    },
    render::{
        blend_mode::BlendMode,
        drawable::Drawable,
        helpers::{set_buffer_data, set_buffer_data_with_indices},
        renderer::TextureRef,
        shader::ShaderInfo,
        vertex::{Vertex, calculate_normals},
    },
};

pub fn in_chunk_bounds(x: C, y: C, z: C) -> bool {
    x >= 0 && x < CHUNK_W as C && y >= 0 && y < CHUNK_H as C && z >= 0 && z < CHUNK_D as C
}
pub fn is_blocked(voxels: &[Voxel], x: C, y: C, z: C) -> bool {
    in_chunk_bounds(x, y, z)
        && voxels[Chunk::get_voxel_index(x as usize, y as usize, z as usize)].id != 0
}

#[derive(Debug)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub texture_atlas: TextureRef,
    vao: u32,
    vbo: u32,
}

impl ChunkMesh {
    pub fn from_chunk(chunk: &Chunk, atlas: TextureRef) -> Result<Self> {
        let mut vertices = vec![];
        let voxels = chunk.get_voxels();

        for y in 0..CHUNK_H {
            for z in 0..CHUNK_D {
                for x in 0..CHUNK_W {
                    let vox = chunk.get_voxel(x as i32, y as i32, z as i32);
                    if vox.is_none() {
                        continue;
                    }
                    let vox = vox.unwrap();
                    let voxel_id = vox.id;
                    if voxel_id == 0 {
                        continue;
                    }

                    let (global_x, global_y, global_z): (C, C, C) = (
                        x as C + chunk.position[0] * CHUNK_W as C,
                        y as C + chunk.position[1] * CHUNK_H as C,
                        z as C + chunk.position[2] * CHUNK_D as C,
                    );
                    let (local_x, local_y, local_z) = (x as C, y as C, z as C);
                    let u = (voxel_id % ATLAS_SIDE as u32) as f32 * UV_SIZE;
                    let v = (voxel_id / ATLAS_SIDE as u32) as f32 * UV_SIZE;
                    if !is_blocked(voxels, local_x, local_y + 1, local_z) {
                        #[rustfmt::skip]
                        [
                            // top
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u+UV_SIZE, v],normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u+UV_SIZE, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u, v], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u, v], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u+UV_SIZE, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, 0.5, 0.5], uv: [u, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32, global_y as f32, global_z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(voxels, local_x, local_y - 1, local_z) {
                        #[rustfmt::skip]
                        [
                            // bottom
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u, v],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u+UV_SIZE, v], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u+UV_SIZE, v], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u+UV_SIZE, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 5
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32, global_y as f32, global_z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(voxels, local_x + 1, local_y, local_z) {
                        #[rustfmt::skip]
                        [
                            // right
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u, v],normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u+UV_SIZE, v], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u+UV_SIZE, v], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, 0.5, 0.5], uv: [u+UV_SIZE, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32, global_y as f32, global_z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(voxels, local_x - 1, local_y, local_z) {
                        #[rustfmt::skip]
                        [
                            // left
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u, v], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u+UV_SIZE, v],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u+UV_SIZE, v],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u+UV_SIZE, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 4
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32, global_y as f32, global_z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(voxels, local_x, local_y, local_z + 1) {
                        #[rustfmt::skip]
                        [
                            // front
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u, v],normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u+UV_SIZE, v], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u+UV_SIZE, v], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, 0.5, 0.5], uv: [u+UV_SIZE, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32, global_y as f32, global_z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(voxels, local_x, local_y, local_z - 1) {
                        #[rustfmt::skip]
                        [
                            // back
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u+UV_SIZE, v],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u+UV_SIZE, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u, v], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u, v], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u+UV_SIZE, v+UV_SIZE], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u, v+UV_SIZE],  normal: [0.0, 0.0, 0.0] }, // 3
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32, global_y as f32, global_z as f32]);
                            vertices.push(v)
                        });
                    }
                }
            }
        }
        calculate_normals(&mut vertices);

        let mut mesh = Self {
            texture_atlas: atlas,
            vertices,
            vao: 0,
            vbo: 0,
        };
        mesh.load_mesh();
        Ok(mesh)
    }
    pub fn new(vertices: Vec<Vertex>, texture_atlas: TextureRef) -> Self {
        let mut mesh = Self {
            texture_atlas,
            vertices,
            vao: 0,
            vbo: 0,
        };
        mesh.load_mesh();
        mesh
    }
    fn load_mesh(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            set_buffer_data(self.vao, self.vbo, &self.vertices);
        }
    }
}

impl Drawable for ChunkMesh {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        // TODO: set textures
        let diffuse_number = 1;
        let specular_number = 1;
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_atlas.id());
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32);
            gl::BindVertexArray(0);
        }
    }
    fn get_blend_mode(&self) -> BlendMode {
        BlendMode::Opaque
    }
    fn get_shader_name(&self) -> Option<ShaderInfo> {
        None
    }
    fn get_texture_config(&self) -> Option<crate::texture::TextureConfig> {
        None
    }
    fn get_texture_name(&self) -> Option<String> {
        None
    }
}

impl Drop for ChunkMesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
