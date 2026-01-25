use std::os::raw::c_void;
#[cfg(debug_assertions)]
use std::time::Instant;

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImage};
use log::info;

#[derive(Debug, Clone, Copy)]
pub enum TextureFormatColor {
    RGBA8,
    RGBA16F,
    RGB8,
    R8,
    R16F,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormatDepth {
    Depth16,
    Depth24,
    Depth32F,
    Depth24Stencil8,
}
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    Color(TextureFormatColor),
    Depth(TextureFormatDepth),
}
impl TextureFormatColor {
    pub fn to_gl_enums(&self) -> (u32, u32, u32) {
        match self {
            &TextureFormatColor::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            TextureFormatColor::RGBA16F => (gl::RGBA16F, gl::RGBA, gl::FLOAT),
            TextureFormatColor::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            TextureFormatColor::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
            TextureFormatColor::R16F => (gl::R16F, gl::RED, gl::FLOAT),
        }
    }
}
impl TextureFormatDepth {
    pub fn to_gl_enums(&self) -> (u32, u32, u32, u32) {
        match self {
            TextureFormatDepth::Depth16 => (
                gl::DEPTH_COMPONENT16,
                gl::DEPTH_ATTACHMENT,
                gl::DEPTH_COMPONENT,
                gl::UNSIGNED_SHORT,
            ),
            TextureFormatDepth::Depth24 => (
                gl::DEPTH_COMPONENT24,
                gl::DEPTH_ATTACHMENT,
                gl::DEPTH_COMPONENT,
                gl::UNSIGNED_INT,
            ),
            TextureFormatDepth::Depth32F => (
                gl::DEPTH_COMPONENT32F,
                gl::DEPTH_ATTACHMENT,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
            ),
            TextureFormatDepth::Depth24Stencil8 => (
                gl::DEPTH24_STENCIL8,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::DEPTH_STENCIL,
                gl::UNSIGNED_INT_24_8,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest,
    Linear,
}
impl TextureFilter {
    pub fn to_gl_enums(&self) -> (u32, u32) {
        match self {
            TextureFilter::Nearest => (gl::NEAREST, gl::NEAREST),
            TextureFilter::Linear => (gl::LINEAR, gl::LINEAR),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextureWrap {
    Repeat,
    ClampToEdge,
    ClampToBorder,
}
impl TextureWrap {
    pub fn to_gl_enums(&self) -> u32 {
        match self {
            TextureWrap::Repeat => gl::REPEAT,
            TextureWrap::ClampToEdge => gl::CLAMP_TO_EDGE,
            TextureWrap::ClampToBorder => gl::CLAMP_TO_BORDER,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureConfig {
    pub wrap_s: i32,
    pub wrap_t: i32,
    pub texture_filtering: i32,
    pub mipmap_filtering: i32,
}
impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig {
            wrap_s: gl::REPEAT as i32,
            wrap_t: gl::REPEAT as i32,
            texture_filtering: gl::LINEAR as i32,
            mipmap_filtering: gl::LINEAR_MIPMAP_LINEAR as i32,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Texture {
    id: u32,
}

impl Texture {
    pub fn white() -> Self {
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            let white_pixel: [u8; 4] = [255, 255, 255, 255];

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                white_pixel.as_ptr() as *const _,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            Self { id }
        }
    }
    pub fn black() -> Self {
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            let white_pixel: [u8; 4] = [0, 0, 0, 1];

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                white_pixel.as_ptr() as *const _,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            Self { id }
        }
    }
    pub fn empty_texture() -> Self {
        Self { id: 0 }
    }
    pub fn with_config(texture_path: &str, config: TextureConfig) -> Result<Self> {
        #[cfg(debug_assertions)]
        let now = Instant::now();

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, config.wrap_s);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, config.wrap_t);
            // texture filtering
            // LINEAR or NEAREST
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                config.texture_filtering,
            );
            // mipmap filtering
            // GL_NEAREST_MIPMAP_NEAREST: takes the nearest mipmap to match the pixel size and uses nearest neighbor interpolation for texture sampling.
            // GL_LINEAR_MIPMAP_NEAREST: takes the nearest mipmap level and samples that level using linear interpolation.
            // GL_NEAREST_MIPMAP_LINEAR: linearly interpolates between the two mipmaps that most closely match the size of a pixel and samples the interpolated level via nearest neighbor interpolation.
            // GL_LINEAR_MIPMAP_LINEAR: linearly interpolates between the two closest mipmaps and samples the interpolated level via linear interpolation.
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                config.mipmap_filtering,
            );

            let image = image::open(texture_path)
                .context(format!("Cannot find texture [{texture_path}]"))?;
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

        #[cfg(debug_assertions)]
        info!(
            "Creating texture [{}] took {}ms",
            texture_path,
            (Instant::now() - now).as_millis()
        );

        Ok(Self { id: texture })
    }
    /// Make texture struct from id
    /// id must be uploaded to OpenGL manually
    pub fn from_id(id: u32) -> Self {
        Self { id }
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
