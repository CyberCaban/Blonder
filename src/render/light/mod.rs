use cgmath::Vector3;

use crate::render::shader::pass_uniforms::PassUniforms;

pub struct PointLight {
    position: Vector3<f32>,

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
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
    direction: Vector3<f32>,

    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
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
    direction: Vector3<f32>,
    cut_off: f32,
    cut_off_outer: f32,

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
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
