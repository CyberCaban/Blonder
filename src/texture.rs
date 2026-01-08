use std::os::raw::c_void;

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImage, RgbaImage};

#[derive(Debug)]
pub struct TextureConfig {
    pub wrap_s: i32,
    pub wrap_t: i32,
}
impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig {
            wrap_s: gl::REPEAT as i32,
            wrap_t: gl::REPEAT as i32,
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    id: u32,
}

impl Texture {
    pub fn empty_texture() -> Self {
        Self { id: 0 }
    }
    pub fn with_config(texture_path: &str, config: TextureConfig) -> Result<Self> {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, config.wrap_s);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, config.wrap_t);
            // texture filtering
            // LINEAR or NEAREST
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            // mipmap filtering
            // GL_NEAREST_MIPMAP_NEAREST: takes the nearest mipmap to match the pixel size and uses nearest neighbor interpolation for texture sampling.
            // GL_LINEAR_MIPMAP_NEAREST: takes the nearest mipmap level and samples that level using linear interpolation.
            // GL_NEAREST_MIPMAP_LINEAR: linearly interpolates between the two mipmaps that most closely match the size of a pixel and samples the interpolated level via nearest neighbor interpolation.
            // GL_LINEAR_MIPMAP_LINEAR: linearly interpolates between the two closest mipmaps and samples the interpolated level via linear interpolation.
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );

            let image = image::open(texture_path)
                .context(format!("Cannot find texture [{}]", texture_path))?
                .rotate180()
                .fliph();
            let (width, height) = (image.width(), image.height());
            let raw_image = image.to_rgba().into_raw();
            let data = raw_image.as_ptr() as *const c_void;
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA.try_into().unwrap(),
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Ok(Self { id: texture })
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn new(texture_path: &str) -> Result<Self> {
        Self::with_config(texture_path, TextureConfig::default())
    }
    pub fn from_image(image: DynamicImage, config: TextureConfig) -> Result<Self> {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, config.wrap_s);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, config.wrap_t);
            // texture filtering
            // LINEAR or NEAREST
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            // mipmap filtering
            // GL_NEAREST_MIPMAP_NEAREST: takes the nearest mipmap to match the pixel size and uses nearest neighbor interpolation for texture sampling.
            // GL_LINEAR_MIPMAP_NEAREST: takes the nearest mipmap level and samples that level using linear interpolation.
            // GL_NEAREST_MIPMAP_LINEAR: linearly interpolates between the two mipmaps that most closely match the size of a pixel and samples the interpolated level via nearest neighbor interpolation.
            // GL_LINEAR_MIPMAP_LINEAR: linearly interpolates between the two closest mipmaps and samples the interpolated level via linear interpolation.
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );

            let image = image.rotate180().fliph();
            let (width, height) = (image.width(), image.height());
            let raw_image = image.to_rgba().into_raw();
            let data = raw_image.as_ptr() as *const c_void;
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA.try_into().unwrap(),
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Ok(Self { id: texture })
    }
    pub fn bind(&self, texture_unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
    pub fn use_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
    pub fn use_empty_texture() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
