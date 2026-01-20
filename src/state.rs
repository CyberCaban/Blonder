use cgmath::{Array, Vector3};
use glfw::{GlfwReceiver, WindowEvent};

use crate::{
    camera::Camera,
    render::{
        consts::{HEIGHT, WIDTH},
        framebuffer::ViewportScaleStrategy,
    },
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
    pub show_ui: bool,
    pub light_pos: Vector3<f32>,
    pub wireframe: bool,
    pub is_lowres: bool,
    pub scale_strategy: ViewportScaleStrategy,
    pub selected_item: Option<usize>,
    pub selected_item_pos: Vector3<f32>,
    pub model_path_to_load: Option<String>,
    pub mouse_free: bool,
    pub screen: Screen,
    pub camera: Camera,
    pub cursor_pos_x: f32,
    pub cursor_pos_y: f32,
    pub mouse_pressed: bool,
    pub delta_time: f32,
    pub last_frame: f32,
    pub window_size_changed: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            color: (0.0, 0.0, 0.0, 0.0),
            wireframe: false,
            is_lowres: false,
            scale_strategy: ViewportScaleStrategy::Stretch,
            selected_item: None,
            selected_item_pos: Vector3::from_value(0.0),
            model_path_to_load: None,
            mouse_free: false,
            show_ui: false,
            light_pos: Vector3::from_value(0.0),
            screen: Screen {
                width: WIDTH,
                height: HEIGHT,
            },
            window_size_changed: false,
            camera: Camera::new(),
            cursor_pos_x: 0.0,
            cursor_pos_y: 0.0,
            mouse_pressed: false,
            delta_time: 0.0,
            last_frame: 0.0,
        }
    }
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;
