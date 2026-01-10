use anyhow::Result;
use cgmath::Vector3;

use crate::{
    render::{
        blend_mode::BlendMode,
        drawable::Drawable,
        helpers::set_buffer_data,
        vertex::{Vertex, calculate_normals},
    },
    shader::ShaderInfo,
    state::State,
};

#[derive(Debug)]
pub struct CubeSettings<'a> {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub texture_name: &'a str,
    pub shader_name: ShaderInfo,
    pub blend_mode: BlendMode,
}

impl Default for CubeSettings<'_> {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            texture_name: "",
            shader_name: ShaderInfo {
                name: "cube".to_string(),
                fragment_path: "assets/shaders/cube/frag.glsl".to_string(),
                vertex_path: "assets/shaders/cube/vert.glsl".to_string(),
            },
            blend_mode: BlendMode::default(),
        }
    }
}

pub struct Cube {
    pub points: Vec<Vertex>,
    pub position: Vector3<f32>,
    pub vbo: u32,
    pub vao: u32,
    pub texture: String,
    pub shader_info: ShaderInfo,
    pub blend_mode: BlendMode,
}

impl Cube {
    pub fn new(settings: CubeSettings) -> Result<Self> {
        #[rustfmt::skip]
        let mut points = vec![
            // back
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 2
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 2
            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 3
            // front
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 4
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 5
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 5
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7
            // bottom
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 1
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 5
            // top
            Vertex { position: [-0.5, 0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 2
            Vertex { position: [-0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
            Vertex { position: [0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
            Vertex { position: [-0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7
            // left
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 2
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 6
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 0
            Vertex { position: [-0.5, -0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 4
            // right
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, 0.0] }, // 1
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 5
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] }, // 5
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] }, // 3
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0],  normal: [0.0, 0.0, 0.0] }, // 7

        ];
        points.iter_mut().for_each(|v| {
            v.add_pos(&settings.position);
            v.rotate_around(&settings.position, &settings.rotation);
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
            points: vec![],
            vao,
            vbo,
            position: Vector3::from(settings.position),
            shader_info: settings.shader_name,
            blend_mode: settings.blend_mode,
            texture: settings.texture_name.to_owned(),
        })
    }
}

impl Drawable for Cube {
    fn draw(&self, glfw: &glfw::Glfw, state: &State) {
        unsafe {
            // gl::ActiveTexture(gl::TEXTURE0);
            // self.texture.use_texture();
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }
    fn get_texture_name(&self) -> String {
        self.texture.to_string()
    }
    fn get_shader_name(&self) -> ShaderInfo {
        self.shader_info.clone()
    }
    fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
    fn requires_shader(&self) -> bool {
        true
    }
    fn requires_texture(&self) -> bool {
        true
    }
}

impl Drop for Cube {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
