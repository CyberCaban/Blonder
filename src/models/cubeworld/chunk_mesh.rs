use std::{ptr, sync::Arc};

use anyhow::Result;

use crate::{
    models::cubeworld::{
        chunk::{Chunk, Voxel},
        chunks::Chunks,
        consts::{ATLAS_SIDE, C, CHUNK_D, CHUNK_H, CHUNK_W, UV_SIZE},
    },
    render::{
        blend_mode::BlendMode,
        drawable::Drawable,
        helpers::set_buffer_data,
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
pub fn chunk_div(value: C, chunk_size: usize) -> C {
    if value < 0 {
        value / chunk_size as C - 1
    } else {
        value / chunk_size as C
    }
}

pub fn to_local_coord(global: C, chunk_size: usize) -> usize {
    let size = chunk_size as C;
    if global >= 0 {
        (global % size) as usize
    } else {
        ((size + (global % size)) % size) as usize
    }
}

pub fn is_blocked_with_neighbors(chunks: &Chunks, global_x: C, global_y: C, global_z: C) -> bool {
    let chunk_x = chunk_div(global_x, CHUNK_W);
    let chunk_y = chunk_div(global_y, CHUNK_H);
    let chunk_z = chunk_div(global_z, CHUNK_D);

    let chunk_x_idx = chunk_x;
    let chunk_y_idx = chunk_y;
    let chunk_z_idx = chunk_z;

    if chunk_x_idx < 0
        || chunk_y_idx < 0
        || chunk_z_idx < 0
        || chunk_x_idx >= chunks.dimensions.width_in_chunks as i32
        || chunk_y_idx >= chunks.dimensions.height_in_chunks as i32
        || chunk_z_idx >= chunks.dimensions.depth_in_chunks as i32
    {
        return false;
    }

    let chunk = chunks.get_chunk(chunk_x_idx, chunk_y_idx, chunk_z_idx);
    if chunk.is_none() {
        return false;
    }
    let chunk = chunk.unwrap();

    let local_x = to_local_coord(global_x, CHUNK_W);
    let local_y = to_local_coord(global_y, CHUNK_H);
    let local_z = to_local_coord(global_z, CHUNK_D);

    if let Some(voxel) = chunk.get_voxel(local_x as C, local_y as C, local_z as C) {
        voxel.id != 0
    } else {
        false
    }
}

pub fn is_neighbor_blocked(
    chunk: &Chunk,
    chunks: &Chunks,
    local_x: C,
    local_y: C,
    local_z: C,
    dx: C,
    dy: C,
    dz: C,
) -> bool {
    let global_x = local_x + chunk.position[0] * CHUNK_W as C + dx;
    let global_y = local_y + chunk.position[1] * CHUNK_H as C + dy;
    let global_z = local_z + chunk.position[2] * CHUNK_D as C + dz;

    is_blocked_with_neighbors(chunks, global_x, global_y, global_z)
}

#[derive(Debug)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub texture_atlas: TextureRef,
    vao: u32,
    vbo: u32,
}

impl ChunkMesh {
    pub fn from_chunk(chunk: &Chunk, chunks: &Chunks, atlas: TextureRef) -> Result<Self> {
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
                    let pad = 0.0005;
                    let u = (voxel_id % ATLAS_SIDE as u32) as f32 * UV_SIZE;
                    let v = (voxel_id / ATLAS_SIDE as u32) as f32 * UV_SIZE;
                    let u_min = u + pad;
                    let u_max = u + UV_SIZE - pad;
                    let v_min = v + pad;
                    let v_max = v + UV_SIZE - pad;
                    if !is_neighbor_blocked(chunk, chunks, local_x, local_y, local_z, 0, 1, 0) {
                        #[rustfmt::skip]
                        [
                            // top
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u_max, v_min],normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u_max, v_max], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u_min, v_min], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u_min, v_min], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u_max, v_max], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, 0.5, 0.5], uv: [u_min, v_max],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32 + 0.5, global_y as f32 + 0.5, global_z as f32 + 0.5]);
                            vertices.push(v)
                        });
                    }
                    if !is_neighbor_blocked(chunk, chunks, local_x, local_y, local_z, 0, -1, 0) {
                        #[rustfmt::skip]
                        [
                            // bottom
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u_min, v_min],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u_max, v_min], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u_min, v_max], normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u_min, v_max], normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u_max, v_min], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u_max, v_max],  normal: [0.0, 0.0, 0.0] }, // 5
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32 + 0.5, global_y as f32 + 0.5, global_z as f32 + 0.5]);
                            vertices.push(v)
                        });
                    }
                    if !is_neighbor_blocked(chunk, chunks, local_x, local_y, local_z, 1, 0, 0) {
                        #[rustfmt::skip]
                        [
                            // right
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u_min, v_min],normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u_max, v_min], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u_min, v_max], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u_min, v_max], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u_max, v_min], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, 0.5, 0.5], uv: [u_max, v_max],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32 + 0.5, global_y as f32 + 0.5, global_z as f32 + 0.5]);
                            vertices.push(v)
                        });
                    }
                    if !is_neighbor_blocked(chunk, chunks, local_x, local_y, local_z, -1, 0, 0) {
                        #[rustfmt::skip]
                        [
                            // left
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u_min, v_min], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u_max, v_min],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u_min, v_max],  normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u_min, v_max],  normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u_max, v_min],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u_max, v_max], normal: [0.0, 0.0, 0.0] }, // 4
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32 + 0.5, global_y as f32 + 0.5, global_z as f32 + 0.5]);
                            vertices.push(v)
                        });
                    }
                    if !is_neighbor_blocked(chunk, chunks, local_x, local_y, local_z, 0, 0, 1) {
                        #[rustfmt::skip]
                        [
                            // front
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [u_min, v_min],normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u_max, v_min], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u_min, v_max], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [u_min, v_max], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, -0.5, 0.5], uv: [u_max, v_min], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, 0.5, 0.5], uv: [u_max, v_max],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32 + 0.5, global_y as f32 + 0.5, global_z as f32 + 0.5]);
                            vertices.push(v)
                        });
                    }
                    if !is_neighbor_blocked(chunk, chunks, local_x, local_y, local_z, 0, 0, -1) {
                        #[rustfmt::skip]
                        [
                            // back
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [u_max, v_min],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u_max, v_max], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u_min, v_min], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, -0.5, -0.5], uv: [u_min, v_min], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [u_max, v_max], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [0.5, 0.5, -0.5], uv: [u_min, v_max],  normal: [0.0, 0.0, 0.0] }, // 3
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[global_x as f32 + 0.5, global_y as f32 + 0.5, global_z as f32 + 0.5]);
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
    fn update(&mut self, state: &crate::state::State) {}
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
