use crate::{
    render::{framebuffer::Framebuffer, shader::Shader},
    state::Screen,
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
pub struct ResolutionFramebuffer {
    framebuffer: Framebuffer,
    pub render_width: i32,
    pub render_height: i32,
    screen_size: (u32, u32),
    screen_shader: Shader,
    scale_strategy: ViewportScaleStrategy,
}

impl ResolutionFramebuffer {
    pub fn new(
        width: i32,
        height: i32,
        screen: &Screen,
        scale_strategy: ViewportScaleStrategy,
    ) -> Result<Self> {
        let mut framebuffer = Framebuffer::new(width, height)?;
        framebuffer.add_color_attachment(
            0,
            TextureFormatColor::RGBA8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
        )?;
        framebuffer.add_depth_attachment(
            TextureFormatDepth::Depth24Stencil8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
        )?;
        framebuffer.clear_color = (0.1, 0.1, 0.1, 0.1);
        framebuffer.use_depth_test = true;
        framebuffer.check_complete()?;

        let screen_shader = Shader::new(
            "assets/shaders/screen/vert.glsl",
            "assets/shaders/screen/frag.glsl",
        )?;
        Ok(Self {
            framebuffer,
            render_width: width,
            screen_shader,
            render_height: height,
            screen_size: (screen.width, screen.height),
            scale_strategy,
        })
    }
    pub fn update_screen_size(&mut self, screen: &Screen) {
        self.screen_size = (screen.width, screen.height)
    }
    pub fn set_scale_strategy(&mut self, scale_strategy: ViewportScaleStrategy) {
        self.scale_strategy = scale_strategy;
    }
    pub fn set_render_size(&mut self, width: u32, height: u32) {
        self.render_width = width as i32;
        self.render_height = height as i32;
        self.framebuffer.resize(width as i32, height as i32);
    }
    pub fn begin_render(&self) {
        self.framebuffer.begin_render();
    }
    pub fn end_scene_render(&self) {
        self.framebuffer.unbind();
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            self.render_scene_to_screen();
        }
    }
    fn render_scene_to_screen(&self) {
        let (viewport_width, viewport_height, offset_x, offset_y) =
            self.calculate_viewport_params();

        self.framebuffer.render_to_screen(
            &self.screen_shader,
            Some((offset_x, offset_y, viewport_width, viewport_height)),
        );
    }
    fn calculate_viewport_params(&self) -> (i32, i32, i32, i32) {
        match self.scale_strategy {
            ViewportScaleStrategy::Fit => {
                let screen_width = self.screen_size.0 as f32;
                let screen_height = self.screen_size.1 as f32;
                let render_width = self.render_width as f32;
                let render_height = self.render_height as f32;

                let screen_aspect = screen_width / screen_height;
                let render_aspect = render_width / render_height;

                let (viewport_width, viewport_height, offset_x, offset_y);

                if screen_aspect > render_aspect {
                    let scale = screen_height / render_height;
                    viewport_height = screen_height as i32;
                    viewport_width = (render_width * scale) as i32;
                    offset_x = ((screen_width - viewport_width as f32) / 2.0) as i32;
                    offset_y = 0;
                } else if screen_aspect < render_aspect {
                    let scale = screen_width / render_width;
                    viewport_width = screen_width as i32;
                    viewport_height = (render_height * scale) as i32;
                    offset_x = 0;
                    offset_y = ((screen_height - viewport_height as f32) / 2.0) as i32;
                } else {
                    viewport_width = screen_width as i32;
                    viewport_height = screen_height as i32;
                    offset_x = 0;
                    offset_y = 0;
                }

                (viewport_width, viewport_height, offset_x, offset_y)
            }
            ViewportScaleStrategy::PixelPerfect => {
                let max_scale_x = self.screen_size.0 as f32 / self.render_width as f32;
                let max_scale_y = self.screen_size.1 as f32 / self.render_height as f32;
                let scale = max_scale_x.min(max_scale_y).floor().max(1.0);

                let render_width = (self.render_width as f32 * scale) as i32;
                let render_height = (self.render_height as f32 * scale) as i32;
                let offset_x = (self.screen_size.0 as i32 - render_width) / 2;
                let offset_y = (self.screen_size.1 as i32 - render_height) / 2;

                (render_width, render_height, offset_x, offset_y)
            }
            ViewportScaleStrategy::Stretch => {
                (self.screen_size.0 as i32, self.screen_size.1 as i32, 0, 0)
            }
        }
    }
}
