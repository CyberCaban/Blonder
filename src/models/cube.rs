use anyhow::Result;
use cgmath::Vector3;
use glfw::Glfw;

use crate::{
    render::{color::Color, drawable::Drawable, helpers::set_buffer_data, vertex::Vertex},
    shader::Shader,
    state::State,
    texture::Texture,
};

#[derive(Debug, Default)]
pub struct CubeSettings<'a> {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub texture_name: &'a str,
}

pub struct Cube {
    pub points: Vec<Vertex>,
    pub position: Vector3<f32>,
    pub vao: u32,
    pub texture: String,
    pub shader: Shader,
}

impl Cube {
    pub fn new(settings: CubeSettings) -> Result<Self> {
        #[rustfmt::skip]
        let mut points = vec![
            // back
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], color: Color::white() }, // 0
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 1.0], color:  Color::white() }, // 2
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 1
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 1
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 1.0], color:  Color::white() }, // 2
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 1.0], color:   Color::white() }, // 3
            // front
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 0.0], color: Color::white() }, // 4
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], color:  Color::white() }, // 5
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 6
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 6
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 0.0], color:  Color::white() }, // 5
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0], color:   Color::white() }, // 7
            // bottom
            Vertex { position: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], color: Color::white() }, // 0
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 1
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 4
            Vertex { position: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 4
            Vertex { position: [0.5, -0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 1
            Vertex { position: [0.5, -0.5, 0.5], uv: [1.0, 1.0], color:   Color::white() }, // 5
            // top
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], color: Color::white() }, // 2
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 6
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 3
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 3
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 6
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0], color:   Color::white() }, // 7
            // left
            Vertex { position: [-0.5, 0.5, -0.5], uv: [0.0, 0.0], color:  Color::white() }, // 2
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0], color: Color::white() }, // 0
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:   Color::white() }, // 6
            Vertex { position: [-0.5, 0.5, 0.5], uv: [0.0, 1.0], color:   Color::white() }, // 6
            Vertex { position: [-0.5, -0.5, -0.5], uv: [1.0, 0.0], color: Color::white() }, // 0
            Vertex { position: [-0.5, -0.5, 0.5], uv: [1.0, 1.0], color:  Color::white() }, // 4
            // right
            Vertex { position: [0.5, -0.5, -0.5], uv: [0.0, 0.0], color: Color::white() }, // 1
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 3
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 5
            Vertex { position: [0.5, -0.5, 0.5], uv: [0.0, 1.0], color:  Color::white() }, // 5
            Vertex { position: [0.5, 0.5, -0.5], uv: [1.0, 0.0], color:  Color::white() }, // 3
            Vertex { position: [0.5, 0.5, 0.5], uv: [1.0, 1.0], color:   Color::white() }, // 7

        ];
        points.iter_mut().for_each(|v| {
            v.add_pos(&settings.position);
            v.rotate_around(&settings.position, &settings.rotation);
        });
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
            vao,
            position: Vector3::from(settings.position),
            shader: Shader::new(
                "assets/shaders/cube/vert.glsl",
                "assets/shaders/cube/frag.glsl",
            )?,
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
            gl::DrawArrays(gl::TRIANGLES, 0, 36 as i32);
        }
    }
    fn get_texture_name(&self) -> String {
        self.texture.to_string()
    }
}
