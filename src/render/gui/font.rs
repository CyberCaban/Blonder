use std::{collections::HashMap, fs, time::Instant};

use anyhow::{Context, Result};
use image::{DynamicImage::ImageRgba8, Rgba, RgbaImage};
use log::info;
use rusttype::{Font, Scale, point};

use crate::texture::{Texture, TextureConfig};

#[derive(Debug)]
pub struct Character {
    pub texture_id: Texture,
    pub size: (f32, f32),
    pub bearing: (f32, f32),
    pub advance: f32,
}

#[derive(Debug)]
pub struct FontAtlas {
    pub characters: HashMap<char, Character>,
    pub size: u32,
}

impl FontAtlas {
    pub fn new(font_path: &str, font_size: u32) -> Result<Self> {
        #[cfg(debug_assertions)]
        let now = Instant::now();
        let font_data =
            fs::read(font_path).context(format!("Failed to load font [{font_path}]"))?;
        let font = Font::try_from_vec(font_data)
            .context(format!("Failed to parse font: [{font_path}]"))?;
        let scale = Scale::uniform(font_size as f32);
        let v_metrics = font.v_metrics(scale);
        let mut characters = HashMap::new();
        for ch in 32..128u8 {
            let ch = ch as char;
            let glyph = font.glyph(ch).scaled(scale);
            let h_metrics = glyph.h_metrics();
            let positioned = glyph.positioned(point(0.0, v_metrics.ascent));

            if let Some(bounding_box) = positioned.pixel_bounding_box() {
                let (w, h) = (bounding_box.width() as u32, bounding_box.height() as u32);
                if w == 0 || h == 0 {
                    let character = Character {
                        texture_id: Texture::empty_texture(),
                        size: (0.0, 0.0),
                        bearing: (0.0, 0.0),
                        advance: h_metrics.advance_width,
                    };
                    characters.insert(ch, character);
                    continue;
                }
                let padding = 2;
                let img_width = w + padding * 2;
                let img_height = h + padding * 2;
                let mut image = RgbaImage::new(img_width, img_height);
                for pixel in image.pixels_mut() {
                    *pixel = Rgba([0, 0, 0, 0]);
                }

                positioned.draw(|x, y, v| {
                    let img_x = x as i32 + padding as i32;
                    let img_y = y as i32 + padding as i32;
                    if img_x >= 0
                        && img_x < img_width as i32
                        && img_y >= 0
                        && img_y < img_height as i32
                    {
                        let inverted_y = (img_height as i32 - 1 - img_y) as u32;
                        let alpha = (v * 255.0) as u8;
                        image.put_pixel(
                            img_x as u32,
                            inverted_y,
                            image::Rgba([alpha, alpha, alpha, alpha]),
                        );
                    }
                });
                // image debug
                // image.save(format!("out/{}.png", ch));
                // unsafe {
                //     gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                // }
                let texture = Texture::from_image(
                    ImageRgba8(image),
                    TextureConfig {
                        wrap_s: gl::CLAMP_TO_EDGE as i32,
                        wrap_t: gl::CLAMP_TO_EDGE as i32,
                        ..Default::default()
                    },
                )
                .context(format!(
                    "Failed to create texture for font [{font_path}], char [{ch}]"
                ))?;
                let character = Character {
                    texture_id: texture,
                    size: (img_width as f32, img_height as f32),
                    bearing: (
                        bounding_box.min.x as f32 - padding as f32,
                        v_metrics.ascent - bounding_box.min.y as f32 - padding as f32,
                    ),
                    advance: h_metrics.advance_width,
                };
                characters.insert(ch, character);
            } else {
                let character = Character {
                    texture_id: Texture::empty_texture(),
                    size: (0.0, 0.0),
                    bearing: (0.0, 0.0),
                    advance: h_metrics.advance_width,
                };
                characters.insert(ch, character);
                continue;
            }
        }
        #[cfg(debug_assertions)]
        info!(
            "Creating atlas [{}] took {}ms",
            font_path,
            (Instant::now() - now).as_millis()
        );
        Ok(FontAtlas {
            characters,
            size: font_size,
        })
    }
    pub fn measure_line(&self, text: &str, scale: f32) -> f32 {
        let mut width = 0.0;

        for ch in text.chars() {
            if let Some(character) = self.characters.get(&ch) {
                width += character.advance * scale;
            } else {
                width += self.size as f32 * 0.0 * scale;
            }
        }

        width
    }
    pub fn get_character(&self, ch: char) -> Option<&Character> {
        self.characters.get(&ch)
    }
}

impl Drop for FontAtlas {
    fn drop(&mut self) {
        unsafe {
            for ch in self.characters.values() {
                gl::DeleteTextures(1, &ch.texture_id.id());
            }
        }
    }
}
