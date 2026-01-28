use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::{
    render::shader::Shader,
    state::State,
    texture::{Texture, TextureFilter, TextureFormatColor, TextureFormatDepth, TextureWrap},
};

pub mod manager;
pub mod mini;
pub mod resolution;
pub mod shadow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentType {
    Color(u32), // color attachment index
    Depth,
    Stencil,
    DepthStencil,
}
#[derive(Debug)]
pub struct Framebuffer {
    fbo: u32,
    attachments: HashMap<AttachmentType, Texture>,
    pub width: i32,
    pub height: i32,
    clear_color: (f32, f32, f32, f32),
    screen_quad_vao: u32,
    screen_quad_vbo: u32,
    use_depth_test: bool,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let mut fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
        }
        let (screen_quad_vao, screen_quad_vbo) = Self::create_screen_quad();
        Ok(Self {
            fbo,
            attachments: HashMap::new(),
            width,
            clear_color: (0.0, 0.0, 0.0, 1.0),
            screen_quad_vao,
            screen_quad_vbo,
            height,
            use_depth_test: true,
        })
    }
    fn create_screen_quad() -> (u32, u32) {
        unsafe {
            let mut vao = 0;
            let mut vbo = 0;

            #[rustfmt::skip]
            let vertices: [f32; 24] = [
                // pos      // tex
                -1.0, -1.0,  0.0, 0.0,
                 1.0, -1.0,  1.0, 0.0,
                1.0,  1.0,  1.0, 1.0,

                -1.0, -1.0,  0.0, 0.0,
                1.0,  1.0,  1.0, 1.0,
                -1.0,  1.0,  0.0, 1.0,
            ];

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Position
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // uv
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as i32,
                (2 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);

            (vao, vbo)
        }
    }
    pub fn add_color_attachment(
        &mut self,
        index: u32,
        format: TextureFormatColor,
        filter: TextureFilter,
        wrap: TextureWrap,
    ) -> Result<()> {
        self.bind();
        let (internal_format, data_format, data_type) = format.to_gl_enums();
        let (min_filter, mag_filter) = filter.to_gl_enums();
        let wrap_mode = wrap.to_gl_enums();

        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                self.width,
                self.height,
                0,
                data_format,
                data_type,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_mode as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_mode as i32);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0 + index,
                gl::TEXTURE_2D,
                texture_id,
                0,
            );
        }

        let texture = Texture::from_id(texture_id);
        self.attachments
            .insert(AttachmentType::Color(index), texture);
        Ok(())
    }
    pub fn add_depth_attachment(
        &mut self,
        format: TextureFormatDepth,
        filter: TextureFilter,
        wrap: TextureWrap,
    ) -> Result<()> {
        self.bind();
        let (internal_format, attachment_type, data_format, data_type) = format.to_gl_enums();
        let (min_filter, mag_filter) = filter.to_gl_enums();
        let wrap_mode = wrap.to_gl_enums();
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                self.width,
                self.height,
                0,
                data_format,
                data_type,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_mode as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_mode as i32);

            if matches!(
                format,
                TextureFormatDepth::Depth16
                    | TextureFormatDepth::Depth24
                    | TextureFormatDepth::Depth32F
            ) {
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_COMPARE_MODE,
                    gl::COMPARE_REF_TO_TEXTURE as i32,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_FUNC, gl::LEQUAL as i32);
            }

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachment_type,
                gl::TEXTURE_2D,
                texture_id,
                0,
            );
        }

        let texture = Texture::from_id(texture_id);
        if format == TextureFormatDepth::Depth24Stencil8 {
            self.attachments
                .insert(AttachmentType::DepthStencil, texture);
        } else {
            self.attachments.insert(AttachmentType::Depth, texture);
        }
        Ok(())
    }
    pub fn check_complete(&self) -> Result<()> {
        self.bind();
        unsafe {
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                let error_msg = match status {
                    gl::FRAMEBUFFER_UNDEFINED => "FRAMEBUFFER_UNDEFINED",
                    gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
                    gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                        "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"
                    }
                    gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
                    gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
                    gl::FRAMEBUFFER_UNSUPPORTED => "FRAMEBUFFER_UNSUPPORTED",
                    gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
                    _ => "UNKNOWN_ERROR",
                };
                bail!("Framebuffer is not complete: {}", error_msg);
            }
        }
        Ok(())
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }
    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        self.width = width;
        self.height = height;

        for (attachment_type, texture) in self.attachments.iter_mut() {
            let texture_id = texture.id();

            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, texture_id);

                match attachment_type {
                    AttachmentType::Color(_) => {
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGBA as i32,
                            width,
                            height,
                            0,
                            gl::RGBA,
                            gl::UNSIGNED_BYTE,
                            std::ptr::null(),
                        );
                    }
                    AttachmentType::Depth | AttachmentType::DepthStencil => {
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::DEPTH_COMPONENT as i32,
                            width,
                            height,
                            0,
                            gl::DEPTH_COMPONENT,
                            gl::FLOAT,
                            std::ptr::null(),
                        );
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
    pub fn begin_render(&self) {
        self.bind();
        unsafe {
            gl::Viewport(0, 0, self.width, self.height);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(
                self.clear_color.0,
                self.clear_color.1,
                self.clear_color.2,
                self.clear_color.3,
            );
            if self.use_depth_test {
                gl::Enable(gl::DEPTH_TEST);
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }
    }
    pub fn get_color_texture(&self, index: u32) -> Option<&Texture> {
        self.attachments.get(&AttachmentType::Color(index))
    }
    pub fn get_depth_texture(&self) -> Option<&Texture> {
        self.attachments
            .get(&AttachmentType::Depth)
            .or_else(|| self.attachments.get(&AttachmentType::DepthStencil))
    }
    pub fn render_to_screen(
        &self,
        state: &State,
        shader: &Shader,
        viewport: Option<(i32, i32, i32, i32)>,
    ) {
        unsafe {
            gl::ClearColor(
                self.clear_color.0,
                self.clear_color.1,
                self.clear_color.2,
                self.clear_color.3,
            );
            gl::Disable(gl::DEPTH_TEST);
            shader.use_shader();

            // Биндим все цветовые текстуры
            let mut color_count = 0;
            for i in 0..4 {
                if let Some(texture) = self.get_color_texture(i) {
                    texture.bind(i);
                    shader.set_int(&format!("colorTexture{}", i), i as i32);
                    color_count += 1;
                }
            }

            // Биндим depth текстуру если есть
            if let Some(depth_texture) = self.get_depth_texture() {
                depth_texture.bind(color_count);
                shader.set_int("depthTexture", color_count as i32);
            }

            if let Some((x, y, width, height)) = viewport {
                gl::Viewport(x, y, width, height);
            }

            gl::BindVertexArray(self.screen_quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);

            gl::Enable(gl::DEPTH_TEST);
            // reset viewport
            unsafe {
                gl::Viewport(0, 0, state.screen.width as i32, state.screen.height as i32);
            }
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteVertexArrays(1, &self.screen_quad_vao);
            gl::DeleteBuffers(1, &self.screen_quad_vbo);
        }
    }
}
