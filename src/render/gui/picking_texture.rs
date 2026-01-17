use std::{ffi::c_void, mem, ptr};

use anyhow::Result;

use crate::shader::Shader;

#[repr(C)]
#[derive(Debug)]
pub struct PixelInfo {
    pub obj_id: u32,
    pub draw_id: u32,
    pub prim_id: u32,
}
#[derive(Debug)]
pub struct PickingTexture {
    fbo: u32,
    picking_texture: u32,
    depth_texture: u32,
    pub shader: Shader,
}

impl PickingTexture {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let (mut fbo, mut picking_texture, mut depth_texture) = (0, 0, 0);
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            // Create picking texture
            gl::GenTextures(1, &mut picking_texture);
            gl::BindTexture(gl::TEXTURE_2D, picking_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB32UI as i32,
                width,
                height,
                0,
                gl::RED_INTEGER,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // attach tex to fbo
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                picking_texture,
                0,
            );

            // Create depth texture
            gl::GenTextures(1, &mut depth_texture);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture);
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

            // attach tex to fbo
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_texture,
                0,
            );

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer not complete")
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Ok(Self {
            fbo,
            picking_texture,
            depth_texture,
            shader: Shader::new(
                "assets/shaders/ui/picking/vert.glsl",
                "assets/shaders/ui/picking/frag.glsl",
            )?,
        })
    }
    pub fn enable_writing(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    pub fn disable_writing(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }
    }
    pub fn read_pixel(&self, x: u32, y: u32) -> u32 {
        let mut pixel = 0;
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::ReadPixels(
                x as i32,
                y as i32,
                1,
                1,
                gl::RED_INTEGER,
                gl::UNSIGNED_INT,
                &mut pixel as *mut u32 as *mut _,
            );
            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }
        pixel
    }
}

impl Drop for PickingTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteTextures(1, &self.depth_texture);
            gl::DeleteTextures(1, &self.picking_texture);
        }
    }
}
