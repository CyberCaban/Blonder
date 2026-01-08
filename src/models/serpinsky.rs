use crate::{
    render::{color::Color, drawable::Drawable, helpers::set_buffer_data, vertex::Vertex},
    shader::ShaderInfo,
    texture::Texture,
};

use anyhow::Result;
use cgmath::{Matrix4, Rad, SquareMatrix};

#[derive(Debug)]
pub struct Serpinsky {
    pub points: Vec<Vertex>,
    count: u32,
    pub vao: u32,
    pub shader: ShaderInfo,
    pub texture: Texture,
}

impl Serpinsky {
    pub fn new() -> Result<Self> {
        Ok(Self {
            points: vec![],
            vao: 0,
            count: 0,
            shader: ShaderInfo {
                name: "serpinsky".to_string(),
                vertex_path: "assets/shaders/serpinsky/vert.glsl".to_string(),
                fragment_path: "assets/shaders/serpinsky/frag.glsl".to_string(),
            },
            texture: Texture::new("assets/textures/cooler.png")?,
        })
    }
    pub fn serp(
        &mut self,
        point_a: &[f32; 3],
        point_b: &[f32; 3],
        point_c: &[f32; 3],
        mut depth: u32,
    ) {
        if depth == 0 {
            return;
        }
        fn middle(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
            [
                (a[0] + b[0]) / 2.0,
                (a[1] + b[1]) / 2.0,
                (a[2] + b[2]) / 2.0,
            ]
        }
        let (px, py, pz) = (
            middle(point_a, point_b),
            middle(point_a, point_c),
            middle(point_b, point_c),
        );

        self.points.extend_from_slice(&[
            Vertex {
                position: px,
                uv: [0.5, -0.5],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: py,
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: pz,
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
        ]);

        depth -= 1;
        self.serp(point_a, &px, &py, depth);
        self.serp(&px, point_b, &pz, depth);
        self.serp(&py, &pz, point_c, depth);
    }
    pub fn prepare(&mut self) {
        let (mut vbo, mut vao) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            set_buffer_data(vao, vbo, &self.points);
        }
        self.vao = vao;
        self.count = self.points.len() as u32;
        self.points.clear();
    }
}

impl Drawable for Serpinsky {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.count as i32);
        }
    }
    fn get_texture_name(&self) -> String {
        "assets/textures/cooler.png".to_string()
        // "".to_string()
    }
    fn get_shader_name(&self) -> ShaderInfo {
        self.shader.clone()
    }
    fn requires_shader(&self) -> bool {
        false
    }
    fn requires_texture(&self) -> bool {
        false
    }
    fn get_blend_mode(&self) -> crate::render::blend_mode::BlendMode {
        crate::render::blend_mode::BlendMode::Opaque
    }
}
