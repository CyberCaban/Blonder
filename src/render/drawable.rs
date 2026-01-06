use crate::state::State;


pub trait Drawable {
    fn draw(&self, glfw: &glfw::Glfw, state: &State);
}