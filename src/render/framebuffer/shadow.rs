use crate::{
    render::{framebuffer::Framebuffer, shader::Shader},
    state::Screen,
    texture::{Texture, TextureFilter, TextureFormatDepth, TextureWrap},
};
use anyhow::Result;
use cgmath::{Array, Matrix4, Point3, Vector3, ortho};

#[derive(Debug)]
pub struct ShadowFramebuffer {
    framebuffer: Framebuffer,
    pub shadow_width: i32,
    pub shadow_height: i32,
    near_plane: f32,
    far_plane: f32,
    screen_size: (u32, u32),
}

impl ShadowFramebuffer {
    pub fn new(width: i32, height: i32, screen: &Screen) -> Result<Self> {
        let mut framebuffer = Framebuffer::new(width, height)?;
        framebuffer.add_depth_attachment(
            TextureFormatDepth::Depth32F,
            TextureFilter::Nearest,
            TextureWrap::ClampToBorder,
        )?;
        framebuffer.clear_color = (1.0, 0.0, 0.0, 1.0);
        framebuffer.use_depth_test = true;
        framebuffer.check_complete()?;
        Ok(Self {
            framebuffer,
            near_plane: 0.1,
            far_plane: 100.0,
            shadow_width: width,
            shadow_height: height,
            screen_size: (screen.width, screen.height),
        })
    }

    pub fn update_screen_size(&mut self, screen: &Screen) {
        self.screen_size = (screen.width, screen.height)
    }
    pub fn set_render_size(&mut self, width: u32, height: u32) {
        self.shadow_width = width as i32;
        self.shadow_height = height as i32;
    }
    pub fn begin_render(&self) {
        self.framebuffer.begin_render();
        unsafe {
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            gl::Viewport(0, 0, self.shadow_width, self.shadow_height);

            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::CullFace(gl::FRONT);
        }
    }
    pub fn end_scene_render(&self) {
        unsafe {
            gl::CullFace(gl::BACK);
            gl::DrawBuffer(gl::BACK);
            gl::ReadBuffer(gl::BACK);
        }
        self.framebuffer.unbind();
    }
    pub fn calculate_light_space_matrix(&self) -> Matrix4<f32> {
        #[rustfmt::skip]
        let light_projection = ortho(
            -10.0, 10.0, // left, right
            -10.0, 10.0, // bottom, top
            self.near_plane,
            self.far_plane,
        );

        let light_view = Matrix4::look_at(
            Point3::from_value(-1.0),
            Point3::from_value(0.0),
            Vector3::unit_y(),
        );

        light_projection * light_view
    }
    fn render_scene_to_screen(&self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);

            gl::ActiveTexture(gl::TEXTURE0);

            // let (viewport_width, viewport_height, offset_x, offset_y) =
            //     self.calculate_viewport_params();

            // gl::Viewport(offset_x, offset_y, viewport_width, viewport_height);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);

            gl::Viewport(0, 0, self.screen_size.0 as i32, self.screen_size.1 as i32);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
    pub fn get_color_texture(&self) -> Option<&Texture> {
        self.framebuffer.get_color_texture(0)
    }

    pub fn get_depth_texture(&self) -> Option<&Texture> {
        self.framebuffer.get_depth_texture()
    }
}
