use std::sync::Arc;

use crate::{
    render::{color::Color, renderer::TextureRef},
    texture::Texture,
};

#[derive(Debug, Clone)]
pub enum UIElement {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
    Texture {
        texture: TextureRef,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
}

impl UIElement {
    pub fn new_rect(x: f32, y: f32, width: f32, height: f32, color: Color) -> Self {
        Self::Rect {
            x,
            y,
            width,
            height,
            color,
        }
    }

    pub fn new_texture(
        texture: TextureRef,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) -> Self {
        Self::Texture {
            texture,
            x,
            y,
            width,
            height,
            color,
        }
    }
}
