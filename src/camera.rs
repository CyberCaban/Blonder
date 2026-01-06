use cgmath::{Angle, InnerSpace, Matrix4, Point3, Rad, Vector3};
use glfw::Action;
use num::Zero;

use crate::render::consts::{HEIGHT, MAX_PITCH_ANGLE, WIDTH};

pub enum MoveDirection {
    FRONT,
    BACK,
    LEFT,
    RIGHT,
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub up: Vector3<f32>,
    pub front: Vector3<f32>,
    pub right: Vector3<f32>,
    pub camera_speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    last_x: f32,
    last_y: f32,
    pub first_mouse: bool,
    pub is_captured: bool,
}

impl Camera {
    pub fn new() -> Self {
        let camera_pos = Vector3::new(0.0, 0.0, 3.0);
        let camera_target = Vector3::new(0.0, 0.0, 0.0);
        let camera_direction = (camera_pos - camera_target).normalize();
        let up = Vector3::new(0.0, 1.0, 0.0);
        let camera_right = up.cross(camera_direction).normalize();
        let camera_up = camera_direction.cross(camera_right);
        let camera_front = Vector3::new(0.0, 0.0, -1.0);
        let mut camera = Camera {
            position: camera_pos,
            up: camera_up,
            front: camera_front,
            right: camera_right,
            camera_speed: 3.55,
            sensitivity: 0.001,
            last_x: WIDTH as f32,
            last_y: HEIGHT as f32,
            first_mouse: true,
            is_captured: false,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        };
        camera.update_vectors();
        camera
    }
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let center = self.position + self.front;
        Matrix4::look_at(
            Point3::new(self.position.x, self.position.y, self.position.z),
            Point3::new(center.x, center.y, center.z),
            self.up,
        )
    }
    pub fn process_mouse_pos(&mut self, x_pos: f32, y_pos: f32) {
        if self.first_mouse {
            self.last_x = x_pos;
            self.last_y = y_pos;
            self.first_mouse = false;
        }
        let mut x_offset = x_pos - self.last_x;
        let mut y_offset = self.last_y - y_pos;
        self.last_x = x_pos;
        self.last_y = y_pos;
        x_offset *= self.sensitivity;
        y_offset *= self.sensitivity;
        self.yaw += x_offset;
        self.pitch += y_offset;
        let angle = MAX_PITCH_ANGLE * self.sensitivity * 10.0;
        if self.pitch > angle {
            self.pitch = angle
        }
        if self.pitch < -angle {
            self.pitch = -angle
        }
        self.update_vectors();
    }
    pub fn process_capture(&mut self, window: &mut glfw::Window) {
        if self.is_captured {
            window.set_cursor_mode(glfw::CursorMode::Normal);
            self.first_mouse = false;
        } else {
            window.set_cursor_mode(glfw::CursorMode::Disabled);
            self.first_mouse = true;
        }
        self.is_captured = !self.is_captured;
    }
    pub fn process_input(&mut self, window: &mut glfw::Window, delta_time: f32) {
        let (x, y) = window.get_cursor_pos();
        if self.is_captured {
            self.process_mouse_pos(x as f32, y as f32);
        }

        let mut move_vector = Vector3::zero();

        if matches!(window.get_key(glfw::Key::W), Action::Press | Action::Repeat) {
            // fly cam
            // move_vector += self.front;
            // fps cam
            move_vector += Vector3::new(self.front.x, 0.0, self.front.z).normalize();
        }
        if matches!(window.get_key(glfw::Key::S), Action::Press | Action::Repeat) {
            // fly cam
            // move_vector -= self.front;
            // fps cam
            move_vector -= Vector3::new(self.front.x, 0.0, self.front.z).normalize();
        }
        if matches!(window.get_key(glfw::Key::A), Action::Press | Action::Repeat) {
            // fly cam
            // move_vector += -self.front.cross(self.up).normalize();
            // fps cam
            move_vector -= self.right;
        }
        if matches!(window.get_key(glfw::Key::D), Action::Press | Action::Repeat) {
            // fly cam
            // move_vector += self.front.cross(self.up).normalize();
            // fps cam
            move_vector += self.right;
        }

        if move_vector.magnitude() > 0.0 {
            move_vector = move_vector.normalize();
            let speed = self.camera_speed * delta_time;
            self.position += move_vector * speed;
        }

        if matches!(
            window.get_key(glfw::Key::Space),
            Action::Press | Action::Repeat
        ) {
            self.position.y += self.camera_speed * delta_time;
        }
        if matches!(
            window.get_key(glfw::Key::LeftShift),
            Action::Press | Action::Repeat
        ) {
            self.position.y -= self.camera_speed * delta_time;
        }
    }
    fn update_vectors(&mut self) {
        let direction = Vector3::new(
            Rad(self.yaw).cos() * Rad(self.pitch).cos(),
            Rad(self.pitch).sin(),
            Rad(self.yaw).sin() * Rad(self.pitch).cos(),
        );
        self.front = direction.normalize();
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        self.right = self.front.cross(world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
