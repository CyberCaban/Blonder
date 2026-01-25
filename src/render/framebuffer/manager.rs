use anyhow::Result;
use cgmath::Matrix4;

use crate::{
    render::framebuffer::{
        resolution::{ResolutionFramebuffer, ViewportScaleStrategy},
        shadow::ShadowFramebuffer,
    },
    state::Screen,
    texture::Texture,
};

#[derive(Debug)]
pub struct FramebufferManager {
    pub resolution_fb: ResolutionFramebuffer,
    pub shadow_fb: ShadowFramebuffer,
    pub current_fb: FrameBufferType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameBufferType {
    Default,
    Resolution,
    Shadow,
}

impl FramebufferManager {
    pub fn new(
        resolution_width: i32,
        resolution_height: i32,
        shadow_width: i32,
        shadow_height: i32,
        screen: &Screen,
        scale_strategy: ViewportScaleStrategy,
    ) -> Result<Self> {
        let resolution_fb = ResolutionFramebuffer::new(
            resolution_width,
            resolution_height,
            screen,
            scale_strategy,
        )?;

        let shadow_fb = ShadowFramebuffer::new(shadow_width, shadow_height, screen)?;

        Ok(Self {
            resolution_fb,
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
                if state.is_lowres {
                    self.resolution_fb.begin_render();
                } else {
                    // Если низкое разрешение выключено, рендерим напрямую
                    self.begin_frame(FrameBufferType::Default, state);
                }
            }
            FrameBufferType::Shadow => {
                self.shadow_fb.begin_render();
            }
        }
    }
    pub fn end_frame(&self, state: &crate::state::State) {
        match self.current_fb {
            FrameBufferType::Resolution => {
                if state.is_lowres {
                    self.resolution_fb.end_scene_render();
                }
            }
            FrameBufferType::Shadow => {
                self.shadow_fb.end_scene_render();
            }
            _ => {}
        }
    }
    pub fn get_current_shadow_matrix(&self) -> Option<Matrix4<f32>> {
        if self.current_fb == FrameBufferType::Shadow {
            Some(self.shadow_fb.calculate_light_space_matrix())
        } else {
            None
        }
    }

    pub fn bind_shadow_texture(&self, texture_unit: u32) {
        if let Some(texture) = self.shadow_fb.get_depth_texture() {
            texture.bind(texture_unit);
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
