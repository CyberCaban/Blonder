use cgmath::{Array, Deg, ElementWise, InnerSpace, Matrix4, Rad, Vector3, perspective};
use log::warn;
use num::Zero;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::render::color::Color;
use crate::render::consts::DEFAULT_FONT;
use crate::render::consts::{HEIGHT, WIDTH};
use crate::render::framebuffer::{Framebuffer, ViewportScaleStrategy};
use crate::render::gui::font::FontAtlas;
use crate::render::gui::ui_manager::UIManager;
use crate::state::Screen;
use crate::texture::TextureConfig;
use crate::{
    render::{drawable::Drawable, transform::Transform},
    shader::Shader,
    state::State,
    texture::Texture,
};
use anyhow::Result;

static FPS_SAMPLES: usize = 100;

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

pub type TextureRef = Arc<Texture>;
pub type FontAtlasRef = Arc<FontAtlas>;

pub struct Renderer {
    drawables: Vec<RenderObject>,
    dynamic_map: HashMap<Uuid, usize>,

    pub textures: HashMap<String, TextureRef>,

    shaders: HashMap<String, Shader>,
    current_shader: Option<String>,

    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>,

    framebuffer: Framebuffer,
    fps_samples: VecDeque<f32>,

    font_atlases: HashMap<String, FontAtlasRef>,

    pub ui_manager: UIManager,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let mut renderer = (Self {
            drawables: vec![],
            dynamic_map: HashMap::new(),
            shaders: HashMap::new(),
            textures: HashMap::new(),
            current_shader: None,
            model: Matrix4::zero(),
            view: Matrix4::zero(),
            projection: Matrix4::zero(),
            framebuffer: Framebuffer::new(
                480,
                360,
                &Screen {
                    width: WIDTH,
                    height: HEIGHT,
                },
                ViewportScaleStrategy::Stretch,
            )?,
            fps_samples: VecDeque::with_capacity(FPS_SAMPLES),
            font_atlases: HashMap::from([(
                DEFAULT_FONT.to_string(),
                Arc::new(FontAtlas::new("assets/fonts/OpenSans.ttf", 96)?),
            )]),
            ui_manager: UIManager::new()?,
        });

        Ok(renderer)
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
        // load texture
        if let Some(texture_name) = object.get_texture_name()
            && !self.textures.contains_key(&texture_name)
        {
            let tex = if let Some(config) = object.get_texture_config() {
                Texture::with_config(&texture_name, config)
            } else {
                Texture::new(&texture_name)
            };
            match tex {
                Ok(texture) => {
                    self.textures
                        .insert(texture_name.clone(), Arc::new(texture));
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        // load shader
        if let Some(shader_name) = object.get_shader_name()
            && !self.shaders.contains_key(&shader_name.get_name())
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
        let render_object = RenderObject {
            drawable: Box::new(object),
            transform: None,
            material: None,
        };
        self.drawables.push(render_object);
        Ok(())
    }
    pub fn add_dynamic_drawable<T: Drawable + 'static>(&mut self, object: T) -> Result<Uuid> {
        // load texture
        if let Some(texture_name) = object.get_texture_name()
            && !self.textures.contains_key(&texture_name)
        {
            let tex = if let Some(config) = object.get_texture_config() {
                Texture::with_config(&texture_name, config)
            } else {
                Texture::new(&texture_name)
            };
            match tex {
                Ok(texture) => {
                    self.textures
                        .insert(texture_name.clone(), Arc::new(texture));
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        // load shader
        if let Some(shader_name) = object.get_shader_name()
            && !self.shaders.contains_key(&shader_name.get_name())
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
        let id = Uuid::new_v4();
        let render_object = RenderObject {
            drawable: Box::new(object),
            transform: Some(Transform::default()),
            material: None,
        };
        self.drawables.push(render_object);
        self.dynamic_map.insert(id, self.drawables.len() - 1);
        Ok(id)
    }

    pub fn add_render_object(&mut self, object: RenderObject) -> Result<Uuid> {
        // load texture
        if let Some(texture_name) = object.drawable.get_texture_name()
            && !self.textures.contains_key(&texture_name)
        {
            let tex = if let Some(config) = object.drawable.get_texture_config() {
                Texture::with_config(&texture_name, config)
            } else {
                Texture::new(&texture_name)
            };
            match tex {
                Ok(texture) => {
                    self.textures
                        .insert(texture_name.clone(), Arc::new(texture));
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
        }
        // load shader
        if let Some(shader_name) = object.drawable.get_shader_name()
            && !self.shaders.contains_key(&shader_name.get_name())
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
                    self.textures
                        .insert(specular_map.clone(), Arc::new(texture));
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
                    self.textures
                        .insert(emission_map.clone(), Arc::new(texture));
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
    pub fn get_or_load_texture(
        &mut self,
        texture_path: &str,
        texture_config: Option<TextureConfig>,
    ) -> TextureRef {
        if let Some(texture) = self.textures.get(texture_path) {
            texture.clone()
        } else {
            let texture = if let Some(config) = texture_config {
                Texture::with_config(texture_path, config)
            } else {
                Texture::new(texture_path)
            };
            match texture {
                Ok(texture) => {
                    self.textures
                        .insert(texture_path.to_owned(), Arc::new(texture));
                }
                Err(e) => {
                    warn!("Failed to load texture: {}", e);
                }
            }
            self.textures.get(texture_path).unwrap().clone()
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
        let mut batches: HashMap<BatchKey, Vec<usize>> = HashMap::new();
        batches.reserve(self.drawables.len() / 4);
        for (index, render_obj) in self.drawables.iter().enumerate() {
            let is_selected = state.selected_item == Some(index);
            let key = BatchKey::from_object(render_obj.drawable.as_ref(), is_selected);

            batches.entry(key).or_default().push(index);
        }

        for (key, objects) in batches {
            let shader = if let Some(shader_name) = &key.shader_name
                && let Some(shader) = self.shaders.get(shader_name.as_ref())
            {
                shader.use_shader();
                self.apply_uniforms(shader, glfw, state);
                self.current_shader = Some(shader_name.to_string());
                Some(shader)
            } else if let Some(default_shader) = self.shaders.get("default") {
                default_shader.use_shader();
                self.current_shader = Some("default".to_string());
                Some(default_shader)
            } else {
                None
            };

            if shader.is_none() {
                continue;
            }
            let shader = shader.unwrap();

            if key.is_selected {
                shader.set_int("isSelected", 1);
                shader.set_vec3("highlightColor", &Vector3::new(0.2, 1.0, 0.2));
                shader.set_float("highlightIntensity", 0.3);
            } else {
                shader.set_int("isSelected", 0);
            }

            if let Some(texture_name) = &key.texture_name
                && let Some(texture) = self.textures.get(texture_name.as_ref())
            {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    texture.use_texture();
                }
            }

            // key.blend_mode.apply();
            for index in &objects {
                let render_obj = &self.drawables[*index];
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

                render_obj.drawable.draw(glfw, state);
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
    pub fn render_checkerboard(&mut self, glfw: &mut glfw::Glfw, state: &mut State) {
        self.update_mvp(state);
        self.framebuffer
            .set_scale_strategy(state.scale_strategy.clone());
        if state.is_lowres {
            self.framebuffer.begin_render();
        }
        if let Err(e) = self.use_current_shader() {
            warn!("Rendering error: [{e}]");
        }
        self.ui_manager.picking_texture.enable_writing();
        self.render_unique(glfw, state);
        self.ui_manager.picking_texture.disable_writing();
        unsafe {
            gl::ClearColor(state.color.0, state.color.1, state.color.2, state.color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            // configurable parameters
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if state.wireframe { gl::LINE } else { gl::FILL },
            );
        }

        if state.is_lowres {
            self.framebuffer.begin_render();
        }
        self.batch_render(glfw, state);
        self.render_ui(glfw, state);
        if state.window_size_changed {
            let _ = self
                .ui_manager
                .picking_texture
                .update_screen_size(state.screen.width as i32, state.screen.height as i32);
            state.window_size_changed = false;
        }
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
        if state.is_lowres {
            self.framebuffer.end_scene_render();
            self.framebuffer.update_screen_size(&state.screen);
        }
        if self.fps_samples.len() >= FPS_SAMPLES {
            self.fps_samples.pop_front();
        }
        self.fps_samples.push_back(1.0 / state.delta_time);
    }
    fn render_ui(&mut self, glfw: &mut glfw::Glfw, state: &mut State) {
        let scale = if state.is_lowres { 0.25 } else { 1.0 };
        let screen = if state.is_lowres {
            Screen {
                width: self.framebuffer.render_width as u32,
                height: self.framebuffer.render_height as u32,
            }
        } else {
            Screen {
                width: state.screen.width,
                height: state.screen.height,
            }
        };

        self.ui_manager.begin_frame(state, &screen);
        // self.ui_manager
        //     .draw_rect(100.0, 100.0, 20.0, 50.0, Color::blue());
        // let tex = self.get_or_load_texture(
        //     "assets/textures/transparency.png",
        //     Some(TextureConfig {
        //         texture_filtering: gl::NEAREST as i32,
        //         ..Default::default()
        //     }),
        // );
        // self.ui_manager.draw_texture(
        //     tex.clone(),
        //     0.0,
        //     0.0,
        //     screen.width as f32 / 10.0,
        //     screen.height as f32 / 10.0,
        //     Color::white(),
        // );

        let current_font = self.font_atlases.get(DEFAULT_FONT).unwrap();
        let font_height = (current_font.size as f32 * scale / 2.0);
        self.ui_manager.draw_text(
            current_font,
            &format!(
                "FPS(sampled:{}): {:.0}",
                self.fps_samples.len(),
                self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32
            ),
            0.0,
            screen.height as f32 - font_height,
            scale,
            Color::white(),
        );

        let (btn_width, btn_height) = (
            screen.width as f32 / 100.0 * 30.0,
            screen.height as f32 / 100.0 * 10.0,
        );
        let (mouse_x, mouse_y) = (
            state.cursor_pos_x * screen.width as f32 / state.screen.width as f32,
            state.cursor_pos_y * screen.height as f32 / state.screen.height as f32,
        );

        if self.ui_manager.button(
            0,
            "Light Up",
            0.0,
            screen.height as f32 / 100.0 * 50.0,
            btn_width,
            btn_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            let light_pos_speed = 8.55;
            state.light_pos.y += 0.1 * state.delta_time * light_pos_speed;
        }

        if self.ui_manager.button(
            0,
            "Light Down",
            0.0,
            screen.height as f32 / 100.0 * 50.0 - btn_height,
            btn_width,
            btn_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            let light_pos_speed = 8.55;
            state.light_pos.y -= 0.1 * state.delta_time * light_pos_speed;
        }
        let screen = if state.is_lowres {
            Screen {
                width: self.framebuffer.render_width as u32,
                height: self.framebuffer.render_height as u32,
            }
        } else {
            Screen {
                width: state.screen.width,
                height: state.screen.height,
            }
        };
        let (mouse_x, mouse_y) = (
            state.cursor_pos_x * screen.width as f32 / state.screen.width as f32,
            state.cursor_pos_y * screen.height as f32 / state.screen.height as f32,
        );
        if state.mouse_free && state.mouse_pressed {
            let p = self
                .ui_manager
                .picking_texture
                .read_pixel(mouse_x as u32, mouse_y as u32);
            if p > 0 {
                let index = p as usize - 1;
                state.selected_item = Some(index);
            } else {
                state.selected_item = None;
            }
        }
        state.mouse_free = true;
        self.ui_manager.end_frame();
    }
    /// render to another framebuffer with object index as object color
    fn render_unique(&mut self, glfw: &mut glfw::Glfw, state: &State) {
        let shader = &self.ui_manager.picking_texture.shader;
        shader.use_shader();
        shader.set_mat4("view", &self.view);
        shader.set_mat4("projection", &self.projection);
        for index in 0..self.drawables.len() {
            let render_obj = &self.drawables[index];
            let object_id = (index + 1) as u32;
            shader.set_uint("objectId", object_id);
            if let Some(transform) = &render_obj.transform {
                let model = transform.calculate_model();
                shader.set_mat4("model", &model);
            } else {
                shader.set_mat4("model", &self.model);
            }

            render_obj.drawable.draw(glfw, state);
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct BatchKey {
    texture_name: Option<Arc<str>>,
    shader_name: Option<Arc<str>>,
    is_selected: bool,
    // blend_mode: BlendMode,
}

impl BatchKey {
    fn from_object(object: &dyn Drawable, is_selected: bool) -> Self {
        BatchKey {
            texture_name: object.get_texture_name().map(|s| Arc::from(s.as_str())),
            shader_name: object
                .get_shader_name()
                .and_then(|s| Some(s.get_name()))
                .map(|s| Arc::from(s.as_str())),
            is_selected,
            // blend_mode: object.get_blend_mode(),
        }
    }
    fn set_shader(&mut self, shader: Option<Arc<str>>) {
        self.shader_name = shader;
    }
}
