use cgmath::Vector3;

use crate::render::shader::pass_uniforms::PassUniforms;

pub enum LightType {
    Point(PointLight),
    Dir(DirLight),
    Spot(SpotLight),
}

impl PassUniforms for LightType {
    fn pass_uniforms(&self, shader: &super::shader::Shader, prefix: &str) {
        match self {
            LightType::Point(point_light) => point_light.pass_uniforms(shader, prefix),
            LightType::Dir(dir_light) => dir_light.pass_uniforms(shader, prefix),
            LightType::Spot(spot_light) => spot_light.pass_uniforms(shader, prefix),
        }
    }
}

pub struct PointLight {
    pub position: Vector3<f32>,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl PassUniforms for PointLight {
    fn pass_uniforms(&self, shader: &super::shader::Shader, prefix: &str) {
        shader.set_vec3(&format!("{}.position", prefix), &self.position);
        shader.set_float(&format!("{}.constant", prefix), self.constant);
        shader.set_float(&format!("{}.linear", prefix), self.linear);
        shader.set_float(&format!("{}.quadratic", prefix), self.quadratic);
        shader.set_vec3(&format!("{}.ambient", prefix), &self.ambient);
        shader.set_vec3(&format!("{}.diffuse", prefix), &self.diffuse);
        shader.set_vec3(&format!("{}.specular", prefix), &self.specular);
    }
}

pub struct DirLight {
    pub direction: Vector3<f32>,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl PassUniforms for DirLight {
    fn pass_uniforms(&self, shader: &super::shader::Shader, prefix: &str) {
        shader.set_vec3(&format!("{}.direction", prefix), &self.direction);
        shader.set_vec3(&format!("{}.ambient", prefix), &self.ambient);
        shader.set_vec3(&format!("{}.diffuse", prefix), &self.diffuse);
        shader.set_vec3(&format!("{}.specular", prefix), &self.specular);
    }
}

pub struct SpotLight {
    pub direction: Vector3<f32>,
    pub cut_off: f32,
    pub cut_off_outer: f32,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl PassUniforms for SpotLight {
    fn pass_uniforms(&self, shader: &super::shader::Shader, prefix: &str) {
        shader.set_vec3(&format!("{}.direction", prefix), &self.direction);
        shader.set_float(&format!("{}.cut_off", prefix), self.cut_off);
        shader.set_float(&format!("{}.cut_off_outer", prefix), self.cut_off_outer);
        shader.set_float(&format!("{}.constant", prefix), self.constant);
        shader.set_float(&format!("{}.linear", prefix), self.linear);
        shader.set_float(&format!("{}.quadratic", prefix), self.quadratic);
        shader.set_vec3(&format!("{}.ambient", prefix), &self.ambient);
        shader.set_vec3(&format!("{}.diffuse", prefix), &self.diffuse);
        shader.set_vec3(&format!("{}.specular", prefix), &self.specular);
    }
}
