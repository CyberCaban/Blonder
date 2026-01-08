use std::collections::HashMap;

use cgmath::{Deg, InnerSpace, Matrix4, Rad, Vector3, perspective};
use log::warn;
use num::Zero;
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

    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            drawables: vec![],
            shaders: HashMap::new(),
            textures: HashMap::new(),
            current_shader: None,
            model: Matrix4::zero(),
            view: Matrix4::zero(),
            projection: Matrix4::zero(),
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
    pub fn use_current_shader(&self) -> Result<()> {
        let current_shader = &self.current_shader;
        if let Some(shader_name) = current_shader
            && let Some(shader) = self.shaders.get(shader_name)
        {
            shader.use_shader();
            shader.set_mat4("model", &self.model);
            shader.set_mat4("view", &self.view);
            shader.set_mat4("projection", &self.projection);
            // shader.set_mat4("mvp", &(self.projection * self.view * self.model));
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
    fn update_mvp(&mut self, state: &State) {
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
        self.model = model_matrix;
        self.view = view_matrix;
        self.projection = projection_matrix;
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
            if key.need_shader {
                if let Some(shader) = self.shaders.get(&key.shader_name) {
                    shader.use_shader();
                    self.apply_uniforms(shader, glfw, state);
                    self.current_shader = Some(key.shader_name.clone());
                } else {
                    if let Some(default_shader) = self.shaders.get("default") {
                        default_shader.use_shader();
                        self.current_shader = Some("default".to_string());
                    }
                }
            } else {
                if let Some(default_shader) = self.shaders.get("default") {
                    default_shader.use_shader();
                    self.current_shader = Some("default".to_string());
                }
            }

            if let Some(shader) = self.get_current_shader() {
                self.apply_uniforms(shader, glfw, state);
            }
            if let Some(texture) = self.textures.get(&key.texture_name) {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    texture.use_texture();
                }
            }

            // key.blend_mode.apply();
            for object in objects {
                object.draw(glfw, state);
            }
        }
    }
    fn apply_uniforms(&self, shader: &Shader, glfw: &mut glfw::Glfw, state: &State) {
        shader.set_float("uTime", glfw.get_time() as f32);
        shader.set_mat4("model", &self.model);
        shader.set_mat4("view", &self.view);
        shader.set_mat4("projection", &self.projection);
        shader.set_float("farPlane", 10.0);
        shader.set_vec3("cameraPos", &state.camera.position);
        shader.set_vec3("lightColor", &Vector3::new(0.33, 0.33, 1.));
        shader.set_vec3(
            "lightPos",
            &Vector3::new(
                (glfw.get_time() as f32).sin() * 3.0,
                1.0,
                (glfw.get_time() as f32).cos() * 3.0,
            ),
        );
    }
    pub fn render_checkerboard(&mut self, glfw: &mut glfw::Glfw, state: &State) {
        self.update_mvp(state);
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

            if let Err(e) = self.use_current_shader() {
                warn!("Rendering error: [{e}]");
            }
            if let Some(shader) = self.get_current_shader() {
                // shader.set_int("checkerboardPattern", pattern);
                // shader.set_int("checkerboardFrame", (FRAME_COUNT % 4) as i32);
                self.apply_uniforms(shader, glfw, state);
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
    need_shader: bool,
    // blend_mode: BlendMode,
}

impl BatchKey {
    fn from_object(object: &dyn Drawable) -> Self {
        BatchKey {
            texture_name: object.get_texture_name(),
            shader_name: object.get_shader_name().get_name(),
            need_shader: object.requires_shader(),
            // blend_mode: object.get_blend_mode(),
        }
    }
}
