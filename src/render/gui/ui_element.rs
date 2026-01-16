use std::sync::Arc;

use crate::{
    render::{
        color::Color,
        renderer::{FontAtlasRef, TextureRef},
    },
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
    Text {
        font: FontAtlasRef,
        text: String,
        x: f32,
        y: f32,
        scale: f32,
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
    pub fn new_text(font: FontAtlasRef, text: String, x: f32, y: f32, scale: f32, color: Color) -> Self {
        Self::Text {
            font,
            text,
            x,
            y,
            scale,
            color,
        }
    }
}
