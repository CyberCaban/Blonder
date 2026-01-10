use cgmath::{Matrix4, Rad, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn calculate_model(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.position);
        let rot_x = Matrix4::from_angle_x(Rad(self.rotation.x));
        let rot_y = Matrix4::from_angle_y(Rad(self.rotation.y));
        let rot_z = Matrix4::from_angle_z(Rad(self.rotation.z));
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        translation * rot_y * rot_x * rot_z * scale
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
