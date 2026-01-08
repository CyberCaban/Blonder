use std::collections::HashMap;

use cgmath::{Deg, InnerSpace, Matrix4, Rad, Vector3, perspective};
use log::warn;
use thiserror::Error;

use crate::{
    render::{blend_mode::BlendMode, drawable::Drawable},
    shader::Shader,
    state::State,
    texture::Texture,
};
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
    pub fn add_default_shader(&mut self, shader: Shader) {
        self.shaders.insert("default".to_string(), shader);
        self.use_shader("default");
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
        if let Some(shader_name) = current_shader
            && let Some(shader) = self.shaders.get(shader_name)
        {
            shader.use_shader();
        }
        Ok(())
    }
    pub fn add_drawable<T: Drawable + 'static>(&mut self, object: T) -> Result<()> {
        let texture_name = object.get_texture_name();
        if object.requires_texture() && !self.textures.contains_key(&texture_name) {
            match Texture::new(&texture_name) {
                Ok(texture) => {
                    self.textures.insert(texture_name.clone(), texture);
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        let shader_name = object.get_shader_name();
        if object.requires_shader() && !self.shaders.contains_key(&shader_name.get_name()) {
            match Shader::new(&shader_name.vertex_path, &shader_name.fragment_path) {
                Ok(s) => {
                    self.shaders.insert(shader_name.name, s);
                }
                Err(e) => {
                    warn!("Failer to load shader: [{}]", e);
                }
            }
        }
        self.drawables.push(Box::new(object));
        Ok(())
    }
    fn get_current_shader(&self) -> Option<&Shader> {
        match &self.current_shader {
            Some(name) => self.shaders.get(name),
            None => None,
        }
    }
    fn draw_object(&self, object: &Box<dyn Drawable>, glfw: &mut glfw::Glfw, state: &State) {
        let texture_name = object.get_texture_name();
        if let Some(texture) = self.textures.get(&texture_name) {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture.id());
            }
        }
        object.draw(glfw, state);
    }
    fn prepare_mvp(&self, state: &State) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
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
        (model_matrix, view_matrix, projection_matrix)
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
        let (m, v, p) = self.prepare_mvp(state);
        let mvp = p * v * m;
        if let Err(e) = self.use_current_shader(&mvp) {
            warn!("Rendering error: [{e}]");
        }
        for object in &self.drawables {
            self.draw_object(object, glfw, state);
        }
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }
    fn batch_render(&mut self, glfw: &mut glfw::Glfw, state: &State) {
        let batches = {
            let mut batches: HashMap<BatchKey, Vec<&Box<dyn Drawable>>> = HashMap::new();
            for object in &self.drawables {
                let key = BatchKey::from_object(object.as_ref());
                batches.entry(key).or_default().push(object);
            }
            batches
        };

        for (key, objects) in batches {
            if let Some(texture) = self.textures.get(&key.texture_name) {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    texture.use_texture();
                }
            }
            if let Some(shader) = self.shaders.get(&key.shader_name) {
                // shader.use_shader();
                // shader.set_vec3("lightColor", &Vector3::new(1.0, 1.0, 1.0));
                // shader.set_vec3("lightPos", &Vector3::new(1.5, 2.0, 1.0));
                self.current_shader = Some(key.shader_name.clone());
            }

            // key.blend_mode.apply();
            for object in objects {
                object.draw(glfw, state);
            }
        }
    }
    pub fn render_batch(&mut self, glfw: &mut glfw::Glfw, state: &State) {
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
        let (m, v, p) = self.prepare_mvp(state);
        let mvp = p * v * m;
        if let Err(e) = self.use_current_shader(&mvp) {
            warn!("Rendering error: [{e}]");
        }
        if let Some(shader) = self.get_current_shader() {
            shader.set_vec3("lightColor", &Vector3::new(1.0, 0.0, 0.0));
            shader.set_vec3("lightPos", &Vector3::new(1.5, 2.0, 1.0));
            shader.set_mat4("model", &m);
            shader.set_mat4("view", &v);
            shader.set_mat4("projection", &p);
        }
        self.batch_render(glfw, state);

        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }
    pub fn render_checkerboard(&mut self, glfw: &mut glfw::Glfw, state: &State) {
        let (m, v, p) = self.prepare_mvp(state);
        let mvp = p * v * m;
        let State {
            color, wireframe, ..
        } = state;
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            // configurable parameters
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if *wireframe { gl::LINE } else { gl::FILL },
            );
        }
        unsafe {
            FRAME_COUNT += 1;
            let pattern = match FRAME_COUNT % 4 {
                0 => 0b00,
                1 => 0b01,
                2 => 0b10,
                3 => 0b11,
                _ => 0,
            };

            if let Err(e) = self.use_current_shader(&mvp) {
                warn!("Rendering error: [{e}]");
            }
            if let Some(shader) = self.get_current_shader() {
                // shader.set_int("checkerboardPattern", pattern);
                // shader.set_int("checkerboardFrame", (FRAME_COUNT % 4) as i32);
                shader.set_float("farPlane", 10.0);
                shader.set_vec3("cameraPos", &state.camera.position);
                shader.set_vec3("lightColor", &Vector3::new(0.55, 0.33, 0.6));
                shader.set_vec3(
                    "lightPos",
                    &Vector3::new((glfw.get_time() as f32).sin(), 1.0, 1.0),
                );
                shader.set_mat4("model", &m);
                shader.set_mat4("view", &v);
                shader.set_mat4("projection", &p);
            }
            self.batch_render(glfw, state);
        }
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct BatchKey {
    texture_name: String,
    shader_name: String,
    // blend_mode: BlendMode,
}

impl BatchKey {
    fn from_object(object: &dyn Drawable) -> Self {
        BatchKey {
            texture_name: object.get_texture_name(),
            shader_name: object.get_shader_name().get_name(),
            // blend_mode: object.get_blend_mode(),
        }
    }
}
