use anyhow::Result;
use cgmath::Vector3;

use crate::{
    models::cube::{Cube, CubeSettings},
    render::{
        blend_mode::BlendMode,
        color::Color,
        drawable::Drawable,
        helpers::set_buffer_data,
        shader::ShaderInfo,
        vertex::{Vertex, calculate_normals},
    },
    state::State,
    texture::Texture,
};

#[derive(Debug)]
pub struct CompassPrimitive {
    primitives: Vec<(Cube, Texture)>,
}

impl CompassPrimitive {
    pub fn new() -> Result<Self> {
        let x = (
            Cube::new(CubeSettings {
                scale: [2.0, 0.1, 0.1],
                ..Default::default()
            })?,
            Texture::from_color(Color::red()),
        );

        let y = (
            Cube::new(CubeSettings {
                scale: [0.1, 2.0, 0.1],
                ..Default::default()
            })?,
            Texture::from_color(Color::green()),
        );
        let z = (
            Cube::new(CubeSettings {
                scale: [0.1, 0.1, 2.0],
                ..Default::default()
            })?,
            Texture::from_color(Color::blue()),
        );
        let primitives: Vec<_> = vec![x, y, z];
        Ok(Self { primitives })
    }
}

impl Drawable for CompassPrimitive {
    fn draw(&self, glfw: &glfw::Glfw, state: &State) {
        for (c, t) in &self.primitives {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
            }
            t.use_texture();
            c.draw(glfw, state);
        }
    }
    fn update(&mut self, state: &State) {}
    fn get_texture_name(&self) -> Option<String> {
        None
    }
    fn get_shader_name(&self) -> Option<ShaderInfo> {
        Some(ShaderInfo {
            name: "CompassPrimitive".to_owned(),
            vertex_path: "assets/shaders/color/vert.glsl".to_string(),
            fragment_path: "assets/shaders/color/frag.glsl".to_string(),
        })
    }
    fn get_blend_mode(&self) -> BlendMode {
        BlendMode::Opaque
    }
    fn get_texture_config(&self) -> Option<crate::texture::TextureConfig> {
        None
    }
}
