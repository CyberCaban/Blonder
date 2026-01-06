use crate::{shader::Shader, state::State};


pub trait Drawable {
    fn draw(&self, glfw: &glfw::Glfw, state: &State);
    fn get_texture_name(&self) -> String;
}