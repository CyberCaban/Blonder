use std::collections::HashMap;

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
    state::{Screen, State},
};

pub enum ButtonState {
    Nothing,
    Hovered,
    Clicked,
}

pub struct UIManager {
    ui_renderer: UIRenderer,
    text_renderer: TextRenderer,

    clicked_buttons: Vec<u32>,
}

impl UIManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ui_renderer: UIRenderer::new()?,
            text_renderer: TextRenderer::new()?,
            clicked_buttons: Vec::new(),
        })
    }
    pub fn begin_frame(&mut self, state: &State, render_screen: &Screen) {
        let State { screen, .. } = state;
        self.ui_renderer
            .update_projection(render_screen.width as f32, render_screen.height as f32);
        self.text_renderer
            .set_projection(render_screen.width as f32, render_screen.height as f32);

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
    pub fn button(
        &mut self,
        id: u32,
        text: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        font: &FontAtlasRef,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
    ) -> bool {
        let is_hovered =
            mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height;

        let was_clicked = if is_hovered && mouse_pressed {
            self.clicked_buttons.push(id);
            true
        } else {
            false
        };
        let color = if is_hovered {
            if mouse_pressed {
                Color::new(0.1, 0.3, 0.7, 1.0)
            } else {
                Color::new(0.3, 0.5, 0.9, 1.0)
            }
        } else {
            Color::new(0.2, 0.4, 0.8, 1.0)
        };

        self.draw_rect(x, y, width, height, color);

        let scale = 0.3;
        let text_width = font.measure_line(text, scale);
        let text_x = x + (width - text_width) / 2.0;
        let text_y = y + (height - font.size as f32 * scale) / 2.0;

        let text_color = if is_hovered && mouse_pressed {
            Color::white()
        } else {
            Color::black()
        };

        self.draw_text(font, text, text_x, text_y, scale, text_color);
        was_clicked
    }
}
