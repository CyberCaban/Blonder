use cgmath::{Matrix4, Point3};

use crate::render::color::Color;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: Color,
}

impl Vertex {
    pub fn add_pos(&mut self, position: &[f32; 3]) {
        self.position[0] += position[0];
        self.position[1] += position[1];
        self.position[2] += position[2];
    }
    pub fn rotate_around(&mut self, pivot: &[f32; 3], angles: &[f32; 3]) {
        let (pitch, yaw, roll) = (angles[0], angles[1], angles[2]);

        let mut x = self.position[0] - pivot[0];
        let mut y = self.position[1] - pivot[1];
        let mut z = self.position[2] - pivot[2];

        if roll != 0.0 {
            let cos_r = roll.cos();
            let sin_r = roll.sin();
            let x_new = x * cos_r - y * sin_r;
            let y_new = x * sin_r + y * cos_r;
            x = x_new;
            y = y_new;
        }

        if pitch != 0.0 {
            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let y_new = y * cos_p - z * sin_p;
            let z_new = y * sin_p + z * cos_p;
            y = y_new;
            z = z_new;
        }

        if yaw != 0.0 {
            let cos_y = yaw.cos();
            let sin_y = yaw.sin();
            let x_new = x * cos_y + z * sin_y;
            let z_new = -x * sin_y + z * cos_y;
            x = x_new;
            z = z_new;
        }

        self.position[0] = x + pivot[0];
        self.position[1] = y + pivot[1];
        self.position[2] = z + pivot[2];
    }
}
