use crate::{shader::Shader, state::Screen, texture::Texture};
use anyhow::Result;

pub struct Framebuffer {
    fbo: u32,
    rbo: u32,
    render_texture: Texture,
    screen_shader: Shader,
    pub render_width: i32,
    pub render_height: i32,
    screen_quad_vao: u32,
    screen_quad_vbo: u32,
    screen_size: (u32, u32),
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, screen: &Screen) -> Result<Self> {
        let mut fbo = 0;
        let mut rbo = 0;
        let mut texture_id = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            // Create render texture
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
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

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // attach tex to fbo
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture_id,
                0,
            );

            // make rbo
            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, 800, 600);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            // attach rbo to fbo
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer not complete")
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        }
        let (screen_quad_vao, screen_quad_vbo) = Self::create_screen_quad();
        let screen_shader = Shader::new("assets/shaders/screen/vert.glsl", "assets/shaders/screen/frag.glsl")?;
        Ok(Self {
            fbo, rbo,
            render_texture: Texture::from_id(texture_id),
            render_width: width,
            screen_shader,
            screen_quad_vao, screen_quad_vbo,
            render_height: height,
            screen_size: (screen.width, screen.height)
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
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, 0 as *const _);
        gl::EnableVertexAttribArray(0);
        
        // uv
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, (2 * std::mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
        
        gl::BindVertexArray(0);
        
        (vao, vbo)
    }
}
pub fn update_screen_size(&mut self, screen: &Screen) {
    self.screen_size = (screen.width, screen.height)
}
pub fn begin_render(&self) {
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        gl::Viewport(0, 0, self.render_width, self.render_height);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
    }
}
pub fn end_scene_render(&self) {
        unsafe {
            // Bind default fbo
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, self.screen_size.0 as i32, self.screen_size.1 as i32);
            
            // Clear screen
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            self.render_scene_to_screen();
        }
    }
    fn render_scene_to_screen(&self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            self.screen_shader.use_shader();
            
            gl::ActiveTexture(gl::TEXTURE0);
            self.render_texture.use_texture();
            
            // Вычисляем размеры для сохранения пропорций пикселей
            let scale = self.calculate_pixel_perfect_scale();
            let render_width = (self.render_width as f32 * scale) as i32;
            let render_height = (self.render_height as f32 * scale) as i32;
            
            // Центрируем
            let offset_x = (self.screen_size.0 as i32 - render_width) / 2;
            let offset_y = (self.screen_size.1 as i32 - render_height) / 2;
            
            // Устанавливаем viewport только для области рендеринга
            gl::Viewport(offset_x as i32, offset_y as i32, render_width, render_height);
            
            // Рисуем квад
            gl::BindVertexArray(self.screen_quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
            
            // Возвращаем viewport на весь экран
            gl::Viewport(0, 0, self.screen_size.0 as i32, self.screen_size.1 as i32);
        }
    }
    fn calculate_pixel_perfect_scale(&self) -> f32 {
        // Ищем максимальный целый масштаб, который помещается в экран
        let max_scale_x = self.screen_size.0 as f32 / self.render_width as f32;
        let max_scale_y = self.screen_size.1 as f32 / self.render_height as f32;
        let max_scale = max_scale_x.min(max_scale_y).floor();
        
        // Минимальный масштаб = 1 (один пиксель сцены = один пиксель экрана)
        let scale = max_scale.max(1.0);
        
        scale
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteRenderbuffers(1, &self.rbo);
            gl::DeleteVertexArrays(1, &self.screen_quad_vao);
            gl::DeleteBuffers(1, &self.screen_quad_vbo);
        }
    }
}