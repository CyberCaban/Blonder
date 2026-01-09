use crate::render::transform::Transform;
use cgmath::{Matrix4, Rad, Vector3};

pub trait DynamicObject {
    fn get_transform(&self) -> Transform;
    fn set_position(&mut self, position: &[f32; 3]) {
        self.get_transform().position = Vector3::new(position[0], position[1], position[2]);
    }
    fn set_rotation(&mut self, rotation: &[f32; 3]) {
        self.get_transform().rotation = Vector3::new(rotation[0], rotation[1], rotation[2]);
    }
    fn set_scale(&mut self, scale: &[f32; 3]) {
        self.get_transform().scale = Vector3::new(scale[0], scale[1], scale[2]);
    }
    fn get_model(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.get_transform().position);
        let rot_x = Matrix4::from_angle_x(Rad(self.get_transform().rotation.x));
        let rot_y = Matrix4::from_angle_y(Rad(self.get_transform().rotation.y));
        let rot_z = Matrix4::from_angle_z(Rad(self.get_transform().rotation.z));
        let scale = Matrix4::from_nonuniform_scale(
            self.get_transform().scale.x,
            self.get_transform().scale.y,
            self.get_transform().scale.z,
        );

        translation * rot_y * rot_x * rot_z * scale
    }
}
