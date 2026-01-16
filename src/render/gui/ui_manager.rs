use anyhow::Result;

use crate::{
    render::{
        color::Color,
        gui::{ui_element::UIElement, ui_renderer::UIRenderer},
        renderer::TextureRef,
    },
    state::State,
};

pub struct UIManager {
    ui_renderer: UIRenderer,
    ui_elements: Vec<UIElement>,
    ui_batch_dirty: bool,
}

impl UIManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ui_renderer: UIRenderer::new()?,
            ui_elements: vec![],
            ui_batch_dirty: false,
        })
    }
    pub fn render(&mut self, state: &State) {
        self.ui_renderer
            .update_projection(state.screen.width as f32, state.screen.height as f32);

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
}
