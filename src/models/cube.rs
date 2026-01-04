use std::ptr;

use anyhow::Result;

use crate::{
    render::{helpers::set_buffer_data_with_indices, vertex::Vertex},
    texture::Texture,
};

pub struct Cube {
    pub points: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vao: u32,
    pub texture: Texture,
}

impl Cube {
    pub fn new(texture_path: &str) -> Result<Self> {
        #[rustfmt::skip]
        let points = vec![
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], color: [0.0, 0.0, 0.0] },
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 0.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 0.0] },
        ];
        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2, 2, 3, 1, // down
            4, 5, 6, 6, 7, 5, // up
            0, 1, 4, 4, 5, 1, // front
            2, 3, 6, 6, 7, 3, // back
            2, 0, 6, 6, 4, 0, // left
            1, 3, 5, 5, 7, 3, // right
        ];
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }
        set_buffer_data_with_indices(vao, vbo, ebo, &points, &indices);
        Ok(Self {
            points,
            indices,
            vao,
            texture: Texture::new(texture_path)?,
        })
    }
    pub fn draw(&self) {
        self.texture.use_texture();
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}
