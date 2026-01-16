use anyhow::Result;

use crate::{
    render::{
        color::Color,
        gui::{
            text_renderer::{TextRenderParams, TextRenderer},
            ui_renderer::UIRenderer,
        },
        renderer::{FontAtlasRef, TextureRef},
    },
    state::State,
};

pub struct UIManager {
    ui_renderer: UIRenderer,
    text_renderer: TextRenderer,
}

impl UIManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ui_renderer: UIRenderer::new()?,
            text_renderer: TextRenderer::new()?,
        })
    }
    pub fn begin_frame(&mut self, state: &State) {
        let State { screen, .. } = state;
        self.ui_renderer
            .update_projection(screen.width as f32, screen.height as f32);
        self.text_renderer
            .set_projection(screen.width as f32, screen.height as f32);

        self.ui_renderer.begin_frame();

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }
    }
    pub fn end_frame(&mut self) {
        self.ui_renderer.end_frame();

        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.ui_renderer.draw_rect(x, y, width, height, &color);
    }

    pub fn draw_texture(
        &mut self,
        texture: TextureRef,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        self.ui_renderer
            .draw_texture(texture, x, y, width, height, &color);
    }

    pub fn draw_text(
        &mut self,
        font: &FontAtlasRef,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: Color,
    ) {
        self.ui_renderer.end_frame();

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }

        let params = TextRenderParams { scale, color };

        self.text_renderer.render_text(font, text, x, y, &params);

        self.ui_renderer.begin_frame();
    }
}
