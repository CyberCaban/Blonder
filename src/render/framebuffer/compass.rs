use anyhow::Result;
use cgmath::Matrix4;

use crate::{
    models::compass_primitive::CompassPrimitive,
    render::{
        drawable::Drawable,
        framebuffer::{self, Framebuffer},
        renderer::ShaderRef,
        shader::Shader,
    },
    state::{Screen, State},
};

#[derive(Debug)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct Compass {
    framebuffer: Framebuffer,
    viewport: Viewport,
    screen: Screen,
    screen_shader: Shader,
    pub primitive: CompassPrimitive,
    pub primitive_shader: Shader,
}

impl Compass {
    pub fn new(viewport: Viewport, screen: &Screen) -> Result<Self> {
        let mut framebuffer = Framebuffer::new(300, 300)?;
        framebuffer.add_color_attachment(
            0,
            crate::texture::TextureFormatColor::RGBA8,
            crate::texture::TextureFilter::Linear,
            crate::texture::TextureWrap::ClampToEdge,
        )?;
        framebuffer.add_depth_attachment(
            crate::texture::TextureFormatDepth::Depth16,
            crate::texture::TextureFilter::Linear,
            crate::texture::TextureWrap::ClampToEdge,
        )?;
        framebuffer.clear_color = (0.0, 0.0, 0.0, 0.0);
        framebuffer.check_complete()?;

        let screen_shader = Shader::new(
            "assets/shaders/screen/vert.glsl",
            "assets/shaders/screen/frag.glsl",
        )?;

        let primitive = CompassPrimitive::new()?;
        let primitive_shader = Shader::new(
            "assets/shaders/color/vert.glsl",
            "assets/shaders/color/frag.glsl",
        )?;
        Ok(Self {
            framebuffer,
            viewport,
            screen_shader,
            screen: screen.clone(),
            primitive,
            primitive_shader,
        })
    }
    pub fn begin_render(&self) {
        self.framebuffer.begin_render();
    }
    pub fn end_scene_render(&self) {
        self.framebuffer.unbind();
    }
    pub fn render_scene_to_screen(&self, state: &State) {
        let Viewport { width, height, .. } = self.viewport;
        let x = self.screen.width as i32 / 2 - width / 2;
        let y = self.screen.height as i32 - height;
        self.framebuffer
            .render_to_screen(state, &self.screen_shader, Some((x, y, width, height)));
    }
    pub fn update_screen_size(&mut self, screen: &Screen) {
        self.screen = Screen {
            width: screen.width,
            height: screen.height,
        };
    }
}
