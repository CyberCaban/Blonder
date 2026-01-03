use glfw::{GlfwReceiver, WindowEvent};

#[derive(Debug, Default)]
pub struct State {
    pub color: (f32, f32, f32, f32),
    pub wireframe: bool,
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;
