use crate::{
    render::{
        framebuffer::{bloom::Bloom, Framebuffer},
        shader::Shader,
    },
    state::{Screen, State},
    texture::{Texture, TextureFilter, TextureFormatColor, TextureFormatDepth, TextureWrap},
};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum ViewportScaleStrategy {
    Fit,
    Stretch,
    PixelPerfect,
}

#[derive(Debug)]
pub struct PostprocessingFramebuffer {
    framebuffer: Framebuffer,
    pub render_width: i32,
    pub render_height: i32,
    screen_size: (u32, u32),
    screen_shader: Shader,
    pub bloom: Bloom,
}

impl PostprocessingFramebuffer {
    pub fn new(width: i32, height: i32, screen: &Screen) -> Result<Self> {
        let mut framebuffer = Framebuffer::new(width, height)?;
        // Use HDR format for proper bloom (floating point)
        framebuffer.add_color_attachment(
            0,
            TextureFormatColor::RGBA16F,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
        )?;
        framebuffer.add_depth_attachment(
            TextureFormatDepth::Depth24Stencil8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
        )?;
        framebuffer.clear_color = (0.1, 0.1, 0.1, 0.0);
        framebuffer.use_depth_test = true;
        framebuffer.check_complete()?;

        let screen_shader = Shader::new(
            "assets/shaders/screen/vert.glsl",
            "assets/shaders/screen/frag.glsl",
        )?;

        let bloom = Bloom::new(width, height)?;

        Ok(Self {
            framebuffer,
            render_width: width,
            screen_shader,
            render_height: height,
            screen_size: (screen.width, screen.height),
            bloom,
        })
    }
    pub fn update_screen_size(&mut self, screen: &Screen) {
        self.screen_size = (screen.width, screen.height)
    }

    pub fn set_render_size(&mut self, width: u32, height: u32) -> Result<()> {
        self.render_width = width as i32;
        self.render_height = height as i32;
        self.framebuffer.resize(width as i32, height as i32)?;
        self.bloom.resize(width as i32, height as i32)?;
        Ok(())
    }
    pub fn begin_render(&self) {
        self.framebuffer.begin_render();
    }
    pub fn end_scene_render(&self, state: &State) {
        self.framebuffer.unbind();
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Apply bloom post-processing
            self.bloom.process(&self.framebuffer, state);

            self.render_scene_to_screen(state);
        }
    }
    fn render_scene_to_screen(&self, state: &State) {
        let (viewport_width, viewport_height, offset_x, offset_y) =
            self.calculate_viewport_params();

        if self.bloom.enabled {
            // Use bloom combine shader for HDR + bloom
            self.bloom.render_final(
                &self.framebuffer,
                Some((offset_x, offset_y, viewport_width, viewport_height)),
            );
        } else {
            // Use simple screen shader when bloom is disabled
            self.framebuffer.render_to_screen(
                state,
                &self.screen_shader,
                Some((offset_x, offset_y, viewport_width, viewport_height)),
            );
        }
    }
    fn calculate_viewport_params(&self) -> (i32, i32, i32, i32) {
        (self.screen_size.0 as i32, self.screen_size.1 as i32, 0, 0)
    }
}
