use std::default;

use cgmath::{Array, Vector3};
use glfw::{GlfwReceiver, WindowEvent};

use crate::{
    camera::Camera,
    render::{
        consts::{HEIGHT, WIDTH},
        framebuffer::postprocessing::ViewportScaleStrategy,
        shader::Shader,
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
pub struct ShaderSettings {
    // vertex snapping
    pub snapping_factor: f32,
    // dithering
    pub dither_intensity: f32,
    // scanlines
    pub scanline_intensity: f32,
    // bloom
    pub bloom_enabled: bool,
    pub bloom_intensity: f32,
    pub bloom_iterations: i32,
    pub bloom_threshold: f32,
    pub exposure: f32,
    // specular
    pub specular_intensity: f32,
}
impl ShaderSettings {
    pub fn apply(&self, shader: &Shader) {
        shader.set_float("snapFactor", self.snapping_factor);
        shader.set_float("ditherIntensity", self.dither_intensity);
        shader.set_float("scanlineIntensity", self.scanline_intensity);
        shader.set_float("specularIntensity", self.specular_intensity);
    }
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug)]
pub struct State {
    pub color: (f32, f32, f32, f32),
    pub show_ui: bool,
    pub shader_settings: ShaderSettings,
    pub wireframe: bool,
    pub is_lowres: bool,
    pub display_debug_info: bool,
    pub scale_strategy: ViewportScaleStrategy,
    pub selected_item: Option<usize>,
    pub selected_item_pos: Vector3<f32>,
    pub selected_item_rot: Vector3<f32>,
    pub model_path_to_load: Option<String>,
    pub mouse_free: bool,
    pub screen: Screen,
    pub camera: Camera,
    pub cursor_pos_x: f32,
    pub cursor_pos_y: f32,
    pub mouse_left_click: bool,
    pub mouse_right_click: bool,
    pub mouse_left_previous: bool,
    pub mouse_right_previous: bool,
    pub delta_time: f32,
    pub last_frame: f32,
    pub window_size_changed: bool,
}

impl State {
    pub fn update_mouse_previous(&mut self) {
        self.mouse_left_previous = self.mouse_left_click;
        self.mouse_right_previous = self.mouse_right_click;
    }

    pub fn mouse_left_just_pressed(&self) -> bool {
        self.mouse_left_click && !self.mouse_left_previous
    }

    pub fn mouse_right_just_pressed(&self) -> bool {
        self.mouse_right_click && !self.mouse_right_previous
    }
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            snapping_factor: 0.0,
            dither_intensity: 0.0,
            scanline_intensity: 0.0,
            bloom_enabled: true,
            bloom_intensity: 0.3,
            bloom_iterations: 3,
            bloom_threshold: 0.8,
            exposure: 1.0,
            specular_intensity: 0.3,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            color: (0.0, 0.0, 0.0, 0.0),
            wireframe: false,
            is_lowres: false,
            scale_strategy: ViewportScaleStrategy::Stretch,
            selected_item: None,
            shader_settings: ShaderSettings::default(),
            selected_item_pos: Vector3::from_value(0.0),
            selected_item_rot: Vector3::from_value(0.0),
            model_path_to_load: None,
            display_debug_info: false,
            mouse_free: false,
            show_ui: false,
            screen: Screen {
                width: WIDTH,
                height: HEIGHT,
            },
            window_size_changed: false,
            camera: Camera::new(),
            cursor_pos_x: 0.0,
            cursor_pos_y: 0.0,
            mouse_left_click: false,
            mouse_right_click: false,
            mouse_left_previous: false,
            mouse_right_previous: false,
            delta_time: 0.0,
            last_frame: 0.0,
        }
    }
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;
