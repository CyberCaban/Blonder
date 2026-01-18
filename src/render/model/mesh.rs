use std::ptr;

use crate::render::{
    blend_mode::BlendMode, drawable::Drawable, helpers::set_buffer_data_with_indices,
    renderer::TextureRef, vertex::Vertex,
};

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<TextureRef>,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<TextureRef>) -> Self {
        let mut mesh = Self {
            indices,
            textures,
            vertices,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };
        mesh.load_mesh();
        mesh
    }
    fn load_mesh(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            gl::GenBuffers(1, &mut self.ebo);
        }
        set_buffer_data_with_indices(self.vao, self.vbo, self.ebo, &self.vertices, &self.indices);
    }
}

impl Drawable for Mesh {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        // TODO: set textures
        let diffuse_number = 1;
        let specular_number = 1;
        for i in 0..self.textures.len() {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + i as u32);
                gl::BindTexture(gl::TEXTURE_2D, self.textures[i].id());
            }
        }
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }
    fn get_blend_mode(&self) -> BlendMode {
        BlendMode::Opaque
    }
    fn get_shader_name(&self) -> Option<crate::shader::ShaderInfo> {
        None
    }
    fn get_texture_config(&self) -> Option<crate::texture::TextureConfig> {
        None
    }
    fn get_texture_name(&self) -> Option<String> {
        None
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
