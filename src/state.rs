
use cgmath::{Matrix4, SquareMatrix, Vector3};
use glfw::{GlfwReceiver, WindowEvent};
extern crate gl;

#[repr(u32)]
#[derive(Debug, Default)]
enum TextureFiltering {
    #[default]
    Nearest = (gl::NEAREST),
    Bilinear = (gl::LINEAR),
}
#[derive(Debug)]
pub struct State {
    pub color: (f32, f32, f32, f32),
    pub wireframe: bool,
    pub transform_matrix: Matrix4<f32>,
}

impl Default for State {
    fn default() -> Self {
        let transform_matrix =
            Matrix4::<f32>::identity() * Matrix4::<f32>::from_translation(Vector3::unit_y() * 0.5);
        Self {
            color: (0.0, 0.0, 0.0, 0.0),
            wireframe: false,
            transform_matrix,
        }
    }
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;
