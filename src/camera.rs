use cgmath::{InnerSpace, Matrix4, Point3, Vector3};
use glfw::Action;

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
    pub camera_speed: f32,
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
        Self {
            position: camera_pos,
            up: camera_up,
            front: camera_front,
            camera_speed: 2.55,
        }
    }
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let center = self.position + self.front;
        Matrix4::look_at(
            Point3::new(self.position.x, self.position.y, self.position.z),
            Point3::new(center.x, center.y, center.z),
            self.up,
        )
    }
    pub fn update_pos(&mut self, direction: MoveDirection, delta_time: f32) {
        let speed = self.camera_speed * delta_time;
        let delta = match direction {
            MoveDirection::FRONT => speed * self.front,
            MoveDirection::BACK => -speed * self.front,
            MoveDirection::LEFT => -self.front.cross(self.up).normalize() * speed,
            MoveDirection::RIGHT => self.front.cross(self.up).normalize() * speed,
        };
        self.position += delta;
    }
    pub fn process_input(&mut self, window: &mut glfw::Window, delta_time: f32) {
        match window.get_key(glfw::Key::W) {
            Action::Press | Action::Repeat => self.update_pos(MoveDirection::FRONT, delta_time),
            _ => {}
        }
        match window.get_key(glfw::Key::S) {
            Action::Press | Action::Repeat => self.update_pos(MoveDirection::BACK, delta_time),
            _ => {}
        }
        match window.get_key(glfw::Key::A) {
            Action::Press | Action::Repeat => self.update_pos(MoveDirection::LEFT, delta_time),
            _ => {}
        }
        match window.get_key(glfw::Key::D) {
            Action::Press | Action::Repeat => self.update_pos(MoveDirection::RIGHT, delta_time),
            _ => {}
        }
    }
}
