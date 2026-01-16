use crate::{render::blend_mode::BlendMode, shader::ShaderInfo, state::State, texture::TextureConfig};


pub trait Drawable {
    fn draw(&self, glfw: &glfw::Glfw, state: &State);
    fn get_texture_name(&self) -> Option<String>;
    fn get_shader_name(&self) -> Option<ShaderInfo>;
    fn get_texture_config(&self) -> Option<TextureConfig>;
    fn get_blend_mode(&self) -> BlendMode;
}