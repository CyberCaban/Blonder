use std::ptr;

use anyhow::Result;
use cgmath::Vector3;

use crate::{
    render::{
        blend_mode::BlendMode,
        drawable::Drawable,
        helpers::{set_buffer_data, set_buffer_data_with_indices, set_buffer_data_with_indices_u8},
        shader::ShaderInfo,
        vertex::{
            calculate_normals, calculate_normals_indexed, calculate_normals_indexed_u8, Vertex,
        },
    },
    state::State,
    texture::TextureConfig,
};

#[derive(Debug)]
pub struct BlockSettings<'a> {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub texture_name: &'a str,
    pub shader_name: ShaderInfo,
    pub blend_mode: BlendMode,
    pub texture_config: TextureConfig,
}

impl Default for BlockSettings<'_> {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            texture_name: "",
            shader_name: ShaderInfo {
                name: "cube".to_string(),
                fragment_path: "assets/shaders/light/frag.glsl".to_string(),
                vertex_path: "assets/shaders/light/vert.glsl".to_string(),
            },
            blend_mode: BlendMode::default(),
            texture_config: TextureConfig::default(),
        }
    }
}

pub struct Block {
    points: Vec<Vertex>,
    position: Vector3<f32>,
    vbo: u32,
    vao: u32,
    ebo: u32,
    texture: String,
    texture_config: TextureConfig,
    shader_info: ShaderInfo,
    blend_mode: BlendMode,
}

impl Block {
    pub fn new(settings: BlockSettings) -> Result<Self> {
        #[rustfmt::skip]
        let mut points = vec![
            // back
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 2
            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 3
            // front
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 4
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 5
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7
            // bottom
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0 <- 8
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1 <- 9
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4 <- 10
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 5 <- 11
            // top
            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 2 <- 12
            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3 <- 13
            Vertex { position: [-0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6 <- 14
            Vertex { position: [0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7 <- 15
            // left
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0 <- 16
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 2 <- 17
            Vertex { position: [-0.5, -0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4 <- 18
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 6 <- 19
            // right
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 1 <- 20
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3 <- 21
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 5 <- 22
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7 <- 23

        ];
        #[rustfmt::skip]
        let indices: [u8; 36] = [
            // back
            2, 1, 0, 2, 3, 1,
            // front
            4, 5, 6, 6, 5, 7,
            // bottom
            8, 11, 10, 9, 11, 8,
            // top
            14, 15, 12, 12, 15, 13,
            // left
            16, 18, 17, 17, 18, 19,
            // right
            22, 20, 23, 23, 20, 21,
        ];
        points.iter_mut().for_each(|v| {
            v.add_pos(&settings.position);
            v.rotate_around(&settings.position, &settings.rotation);
        });
        let _ = calculate_normals_indexed_u8(&mut points, &indices);
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }
        set_buffer_data_with_indices_u8(vao, vbo, ebo, &points, &indices);
        Ok(Self {
            points: vec![],
            vao,
            vbo,
            ebo,
            position: Vector3::from(settings.position),
            shader_info: settings.shader_name,
            blend_mode: settings.blend_mode,
            texture: settings.texture_name.to_owned(),
            texture_config: settings.texture_config,
        })
    }
}

impl Drawable for Block {
    fn draw(&self, glfw: &glfw::Glfw, state: &State) {
        unsafe {
            // gl::ActiveTexture(gl::TEXTURE0);
            // self.texture.use_texture();
            gl::BindVertexArray(self.vao);
            // gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_BYTE, ptr::null());
        }
    }
    fn update(&mut self, state: &State) {}
    fn get_texture_name(&self) -> Option<String> {
        Some(self.texture.to_string())
    }
    fn get_shader_name(&self) -> Option<ShaderInfo> {
        None
    }
    fn get_texture_config(&self) -> Option<TextureConfig> {
        Some(self.texture_config)
    }
    fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
