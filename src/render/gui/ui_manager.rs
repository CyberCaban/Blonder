use anyhow::Result;

use crate::{
    render::{
        color::Color,
        gui::{
            text_renderer::{TextRenderParams, TextRenderer},
            ui_element::UIElement,
            ui_renderer::UIRenderer,
        },
        renderer::{FontAtlasRef, TextureRef},
    },
    state::State,
};

pub struct UIManager {
    ui_renderer: UIRenderer,
    text_render: TextRenderer,
    ui_elements: Vec<UIElement>,
    ui_batch_dirty: bool,
}

impl UIManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ui_renderer: UIRenderer::new()?,
            text_render: TextRenderer::new()?,
            ui_elements: vec![],
            ui_batch_dirty: false,
        })
    }
    pub fn render(&mut self, state: &State) {
        let State { screen, .. } = state;
        self.ui_renderer
            .update_projection(screen.width as f32, screen.height as f32);
        self.text_render
            .set_projection(screen.width as f32, screen.height as f32);

        self.ui_renderer.begin_frame();
        for element in &self.ui_elements {
            match element {
                UIElement::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    self.ui_renderer.draw_rect(*x, *y, *width, *height, color);
                }
                UIElement::Texture {
                    texture,
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    self.ui_renderer
                        .draw_texture(texture.clone(), *x, *y, *width, *height, color);
                }
                UIElement::Text {
                    font,
                    text,
                    x,
                    y,
                    scale,
                    color,
                } => {
                    let params = TextRenderParams {
                        scale: *scale,
                        color: color.clone(),
                    };
                    unsafe {
                        gl::Disable(gl::DEPTH_TEST);
                        gl::Enable(gl::BLEND);
                        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    }

                    self.text_render.render_text(font, text, *x, *y, &params);

                    self.ui_renderer.begin_frame();
                }
                _ => todo!(),
            }
        }
        self.ui_renderer.end_frame();
    }
    pub fn add_ui_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.ui_elements
            .push(UIElement::new_rect(x, y, width, height, color));
        self.ui_batch_dirty = true;
    }

    pub fn add_ui_texture(
        &mut self,
        texture: TextureRef,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        self.ui_elements
            .push(UIElement::new_texture(texture, x, y, width, height, color));
        self.ui_batch_dirty = true;
    }
    pub fn add_text(
        &mut self,
        font: FontAtlasRef,
        text: String,
        x: f32,
        y: f32,
        scale: f32,
        color: Color,
    ) {
        self.ui_elements
            .push(UIElement::new_text(font, text, x, y, scale, color));
    }
}
