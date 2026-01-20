use crate::render::shader::Shader;

pub trait PassUniforms {
    fn pass_uniforms(&self, shader: &Shader, prefix: &str);
}
