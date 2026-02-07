use anyhow::Result;
use cgmath::Matrix4;

use crate::{
    render::{
        framebuffer::{
            compass::{self, Compass, Viewport},
            mini::Mini,
            postprocessing::{PostprocessingFramebuffer, ViewportScaleStrategy},
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
    pub resolution_fb: PostprocessingFramebuffer,
    pub shadow_fb: ShadowFramebuffer,
    pub current_fb: FrameBufferType,
    pub mini_fb: Mini,
    pub compass_fb: Compass,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameBufferType {
    Default,
    Postprocessing,
    Shadow,
    Mini,
    Compass,
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
            PostprocessingFramebuffer::new(resolution_width, resolution_height, screen)?;

        let shadow_fb = ShadowFramebuffer::new(shadow_width, shadow_height, screen)?;

        let mini_fb = Mini::new(320, 320)?;

        let compass = Compass::new(
            Viewport {
                x: 100,
                y: 100,
                width: 300,
                height: 300,
            },
            screen,
        )?;
        Ok(Self {
            resolution_fb,
            mini_fb,
            shadow_fb,
            current_fb: FrameBufferType::Default,
            compass_fb: compass,
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
            FrameBufferType::Postprocessing => {
                self.resolution_fb.begin_render();
            }
            FrameBufferType::Shadow => {
                self.shadow_fb.begin_render();
            }
            FrameBufferType::Mini => {
                self.mini_fb.begin_render();
            }
            FrameBufferType::Compass => {
                self.compass_fb.begin_render();
            }
        }
    }
    pub fn end_frame(&self, state: &crate::state::State) {
        match self.current_fb {
            FrameBufferType::Postprocessing => {
                if state.is_lowres {
                    self.resolution_fb.end_scene_render(state);
                }
            }
            FrameBufferType::Shadow => {
                self.shadow_fb.end_scene_render();
            }
            FrameBufferType::Mini => {
                self.mini_fb.end_scene_render();
            }
            FrameBufferType::Compass => {
                self.compass_fb.end_scene_render();
            }
            _ => {}
        }
        unsafe {
            gl::ClearColor(state.color.0, state.color.1, state.color.2, state.color.3);
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
        self.compass_fb.update_screen_size(screen);
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
