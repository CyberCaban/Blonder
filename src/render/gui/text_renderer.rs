use anyhow::Result;
use cgmath::{Matrix4, Vector3, Vector4};
use num::Zero;

use crate::{
    render::{color::Color, gui::font::FontAtlas},
    shader::Shader,
    state::Screen,
};

#[derive(Debug, Default)]
pub struct TextRenderParams {
    pub scale: f32,
    pub color: Color,
}

pub struct TextRenderer {
    vao: u32,
    vbo: u32,
    shader_program: Shader,
    projection_matrix: Matrix4<f32>,
}

impl TextRenderer {
    pub fn new(width: f32, height: f32) -> Result<Self> {
        let (vao, vbo) = TextRenderer::setup_buffers()?;
        let shader = Shader::new(
            "assets/shaders/text/vert.glsl",
            "assets/shaders/text/frag.glsl",
        )?;

        let mut renderer = TextRenderer {
            vao,
            vbo,
            shader_program: shader,
            projection_matrix: Matrix4::zero(),
        };
        renderer.set_projection(width, height);
        Ok(renderer)
    }
    fn setup_buffers() -> Result<(u32, u32)> {
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Буфер для 6 вершин (2 треугольника) на символ
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 4 * std::mem::size_of::<f32>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Позиция
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );

            // Текстурные координаты
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                (2 * std::mem::size_of::<f32>()) as *const _,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok((vao, vbo))
    }
    pub fn set_projection(&mut self, width: f32, height: f32) {
        let left = 0.0;
        let right = width;
        let bottom = 0.0;
        let top = height;
        let near = -1.0;
        let far = 1.0;

        #[rustfmt::skip]
        let projection_matrix = Matrix4::new(
            2.0 / (right - left), 0.0, 0.0, 0.0,
            0.0, 2.0 / (top - bottom), 0.0, 0.0,
            0.0, 0.0, -2.0 / (far - near), 0.0,
            -(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near) / (far - near), 1.0,
        );
        self.projection_matrix = projection_matrix;
    }
    pub fn render_text(
        &mut self,
        font_atlas: &FontAtlas,
        text: &str,
        x: f32,
        y: f32,
        screen: &Screen,
        render_params: &TextRenderParams,
    ) {
        let TextRenderParams { color, scale, .. } = render_params;
        unsafe {
            self.shader_program.use_shader();
            self.set_projection(screen.width as f32, screen.height as f32);
            self.shader_program
                .set_mat4("projection", &self.projection_matrix);

            self.shader_program.set_vec4(
                "uTextColor",
                &Vector4::new(color[0], color[1], color[2], color[3]),
            );

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ActiveTexture(gl::TEXTURE0);

            gl::BindVertexArray(self.vao);

            let mut cursor_x = x;
            let cursor_y = y;

            for ch in text.chars() {
                if let Some(character) = font_atlas.get_character(ch) {
                    if character.size.0 == 0.0 || character.size.1 == 0.0 {
                        cursor_x += character.advance * scale;
                        continue;
                    }
                    let xpos = cursor_x + character.bearing.0 * scale;
                    let ypos = cursor_y - (character.size.1 - character.bearing.1) * scale;

                    let w = character.size.0 * scale;
                    let h = character.size.1 * scale;

                    // Обновляем VBO для текущего символа
                    #[rustfmt::skip]
                    let vertices: [f32; 24] = [
                        // Позиция       // Текстурные координаты
                        xpos, ypos + h, 0.0, 0.0,
                        xpos, ypos, 0.0, 1.0,
                        xpos + w, ypos, 1.0, 1.0,
                        xpos, ypos + h, 0.0, 0.0,
                        xpos + w, ypos, 1.0, 1.0,
                        xpos + w, ypos + h, 1.0, 0.0,
                    ];

                    character.texture_id.use_texture();
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        (vertices.len() * std::mem::size_of::<f32>()) as isize,
                        vertices.as_ptr() as *const _,
                    );
                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);

                    gl::DrawArrays(gl::TRIANGLES, 0, 6);

                    cursor_x += character.advance * scale;
                } else {
                    cursor_x += font_atlas.size as f32 * 0.5 * scale;
                }
            }

            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::Disable(gl::BLEND);
        }
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
