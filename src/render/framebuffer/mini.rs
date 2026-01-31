use std::sync::Arc;

use anyhow::Result;

use crate::{
    render::{framebuffer::Framebuffer, renderer::ShaderRef, shader::Shader},
    state::State,
};

#[derive(Debug)]
pub struct Mini {
    framebuffer: Framebuffer,
    screen_shader: Shader,
    pub normal_shader: ShaderRef,
    width: i32,
    height: i32,
}

impl Mini {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let mut framebuffer = Framebuffer::new(width, height)?;
        framebuffer.add_color_attachment(
            0,
            crate::texture::TextureFormatColor::RGBA8,
            crate::texture::TextureFilter::Nearest,
            crate::texture::TextureWrap::ClampToEdge,
        )?;
        framebuffer.add_depth_attachment(
            crate::texture::TextureFormatDepth::Depth24Stencil8,
            crate::texture::TextureFilter::Nearest,
            crate::texture::TextureWrap::ClampToEdge,
        )?;
        framebuffer.clear_color = (0.0, 0.0, 0.0, 0.4);
        framebuffer.check_complete()?;
        let screen_shader = Shader::new(
            "assets/shaders/screen/vert.glsl",
            "assets/shaders/screen/frag.glsl",
        )?;
        let normal_shader = Arc::new(Shader::new(
            "assets/shaders/filters/normals/vert.glsl",
            "assets/shaders/filters/normals/frag.glsl",
        )?);
        // let normal_shader = Arc::new(Shader::new(
        //     "assets/shaders/filters/depth/vert.glsl",
        //     "assets/shaders/filters/depth/frag.glsl",
        // )?);

        Ok(Self {
            framebuffer,
            screen_shader,
            normal_shader,
            width,
            height,
        })
    }
    pub fn begin_render(&self) {
        self.framebuffer.begin_render();
    }
    pub fn end_scene_render(&self) {
        self.framebuffer.unbind();
    }
    pub fn render_scene_to_screen(&self, state: &State) {
        let (x, y, width, height) = self.calculate_screen_position(state);

        self.framebuffer
            .render_to_screen(state, &self.screen_shader, Some((x, y, width, height)));
    }
    fn calculate_screen_position(&self, state: &State) -> (i32, i32, i32, i32) {
        let (pos_x, pos_y) = (30, 30);
        let (width, height) = (300, 300);

        let x = pos_x.min(state.screen.width as i32 - width);
        let y = pos_y.min(state.screen.height as i32 - height);

        (x, y, width, height)
    }
}
