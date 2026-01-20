use cgmath::{Array, Deg, ElementWise, InnerSpace, Matrix4, Rad, Vector3, perspective};
use log::info;
use log::warn;
use num::Zero;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::render::blend_mode::BlendMode;
use crate::render::color::Color;
use crate::render::consts::DEFAULT_BLACK_TEXTURE;
use crate::render::consts::DEFAULT_FONT;
use crate::render::consts::DEFAULT_SHADER_FRAG;
use crate::render::consts::DEFAULT_SHADER_NAME;
use crate::render::consts::DEFAULT_SHADER_VERT;
use crate::render::consts::DEFAULT_WHITE_TEXTURE;
use crate::render::consts::MAX_DIR_LIGHTS;
use crate::render::consts::MAX_POINT_LIGHTS;
use crate::render::consts::MAX_SPOT_LIGHTS;
use crate::render::consts::{HEIGHT, WIDTH};
use crate::render::framebuffer::{Framebuffer, ViewportScaleStrategy};
use crate::render::gui::font::FontAtlas;
use crate::render::gui::ui_manager::UIManager;
use crate::render::light::DirLight;
use crate::render::light::PointLight;
use crate::render::light::SpotLight;
use crate::render::model::Model;
use crate::render::shader::Shader;
use crate::render::shader::pass_uniforms::PassUniforms;
use crate::state::Screen;
use crate::texture::TextureConfig;
use crate::{
    render::{drawable::Drawable, transform::Transform},
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

pub struct RenderObject {
    pub drawable: Box<dyn Drawable>,
    pub transform: Transform,
    pub material: RenderMaterial,
}

impl RenderObject {
    pub fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Shader [{0}] not found")]
    ShaderNotFound(String),
    #[error("Object with id: [{0}] not found")]
    ObjectNotFound(String),
}

pub type TextureRef = Arc<Texture>;
pub type FontAtlasRef = Arc<FontAtlas>;
pub type ShaderRef = Arc<Shader>;

pub struct Renderer {
    drawables: Vec<RenderObject>,
    dynamic_map: HashMap<Uuid, usize>,

    pub textures: HashMap<String, TextureRef>,

    point_lights: VecDeque<PointLight>,
    dir_lights: VecDeque<DirLight>,
    spot_lights: VecDeque<SpotLight>,

    shaders: HashMap<String, ShaderRef>,
    current_shader: Option<ShaderRef>,

    model_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,

    framebuffer: Framebuffer,
    fps_samples: VecDeque<f32>,

    font_atlases: HashMap<String, FontAtlasRef>,

    pub ui_manager: UIManager,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let renderer = Self {
            drawables: vec![],
            dynamic_map: HashMap::new(),
            shaders: HashMap::from([(
                DEFAULT_SHADER_NAME.to_string(),
                Arc::new(Shader::new(DEFAULT_SHADER_VERT, DEFAULT_SHADER_FRAG)?),
            )]),
            textures: HashMap::from([
                (
                    DEFAULT_WHITE_TEXTURE.to_string(),
                    Arc::new(Texture::white()),
                ),
                (
                    DEFAULT_BLACK_TEXTURE.to_string(),
                    Arc::new(Texture::black()),
                ),
            ]),
            point_lights: VecDeque::with_capacity(MAX_POINT_LIGHTS),
            dir_lights: VecDeque::with_capacity(MAX_DIR_LIGHTS),
            spot_lights: VecDeque::with_capacity(MAX_SPOT_LIGHTS),
            current_shader: None,
            model_matrix: Matrix4::zero(),
            view_matrix: Matrix4::zero(),
            projection_matrix: Matrix4::zero(),
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
        };

        Ok(renderer)
    }
    pub fn add_shader(&mut self, name: &str, shader: Shader) {
        self.shaders.insert(name.to_string(), shader.into());
    }
    pub fn add_default_shader(&mut self, shader: Shader) {
        self.shaders.insert("default".to_string(), shader.into());
        let _ = self.use_shader("default");
    }
    pub fn use_shader(&mut self, name: &str) -> Result<()> {
        if let Some(shader) = self.shaders.get(name) {
            shader.use_shader();
            self.current_shader = Some(shader.clone());
            Ok(())
        } else {
            warn!("Shader [{name}] not found");
            Err(RendererError::ShaderNotFound(name.to_string()).into())
        }
    }
    pub fn use_current_shader(&self) -> Result<()> {
        let current_shader = &self.current_shader;
        if let Some(shader) = current_shader {
            shader.use_shader();
            shader.set_mat4("model", &self.model_matrix);
            shader.set_mat4("view", &self.view_matrix);
            shader.set_mat4("projection", &self.projection_matrix);
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
                    warn!("Failed to load texture: {e}");
                }
            }
        }
        // load shader
        if let Some(shader_name) = object.get_shader_name()
            && !self.shaders.contains_key(&shader_name.get_name())
        {
            match Shader::new(&shader_name.vertex_path, &shader_name.fragment_path) {
                Ok(s) => {
                    self.shaders.insert(shader_name.name, s.into());
                }
                Err(e) => {
                    warn!("Failer to load shader: [{e}]");
                }
            }
        }
        let render_object = RenderObject {
            drawable: Box::new(object),
            transform: Transform::default(),
            material: RenderMaterial::default(),
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
                    warn!("Failed to load texture: {e}");
                }
            }
        }
        // load shader
        if let Some(shader_name) = object.get_shader_name()
            && !self.shaders.contains_key(&shader_name.get_name())
        {
            match Shader::new(&shader_name.vertex_path, &shader_name.fragment_path) {
                Ok(s) => {
                    self.shaders.insert(shader_name.name, s.into());
                }
                Err(e) => {
                    warn!("Failer to load shader: [{e}]");
                }
            }
        }
        let id = Uuid::new_v4();
        let render_object = RenderObject {
            drawable: Box::new(object),
            transform: Transform::default(),
            material: RenderMaterial::default(),
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
                    warn!("Failed to load texture: {e}");
                }
            }
        }
        // load shader
        if let Some(shader_name) = object.drawable.get_shader_name()
            && !self.shaders.contains_key(&shader_name.get_name())
        {
            match Shader::new(&shader_name.vertex_path, &shader_name.fragment_path) {
                Ok(s) => {
                    self.shaders.insert(shader_name.name, s.into());
                }
                Err(e) => {
                    warn!("Failer to load shader: [{e}]");
                }
            }
        }
        let material = &object.material;
        // load specular map
        if let Some(specular_map) = &material.specular
            && !self.textures.contains_key(specular_map)
        {
            match Texture::new(specular_map) {
                Ok(texture) => {
                    self.textures
                        .insert(specular_map.clone(), Arc::new(texture));
                }
                Err(e) => {
                    warn!("Failed to load texture: {e}");
                }
            }
        }
        // load emission map
        if let Some(emission_map) = &material.emission
            && !self.textures.contains_key(emission_map)
        {
            match Texture::new(emission_map) {
                Ok(texture) => {
                    self.textures
                        .insert(emission_map.clone(), Arc::new(texture));
                }
                Err(e) => {
                    warn!("Failed to load texture: {e}");
                }
            }
        }
        let id = Uuid::new_v4();
        self.drawables.push(object);
        self.dynamic_map.insert(id, self.drawables.len() - 1);
        Ok(id)
    }
    pub fn add_point_light(&mut self, light: PointLight) {
        if self.point_lights.len() >= MAX_POINT_LIGHTS {
            self.point_lights.pop_front();
        }
        self.point_lights.push_back(light);
    }
    pub fn add_dir_light(&mut self, light: DirLight) {
        if self.dir_lights.len() >= MAX_DIR_LIGHTS {
            self.dir_lights.pop_front();
        }
        self.dir_lights.push_back(light);
    }
    pub fn add_spot_light(&mut self, light: SpotLight) {
        if self.spot_lights.len() >= MAX_SPOT_LIGHTS {
            self.spot_lights.pop_front();
        }
        self.spot_lights.push_back(light);
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
                    warn!("Failed to load texture: {e}");
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
        self.model_matrix = model_matrix;
        self.view_matrix = view_matrix;
        self.projection_matrix = projection_matrix;
    }
    fn batch_render(&mut self, glfw: &mut glfw::Glfw, state: &mut State) {
        let mut batches: HashMap<BatchKey, Vec<usize>> = HashMap::new();
        batches.reserve(self.drawables.len() / 4);
        let mut transparent_objects: Vec<(usize, f32)> = Vec::new();
        for (index, render_obj) in self.drawables.iter().enumerate() {
            let is_selected = state.selected_item == Some(index);
            let key = BatchKey::from_object(render_obj.drawable.as_ref(), is_selected);

            if render_obj.drawable.get_blend_mode() == BlendMode::Opaque || true {
                batches.entry(key).or_default().push(index);
            } else {
                let distance =
                    (state.camera.position - render_obj.transform.get_position()).magnitude();
                transparent_objects.push((index, distance));
            }
        }

        transparent_objects.sort_by(|a, b| (b.1).partial_cmp(&a.1).unwrap());

        if state.display_debug_info {
            state.display_debug_info = false;
            dbg!(&batches, &transparent_objects);
        }
        self.draw_batch(glfw, state, batches);
    }
    fn draw_batch(
        &mut self,
        glfw: &mut glfw::Glfw,
        state: &mut State,
        batches: HashMap<BatchKey, Vec<usize>>,
    ) {
        for (key, objects) in batches {
            self.apply_shaders(&key, glfw, state);
            if self.current_shader.is_none() {
                continue;
            }
            let shader = self.current_shader.as_ref().unwrap();

            self.apply_batch_uniforms(&key, glfw, state);
            self.apply_batch_textures(&key);

            key.blend_mode.apply();
            for index in &objects {
                let render_obj = &self.drawables[*index];
                // set transform
                self.apply_transform(render_obj);

                // set material
                self.apply_material(render_obj, &key);
                render_obj.drawable.draw(glfw, state);
            }
        }
    }
    fn apply_uniforms(&self, shader: &Shader, glfw: &mut glfw::Glfw, state: &State) {
        shader.set_float("uTime", glfw.get_time() as f32);
        shader.set_mat4("view", &self.view_matrix);
        shader.set_mat4("projection", &self.projection_matrix);
        shader.set_float("farPlane", 10.0);
        shader.set_vec3("cameraPos", &state.camera.position);

        shader.set_int("numDirLights", self.dir_lights.len() as i32);
        shader.set_int("numPointLights", self.point_lights.len() as i32);
        shader.set_int("numSpotLights", self.spot_lights.len() as i32);

        for (index, light) in self.point_lights.iter().enumerate() {
            light.pass_uniforms(shader, &format!("pointLights[{}]", index));
        }
        for (index, light) in self.dir_lights.iter().enumerate() {
            light.pass_uniforms(shader, &format!("dirLights[{}]", index));
        }
        for (index, light) in self.spot_lights.iter().enumerate() {
            light.pass_uniforms(shader, &format!("spotLights[{}]", index));
        }
    }
    fn apply_shaders(&mut self, key: &BatchKey, glfw: &mut glfw::Glfw, state: &mut State) -> bool {
        let shader = if let Some(shader_name) = &key.shader_name
            && let Some(shader) = self.shaders.get(shader_name.as_ref())
        {
            shader.use_shader();
            self.apply_uniforms(shader, glfw, state);
            self.current_shader = Some(shader.clone());
            Some(shader)
        } else if let Some(default_shader) = self.shaders.get(DEFAULT_SHADER_NAME) {
            default_shader.use_shader();
            self.apply_uniforms(default_shader, glfw, state);
            self.current_shader = Some(default_shader.clone());
            Some(default_shader)
        } else {
            None
        };
        shader.is_some()
    }
    fn apply_batch_textures(&self, key: &BatchKey) {
        if let Some(texture_name) = &key.texture_name
            && let Some(texture) = self.textures.get(texture_name.as_ref())
        {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                texture.use_texture();
            }
        } else if let Some(default_texture) = self.textures.get(DEFAULT_WHITE_TEXTURE) {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                default_texture.use_texture();
            }
        }
    }
    fn apply_batch_uniforms(&self, key: &BatchKey, glfw: &mut glfw::Glfw, state: &mut State) {
        let shader = self.current_shader.as_ref().unwrap();
        if key.is_selected {
            shader.set_int("isSelected", 1);
            shader.set_vec3("highlightColor", &Vector3::new(0.2, 1.0, 0.2));
            shader.set_float("highlightIntensity", 0.3);
        } else {
            shader.set_int("isSelected", 0);
        }
    }
    fn apply_transform(&self, render_obj: &RenderObject) {
        let transform = &render_obj.transform;
        let model = transform.calculate_model();
        let shader = self.current_shader.as_ref().unwrap();
        shader.set_mat4("model", &model);
    }
    fn apply_material(&self, render_obj: &RenderObject, key: &BatchKey) {
        let shader = self.current_shader.as_ref().unwrap();
        let obj_material = &render_obj.material;
        shader.set_float("material.shininess", obj_material.shininess);

        // set specular
        shader.set_int("material.specular", 1);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1);
        }
        if let Some(spec) = &obj_material.specular
            && let Some(spec_texture) = self.textures.get(spec)
        {
            spec_texture.use_texture();
        }
        // load RO texture as specular texture if one's missing
        else if let Some(texture_name) = &key.texture_name
            && let Some(texture) = self.textures.get(texture_name.as_ref())
        {
            texture.use_texture();
        } else if let Some(default_texture) = self.textures.get(DEFAULT_WHITE_TEXTURE) {
            default_texture.use_texture();
        }

        // set emission
        shader.set_int("material.emission", 2);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE2);
        }
        if let Some(emission) = &obj_material.emission
            && let Some(emission_texture) = self.textures.get(emission)
        {
            emission_texture.use_texture();
        } else {
            // emission texture must be present even if RO doesn't specify it
            let black_emission = self.textures.get(DEFAULT_BLACK_TEXTURE).unwrap();
            black_emission.use_texture();
        }
    }
    pub fn render_checkerboard(&mut self, glfw: &mut glfw::Glfw, state: &mut State) {
        self.update_mvp(state);
        self.framebuffer.set_scale_strategy(state.scale_strategy);
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

            if let Some(_) = &state.model_path_to_load {
                if let Err(e) = self.load_model(state) {
                    warn!("Failed to load model: [{e}]");
                }
            }
        }

        if state.is_lowres {
            self.framebuffer.begin_render();
        }
        self.batch_render(glfw, state);
        if state.show_ui {
            self.render_ui(glfw, state);
        }
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
    fn load_model(&mut self, state: &mut State) -> Result<()> {
        let m = Model::new(&state.model_path_to_load.as_ref().unwrap())?;
        self.add_render_object(RenderObject {
            drawable: Box::new(m),
            transform: Transform::default(),
            material: RenderMaterial::default(),
        })?;
        state.model_path_to_load = None;
        Ok(())
    }
    fn render_ui(&mut self, glfw: &mut glfw::Glfw, state: &mut State) {
        let scale = if state.is_lowres { 0.25 } else { 0.5 };
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

        let (mouse_x, mouse_y) = (
            state.cursor_pos_x * screen.width as f32 / state.screen.width as f32,
            state.cursor_pos_y * screen.height as f32 / state.screen.height as f32,
        );

        let panel_width = screen.width as f32 * 0.3;
        let panel_x = 0.0;
        let panel_height = screen.height as f32;

        if self.ui_manager.panel(
            0,
            panel_x,
            0.0,
            panel_width,
            panel_height,
            Color::new(0.3, 0.3, 0.3, 0.5),
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
        }
        let current_font = self.font_atlases.get(DEFAULT_FONT).unwrap();
        let font_height = current_font.size as f32 * scale / 2.0;
        self.ui_manager.draw_text(
            current_font,
            &format!(
                "FPS: {:.0}",
                self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32
            ),
            0.0,
            screen.height as f32 - font_height,
            scale,
            Color::white(),
        );

        let button_width = panel_width * 0.8 / 3.0;
        let button_height = 40.0;
        let spacing_y = 10.0;
        let spacing_x = 10.0;
        let margin_x = panel_x + panel_width * 0.1;
        let mut current_y = 20.0;

        // bottom row
        if self.ui_manager.button(
            0,
            "-X",
            margin_x,
            current_y,
            button_width,
            button_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            state.selected_item_pos.x -= 0.1 * state.delta_time * 8.55;
        }

        if self.ui_manager.button(
            0,
            "-Z",
            margin_x + button_width + spacing_x,
            current_y,
            button_width,
            button_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            state.selected_item_pos.z -= 0.1 * state.delta_time * 8.55;
        }

        if self.ui_manager.button(
            0,
            "-Y",
            margin_x + (button_width + spacing_x) * 2.0,
            current_y,
            button_width,
            button_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            state.selected_item_pos.y -= 0.1 * state.delta_time * 8.55;
        }
        current_y += button_height + spacing_y;

        // 2nd row
        if self.ui_manager.button(
            0,
            "+X",
            margin_x,
            current_y,
            button_width,
            button_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            state.selected_item_pos.x += 0.1 * state.delta_time * 8.55;
        }

        if self.ui_manager.button(
            0,
            "+Z",
            margin_x + button_width + spacing_x,
            current_y,
            button_width,
            button_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            state.selected_item_pos.z += 0.1 * state.delta_time * 8.55;
        }

        if self.ui_manager.button(
            0,
            "+Y",
            margin_x + (button_width + spacing_x) * 2.0,
            current_y,
            button_width,
            button_height,
            current_font,
            mouse_x,
            mouse_y,
            state.mouse_pressed,
            state.camera.is_captured,
        ) {
            state.mouse_free = false;
            state.selected_item_pos.y += 0.1 * state.delta_time * 8.55;
        }
        current_y += button_height + spacing_y;

        if let Some(i) = state.selected_item
            && let tr = &mut self.drawables[i].transform
        {
            tr.set_position(state.selected_item_pos);
            //     if let Some(light_src) = &self.light_source
            //         && light_src.id == i
            //     {
            //         state.light_pos = state.selected_item_pos;
            //     }
        }

        let slider_width = panel_width * 0.8;
        let slider_height = 20.0;
        // if let Some(value) = self.ui_manager.slider_with_label(
        //     4,
        //     "Light intensity",
        //     margin_x,
        //     current_y,
        //     slider_width,
        //     slider_height,
        //     state.numbers[1],
        //     -2.0,
        //     2.0,
        //     current_font,
        //     mouse_x,
        //     mouse_y,
        //     state.mouse_pressed,
        //     state.camera.is_captured,
        // ) {
        //     state.mouse_free = false;
        //     state.numbers[1] = value;
        // }
        // current_y += slider_height + spacing_y;

        // if let Some(value) = self.ui_manager.slider_with_label(
        //     5,
        //     "Rotation",
        //     margin_x,
        //     current_y,
        //     slider_width,
        //     slider_height,
        //     state.numbers[2],
        //     0.0,
        //     2.0 * PI,
        //     current_font,
        //     mouse_x,
        //     mouse_y,
        //     state.mouse_pressed,
        //     state.camera.is_captured,
        // ) {
        //     state.mouse_free = false;
        //     state.numbers[2] = value;
        // }
        // current_y += slider_height + spacing_y;

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
                let selected_item_pos = self.drawables[index].transform.clone().get_position();
                state.selected_item_pos = selected_item_pos;
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
        shader.set_mat4("view", &self.view_matrix);
        shader.set_mat4("projection", &self.projection_matrix);
        for index in 0..self.drawables.len() {
            let render_obj = &self.drawables[index];
            let object_id = (index + 1) as u32;
            shader.set_uint("objectId", object_id);
            let transform = &render_obj.transform;
            let model = transform.calculate_model();
            shader.set_mat4("model", &model);

            render_obj.drawable.draw(glfw, state);
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct BatchKey {
    texture_name: Option<Arc<str>>,
    shader_name: Option<Arc<str>>,
    is_selected: bool,
    blend_mode: BlendMode,
}

impl BatchKey {
    fn from_object(object: &dyn Drawable, is_selected: bool) -> Self {
        BatchKey {
            texture_name: object.get_texture_name().map(|s| Arc::from(s.as_str())),
            shader_name: object
                .get_shader_name()
                .map(|s| s.get_name())
                .map(|s| Arc::from(s.as_str())),
            is_selected,
            blend_mode: object.get_blend_mode(),
        }
    }
    fn set_shader(&mut self, shader: Option<Arc<str>>) {
        self.shader_name = shader;
    }
}
