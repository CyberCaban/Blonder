use std::{collections::HashMap, ops::Mul};

use cgmath::{Array, Deg, ElementWise, InnerSpace, Matrix4, Rad, Vector3, perspective};
use log::warn;
use num::Zero;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    render::{
        drawable::Drawable,
        material::{self, Material},
        transform::Transform,
    },
    shader::Shader,
    state::State,
    texture::Texture,
};
use anyhow::Result;

pub struct RenderMaterial {
    pub specular: Option<String>, // path to specular texture
    pub emission: Option<String>, // path to emission texture
    pub shininess: f32,
}
impl Default for RenderMaterial {
    fn default() -> Self {
        Self {
            specular: None,
            emission: None,
            shininess: 32.0,
        }
    }
}
// impl From<Material> for RenderMaterial {
//     fn from(value: Material) -> Self {
//         let specular = match value.get_specular() {
//             Some(texture_path) => {
//                 Texture::new(texture_path)
//             }
//         }
//         Self {
//             specular,
//             shininess: value.get_shininess(),
//         }
//     }
// }

pub struct RenderObject {
    pub drawable: Box<dyn Drawable>,
    pub transform: Option<Transform>,
    pub material: Option<RenderMaterial>,
    pub is_dynamic: bool,
}

impl RenderObject {
    pub fn get_transform_mut(&mut self) -> Option<&mut Transform> {
        self.transform.as_mut()
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Shader [{0}] not found")]
    ShaderNotFound(String),
}

pub struct Renderer {
    drawables: Vec<RenderObject>,
    dynamic_map: HashMap<Uuid, usize>,

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
            dynamic_map: HashMap::new(),
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
        let _ = self.use_shader("default");
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
    pub fn add_static_drawable<T: Drawable + 'static>(&mut self, object: T) -> Result<()> {
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
        let render_object = RenderObject {
            drawable: Box::new(object),
            transform: None,
            material: None,
            is_dynamic: false,
        };
        self.drawables.push(render_object);
        Ok(())
    }
    pub fn add_dynamic_drawable<T: Drawable + 'static>(&mut self, object: T) -> Result<Uuid> {
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
        let id = Uuid::new_v4();
        let render_object = RenderObject {
            drawable: Box::new(object),
            transform: Some(Transform::default()),
            material: None,
            is_dynamic: true,
        };
        self.drawables.push(render_object);
        self.dynamic_map.insert(id, self.drawables.len() - 1);
        Ok(id)
    }

    pub fn add_render_object(&mut self, object: RenderObject) -> Result<Uuid> {
        // load texture
        let texture_name = object.drawable.get_texture_name();
        if object.drawable.requires_texture() && !self.textures.contains_key(&texture_name) {
            match Texture::new(&texture_name) {
                Ok(texture) => {
                    self.textures.insert(texture_name.clone(), texture);
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        // load shader
        let shader_name = object.drawable.get_shader_name();
        if object.drawable.requires_shader() && !self.shaders.contains_key(&shader_name.get_name())
        {
            match Shader::new(&shader_name.vertex_path, &shader_name.fragment_path) {
                Ok(s) => {
                    self.shaders.insert(shader_name.name, s);
                }
                Err(e) => {
                    warn!("Failer to load shader: [{}]", e);
                }
            }
        }
        // load specular map
        if let Some(material) = &object.material
            && let Some(specular_map) = &material.specular
            && !self.textures.contains_key(specular_map)
        {
            match Texture::new(specular_map) {
                Ok(texture) => {
                    self.textures.insert(specular_map.clone(), texture);
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        // load emission map
        if let Some(material) = &object.material
            && let Some(emission_map) = &material.emission
            && !self.textures.contains_key(emission_map)
        {
            match Texture::new(emission_map) {
                Ok(texture) => {
                    self.textures.insert(emission_map.clone(), texture);
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        let id = Uuid::new_v4();
        self.drawables.push(object);
        self.dynamic_map.insert(id, self.drawables.len() - 1);
        Ok(id)
    }
    pub fn get_transform(&self, id: &Uuid) -> Option<&RenderObject> {
        self.dynamic_map
            .get(id)
            .map(|index| &self.drawables[*index])
    }
    pub fn get_transform_mut(&mut self, id: &Uuid) -> Option<&mut RenderObject> {
        self.dynamic_map
            .get(id)
            .map(|index| &mut self.drawables[*index])
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
        let projection_matrix = perspective(Deg(45.0), aspect, 0.01, 100.0);

        let view_matrix = state.camera.view_matrix();
        self.model = model_matrix;
        self.view = view_matrix;
        self.projection = projection_matrix;
    }
    fn batch_render(&mut self, glfw: &mut glfw::Glfw, state: &State) {
        let batches = {
            let mut batches: HashMap<BatchKey, Vec<usize>> = HashMap::new();
            for (index, render_obj) in self.drawables.iter().enumerate() {
                let key = BatchKey::from_object(render_obj.drawable.as_ref());

                batches.entry(key).or_default().push(index);
            }
            batches
        };

        for (key, objects) in batches {
            if key.need_shader
                && let Some(shader) = self.shaders.get(&key.shader_name)
            {
                shader.use_shader();
                self.apply_uniforms(shader, glfw, state);
                self.current_shader = Some(key.shader_name.clone());
            } else if let Some(default_shader) = self.shaders.get("default") {
                default_shader.use_shader();
                self.current_shader = Some("default".to_string());
            }
            // if key.need_shader {
            //     if let Some(shader) = self.shaders.get(&key.shader_name) {
            //         shader.use_shader();
            //         self.apply_uniforms(shader, glfw, state);
            //         self.current_shader = Some(key.shader_name.clone());
            //     } else {
            //         if let Some(default_shader) = self.shaders.get("default") {
            //             default_shader.use_shader();
            //             self.current_shader = Some("default".to_string());
            //         }
            //     }
            // } else {
            //     if let Some(default_shader) = self.shaders.get("default") {
            //         default_shader.use_shader();
            //         self.current_shader = Some("default".to_string());
            //     }
            // }

            if let Some(texture) = self.textures.get(&key.texture_name) {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    texture.use_texture();
                }
            }

            // key.blend_mode.apply();
            for index in objects {
                if let Some(render_obj) = self.drawables.get(index) {
                    // get shader
                    if let Some(shader) = self.get_current_shader() {
                        self.apply_uniforms(shader, glfw, state);
                        // set transform
                        if let Some(transform) = &render_obj.transform {
                            let model = transform.calculate_model();
                            shader.set_mat4("model", &model);
                        } else {
                            shader.set_mat4("model", &self.model);
                        }

                        // set material
                        let obj_material = if let Some(material) = &render_obj.material {
                            material
                        } else {
                            &RenderMaterial::default()
                        };
                        shader.set_float("material.shininess", obj_material.shininess);
                        shader.set_vec3(
                            "lightPos",
                            // &Vector3::new(1.0, 0.0, 0.0),
                            &state.light_pos,
                            // &Vector3::new(
                            //     (glfw.get_time() as f32).sin() * 3.0,
                            //     1.0,
                            //     (glfw.get_time() as f32).cos() * 3.0,
                            // ),
                        );

                        // set specular
                        if let Some(render_mat) = &render_obj.material {
                            if let Some(spec) = &render_mat.specular
                                && let Some(spec_texture) = self.textures.get(spec)
                            {
                                shader.set_int("material.specular", 1);
                                unsafe {
                                    gl::ActiveTexture(gl::TEXTURE1);
                                    spec_texture.use_texture();
                                }
                            }

                            if let Some(emission) = &render_mat.emission
                                && let Some(emission_texture) = self.textures.get(emission)
                            {
                                shader.set_int("material.emission", 2);
                                unsafe {
                                    gl::ActiveTexture(gl::TEXTURE2);
                                    emission_texture.use_texture();
                                }
                            }
                        }

                        let light_color = Vector3::new(1.0, 1.0, 1.0);
                        let diffuse_color = light_color.mul_element_wise(Vector3::from_value(0.5));
                        let ambient_color =
                            diffuse_color.mul_element_wise(Vector3::from_value(state.numbers[1]));
                        shader.set_vec3("light.ambient", &ambient_color);
                        shader.set_vec3("light.diffuse", &diffuse_color);
                        shader.set_vec3("light.specular", &Vector3::from_value(1.0));
                    }
                    render_obj.drawable.draw(glfw, state);
                }
            }
        }
    }
    fn apply_uniforms(&self, shader: &Shader, glfw: &mut glfw::Glfw, state: &State) {
        shader.set_float("uTime", glfw.get_time() as f32);
        // shader.set_mat4("model", &self.model);
        shader.set_mat4("view", &self.view);
        shader.set_mat4("projection", &self.projection);
        shader.set_float("farPlane", 10.0);
        shader.set_vec3("cameraPos", &state.camera.position);
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
                // self.apply_uniforms(shader, glfw, state);
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
