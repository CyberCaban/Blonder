use glfw::{GlfwReceiver, WindowEvent};

use crate::{
    camera::Camera,
    render::consts::{HEIGHT, WIDTH},
};

extern crate gl;

#[repr(u32)]
#[derive(Debug, Default)]
enum TextureFiltering {
    #[default]
    Nearest = (gl::NEAREST),
    Bilinear = (gl::LINEAR),
}

#[derive(Debug)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug)]
pub struct State {
    pub color: (f32, f32, f32, f32),
    pub numbers: [f32; 10],
    pub wireframe: bool,
    pub screen: Screen,
    pub camera: Camera,
    pub delta_time: f32,
    pub last_frame: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            color: (0.0, 0.0, 0.0, 0.0),
            wireframe: false,
            numbers: [0.0; 10],
            screen: Screen {
                width: WIDTH,
                height: HEIGHT,
            },
            camera: Camera::new(),
            delta_time: 0.0,
            last_frame: 0.0,
        }
    }
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;
