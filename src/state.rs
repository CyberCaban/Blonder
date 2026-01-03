use std::default;

use glfw::{GlfwReceiver, WindowEvent};
extern crate gl;

#[repr(u32)]
#[derive(Debug, Default)]
enum TextureFiltering {
    #[default]
    Nearest = (gl::NEAREST),
    Bilinear = (gl::LINEAR),
}
#[derive(Debug, Default)]
pub struct State {
    pub color: (f32, f32, f32, f32),
    pub wireframe: bool,
    pub texture_filtering: TextureFiltering,
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;
