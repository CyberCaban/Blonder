use cgmath::{Array, Vector3};

pub struct Material {
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambient: Vector3::from_value(1.0),
            diffuse: Vector3::from_value(1.0),
            specular: Vector3::from_value(1.0),
            shininess: 32.0,
        }
    }
}
