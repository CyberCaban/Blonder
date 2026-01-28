use anyhow::Result;
use cgmath::Matrix4;

use crate::{
    render::{
        framebuffer::{
            mini::Mini,
            resolution::{ResolutionFramebuffer, ViewportScaleStrategy},
            shadow::ShadowFramebuffer,
        },
        renderer::ShaderRef,
        shader::Shader,
    },
    state::Screen,
    texture::Texture,
};

#[derive(Debug)]
pub struct FramebufferManager {
    pub resolution_fb: ResolutionFramebuffer,
    pub shadow_fb: ShadowFramebuffer,
    pub current_fb: FrameBufferType,
    pub mini_fb: Mini,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameBufferType {
    Default,
    Resolution,
    Shadow,
    Mini,
}

impl FramebufferManager {
    pub fn new(
        resolution_width: i32,
        resolution_height: i32,
        shadow_width: i32,
        shadow_height: i32,
        screen: &Screen,
    ) -> Result<Self> {
        let resolution_fb =
            ResolutionFramebuffer::new(resolution_width, resolution_height, screen)?;

        let shadow_fb = ShadowFramebuffer::new(shadow_width, shadow_height, screen)?;

        let mini_fb = Mini::new(320, 320)?;

        Ok(Self {
            resolution_fb,
            mini_fb,
            shadow_fb,
            current_fb: FrameBufferType::Default,
        })
    }
    pub fn begin_frame(&mut self, target: FrameBufferType, state: &crate::state::State) {
        self.current_fb = target;

        match target {
            FrameBufferType::Default => unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::Viewport(0, 0, state.screen.width as i32, state.screen.height as i32);
                gl::ClearColor(state.color.0, state.color.1, state.color.2, state.color.3);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::Enable(gl::DEPTH_TEST);
            },
            FrameBufferType::Resolution => {
                self.resolution_fb.begin_render();
            }
            FrameBufferType::Shadow => {
                self.shadow_fb.begin_render();
            }
            FrameBufferType::Mini => {
                self.mini_fb.begin_render();
            }
        }
    }
    pub fn end_frame(&self, state: &crate::state::State) {
        match self.current_fb {
            FrameBufferType::Resolution => {
                if state.is_lowres {
                    self.resolution_fb.end_scene_render(state);
                } else {
                }
            }
            FrameBufferType::Shadow => {
                self.shadow_fb.end_scene_render();
            }
            FrameBufferType::Mini => {
                self.mini_fb.end_scene_render();
            }
            _ => {}
        }
    }
    pub fn get_shader(&self) -> Option<ShaderRef> {
        match self.current_fb {
            FrameBufferType::Mini => Some(self.mini_fb.normal_shader.clone()),
            _ => None,
        }
    }

    pub fn update_screen_size(&mut self, screen: &Screen) -> Result<()> {
        self.resolution_fb.update_screen_size(screen);
        self.shadow_fb.update_screen_size(screen);
        Ok(())
    }
    pub fn get_shadow_depth_texture(&self) -> Option<&Texture> {
        self.shadow_fb.get_depth_texture()
        // match self.current_fb {
        //     FrameBufferType::Shadow => self.shadow_fb.get_depth_texture(),
        //     _ => None,
        // }
    }
}
