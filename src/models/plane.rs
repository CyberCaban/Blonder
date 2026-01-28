use anyhow::Result;
use cgmath::Vector3;

use crate::{
    render::shader::ShaderInfo,
    render::{
        blend_mode::BlendMode,
        drawable::Drawable,
        helpers::set_buffer_data,
        vertex::{Vertex, calculate_normals},
    },
    state::State,
};

pub struct Plane {
    pub position: Vector3<f32>,
    pub vbo: u32,
    pub vao: u32,
    pub texture: String,
    pub shader_info: ShaderInfo,
    pub blend_mode: BlendMode,
}

impl Plane {
    pub fn new(points: [[f32; 3]; 4], position: [f32; 3]) -> Result<Self> {
        #[rustfmt::skip]
        let mut points = [
            Vertex { position: points[0], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] },
            Vertex { position: points[2], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: points[1], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: points[1], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: points[2], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: points[3], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] },
        ];
        points.iter_mut().for_each(|v| {
            v.add_pos(&position);
        });
        calculate_normals(&mut points);
        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
        }
        // set_buffer_data_with_indices(vao, vbo, ebo, &points, &indices);
        set_buffer_data(vao, vbo, &points);
        Ok(Self {
            position: Vector3::from(position),
            vbo,
            vao,
            texture: "assets/textures/white.png".to_string(),
            shader_info: ShaderInfo {
                name: "plane".to_string(),
                fragment_path: "assets/shaders/light/frag.glsl".to_string(),
                vertex_path: "assets/shaders/light/vert.glsl".to_string(),
            },
            blend_mode: BlendMode::Opaque,
        })
    }
}

impl Drawable for Plane {
    fn draw(&self, glfw: &glfw::Glfw, state: &State) {
        unsafe {
            // gl::ActiveTexture(gl::TEXTURE0);
            // self.texture.use_texture();
            gl::Disable(gl::CULL_FACE);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::Enable(gl::CULL_FACE);
        }
    }
    fn get_texture_name(&self) -> Option<String> {
        Some(self.texture.to_string())
    }
    fn get_shader_name(&self) -> Option<ShaderInfo> {
        None
    }
    fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
    fn get_texture_config(&self) -> Option<crate::texture::TextureConfig> {
        None
    }
}

impl Drop for Plane {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
