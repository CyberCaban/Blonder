use std::ptr;

use anyhow::Result;
use cgmath::{Deg, InnerSpace, Matrix4, Rad, Vector3, perspective};
use glfw::Glfw;

use crate::{
    render::{
        helpers::{set_buffer_data, set_buffer_data_with_indices},
        vertex::Vertex,
    },
    shader::Shader,
    state::State,
    texture::Texture,
};

pub struct Cube {
    pub points: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub position: Vector3<f32>,
    pub vao: u32,
    pub texture: Texture,
    pub shader: Shader,
}

impl Cube {
    pub fn new(texture_path: &str, position: &[f32; 3]) -> Result<Self> {
        // TODO make vertices render CCW to enable backface culling
        #[rustfmt::skip]
        let mut points = vec![
            // bottom
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] }, // 0
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 2
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 1
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 1
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 2
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 3
            // top
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] }, // 4
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 5
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 6
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 6
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 5
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 7
            // front
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] }, // 0
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 1
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 4
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 4
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 1
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 5
            // back
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] }, // 2
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 6
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 3
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 3
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 6
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 7
            // left
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 2
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0], color: [1.0, 1.0, 1.0] }, // 0
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 6
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 6
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0], color: [1.0, 1.0, 1.0] }, // 0
            Vertex { position: [-0.5, -0.5, 0.5], uv: [1.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 4
            // right
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] }, // 1
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 3
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 5
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  [1.0, 1.0, 1.0] }, // 5
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  [1.0, 1.0, 1.0] }, // 3
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0], color:   [1.0, 1.0, 1.0] }, // 7

        ];
        points.iter_mut().for_each(|v| v.add_pos(position));
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
        // set_buffer_data_with_indices(vao, vbo, ebo, &points, &indices);
        set_buffer_data(vao, vbo, &points);
        Ok(Self {
            points: vec![],
            indices: vec![],
            vao,
            position: Vector3::from(*position),
            shader: Shader::new("shaders/cube/vert.glsl", "shaders/cube/frag.glsl")?,
            texture: Texture::new(texture_path)?,
        })
    }
    pub fn draw(&self, glfw: &Glfw, state: &State) {
        unsafe {
            // self.shader.use_shader();
            // let aspect = (state.screen.width as f32 / state.screen.height as f32);
            // let projection_matrix = perspective(Deg(45.0), (aspect as f32), 0.01, 100.0);
            // let rotation = Matrix4::from_axis_angle(
            //     Vector3::new(0.5, 1.0, 0.0).normalize(),
            //     Rad(1.0) * glfw.get_time() as f32,
            // );
            // let model_matrix =
            //     Matrix4::from_translation(self.position + Vector3::new(0.0, 0.0, 0.0)) * rotation
            //     ;
            // let view_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, -3.0));
            // let mvp = projection_matrix * view_matrix * model_matrix;
            // self.shader.set_mat4("mvp", &mvp);
            gl::ActiveTexture(gl::TEXTURE0);
            self.texture.use_texture();
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36 as i32);
        }
    }
}
