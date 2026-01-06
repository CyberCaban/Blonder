use std::collections::HashMap;

use cgmath::{Deg, InnerSpace, Matrix4, Rad, Vector3, perspective};
use log::warn;
use thiserror::Error;

use crate::{render::drawable::Drawable, shader::Shader, state::State, texture::Texture};
use anyhow::Result;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Shader [{0}] not found")]
    ShaderNotFound(String),
}

pub struct Renderer {
    drawables: Vec<Box<dyn Drawable>>,

    textures: HashMap<String, Texture>,

    shaders: HashMap<String, Shader>,
    current_shader: Option<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            drawables: vec![],
            shaders: HashMap::new(),
            textures: HashMap::new(),
            current_shader: None,
        }
    }
    pub fn add_shader(&mut self, name: &str, shader: Shader) {
        self.shaders.insert(name.to_string(), shader);
    }
    pub fn use_shader(&mut self, name: &str) -> Result<()> {
        if let Some(shader) = self.shaders.get(name) {
            shader.use_shader();
            self.current_shader = Some(name.to_string());
            Ok(())
        } else {
            warn!("Shader [{}] not found", name);
            Err(RendererError::ShaderNotFound(name.to_string()).into())
        }
    }
    pub fn use_current_shader(&mut self, mvp: &Matrix4<f32>) -> Result<()> {
        let current_shader = &self.current_shader;
        if let Some(shader_name) = current_shader {
            if let Some(shader) = self.shaders.get(shader_name) {
                shader.use_shader();
                shader.set_mat4("mvp", mvp);
            }
        }
        Ok(())
    }
    pub fn add_drawable<T: Drawable + 'static>(&mut self, object: T) -> Result<()> {
        let texture_name = object.get_texture_name();
        if !self.textures.contains_key(&texture_name) {
            match Texture::new(&texture_name) {
                Ok(texture) => {
                    self.textures.insert(texture_name.clone(), texture);
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        self.drawables.push(Box::new(object));
        Ok(())
    }
    fn draw_object(&self, object: &Box<dyn Drawable>, glfw: &mut glfw::Glfw, state: &State) {
        let texture_name = object.get_texture_name();
        if let Some(texture) = self.textures.get(&texture_name){
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture.id());
            }
        }
        object.draw(glfw, state);
    }
    pub fn render(&mut self, glfw: &mut glfw::Glfw, state: &State) {
        let State {
            color, wireframe, ..
        } = state;
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            // configurable parameters
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if *wireframe { gl::LINE } else { gl::FILL },
            );
        }
        let aspect = if state.screen.height > 0 {
            state.screen.width as f32 / state.screen.height as f32
        } else {
            1.0
        };
        let model_matrix =
            Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 0.0).normalize(), Rad(0.0));
        let view_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, -3.0));
        let projection_matrix = perspective(Deg(45.0), aspect, 0.01, 100.0);

        let view_matrix = state.camera.view_matrix();
        let mvp = projection_matrix * view_matrix * model_matrix;
        if let Err(e) = self.use_current_shader(&mvp) {
            warn!("Rendering error: [{e}]");
        }
        for object in &self.drawables {
            self.draw_object(object, glfw, &state);
        }
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }
}
