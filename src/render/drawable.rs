use crate::{render::blend_mode::BlendMode, shader::ShaderInfo, state::State};


pub trait Drawable {
    fn draw(&self, glfw: &glfw::Glfw, state: &State);
    fn get_texture_name(&self) -> String;
    fn get_shader_name(&self) -> ShaderInfo;
    fn requires_shader(&self) -> bool;
    fn get_blend_mode(&self) -> BlendMode;
}