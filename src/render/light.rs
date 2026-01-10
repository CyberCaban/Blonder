use cgmath::Vector3;

pub struct Light {
    position: Vector3<f32>,
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
}
