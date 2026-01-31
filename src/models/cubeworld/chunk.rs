use std::sync::Arc;

use crate::{
    models::cubeworld::consts::{C, CHUNK_D, CHUNK_H, CHUNK_VOL, CHUNK_W},
    render::{
        drawable::Drawable,
        model::mesh::Mesh,
        vertex::{Vertex, calculate_normals},
    },
    texture::Texture,
};

#[derive(Debug, Clone, Copy)]
pub struct Voxel {
    pub id: u32,
}

pub fn in_chunk_bounds(x: C, y: C, z: C) -> bool {
    x >= 0 && x < CHUNK_W as C && y >= 0 && y < CHUNK_H as C && z >= 0 && z < CHUNK_D as C
}
pub fn is_blocked(voxels: &[Voxel], x: C, y: C, z: C) -> bool {
    in_chunk_bounds(x, y, z)
        && voxels[Chunk::get_voxel_index(x as usize, y as usize, z as usize)].id != 0
}

#[derive(Debug)]
pub struct Chunk {
    voxels: [Voxel; CHUNK_VOL],
    mesh: Mesh,
}

impl Chunk {
    pub fn new(position: &[f32; 3]) -> Self {
        let voxels = Self::init_voxels(position);
        let mut vertices = vec![];

        for y in 0..CHUNK_H {
            for z in 0..CHUNK_D {
                for x in 0..CHUNK_W {
                    let vox = voxels[Self::get_voxel_index(x, y, z)];
                    let voxel_id = vox.id;
                    if voxel_id == 0 {
                        continue;
                    }

                    let (x, y, z): (C, C, C) = (x as C, y as C, z as C);
                    if !is_blocked(&voxels, x, y + 1, z) {
                        #[rustfmt::skip]
                        [
                            // top
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[x as f32, y as f32, z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(&voxels, x, y - 1, z) {
                        #[rustfmt::skip]
                        [
                            // bottom
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 5
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[x as f32, y as f32, z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(&voxels, x + 1, y, z) {
                        #[rustfmt::skip]
                        [
                            // right
                            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
                            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[x as f32, y as f32, z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(&voxels, x - 1, y, z) {
                        #[rustfmt::skip]
                        [
                            // left
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[x as f32, y as f32, z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(&voxels, x, y, z + 1) {
                        #[rustfmt::skip]
                        [
                            // front
                            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 4
                            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
                            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 5
                            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[x as f32, y as f32, z as f32]);
                            vertices.push(v)
                        });
                    }
                    if !is_blocked(&voxels, x, y, z - 1) {
                        #[rustfmt::skip]
                        [
                            // back
                            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
                            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 2
                            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 3
                        ]
                        .into_iter()
                        .for_each(|mut v| {
                            v.add_pos(&[x as f32, y as f32, z as f32]);
                            vertices.push(v)
                        });
                    }
                }
            }
        }
        calculate_normals(&mut vertices);
        let textures = vec![Arc::new(Texture::white()), Arc::new(Texture::black())];

        let mesh = Mesh::new(vertices, vec![], textures, false);
        Self { mesh, voxels }
    }
    fn init_voxels(position: &[f32; 3]) -> [Voxel; CHUNK_VOL] {
        let mut voxels = [Voxel { id: 0 }; CHUNK_VOL];
        for y in 0..CHUNK_H {
            for z in 0..CHUNK_D {
                for x in 0..CHUNK_W {
                    let id = (((x + z) as f32).cos() + 1.0) as u32;
                    voxels[Self::get_voxel_index(x, y, z)].id = id;
                }
            }
        }
        voxels
    }
    fn get_voxel_index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_D + z) * CHUNK_W + x
    }
    pub fn get_voxels(&self) -> &[Voxel; CHUNK_VOL] {
        &self.voxels
    }
    pub fn get_voxel(&self, x: C, y: C, z: C) -> Option<Voxel> {
        if in_chunk_bounds(x, y, z) {
            Some(self.voxels[Self::get_voxel_index(x as usize, y as usize, z as usize)])
        } else {
            None
        }
    }
}

impl Drawable for Chunk {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        self.mesh.draw(glfw, state);
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
