use anyhow::Result;
use std::{
    ffi::c_void, mem::{self, offset_of}, ptr
};

use cgmath::{Matrix, Matrix4};
use gl::types::{GLsizei, GLsizeiptr};
use num::Zero;

use crate::{
    render::{color::Color, renderer::TextureRef},
    shader::Shader,
    texture::Texture,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UIVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

const MAX_INDICES: usize = 1024;
const MAX_VERTICES: usize = 512;

pub struct UIRenderer {
    shader: Shader,
    white_texture: Texture,
    projection_matrix: Matrix4<f32>,
    screen_width: f32,
    screen_height: f32,
    vertex_data: Vec<UIVertex>,
    index_data: Vec<u32>,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl UIRenderer {
    pub fn new() -> Result<Self> {
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }
        let white_texture = Texture::white();
        let renderer = Self {
            white_texture,
            shader: Shader::new("assets/shaders/ui/vert.glsl", "assets/shaders/ui/frag.glsl")?,
            projection_matrix: Matrix4::zero(),
            screen_height: 0.0,
            screen_width: 0.0,
            vertex_data: Vec::with_capacity(MAX_VERTICES),
            index_data: Vec::with_capacity(MAX_INDICES),
            vao,
            vbo,
            ebo,
        };
        renderer.setup_buffers();
        Ok(renderer)
    }
    fn setup_buffers(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<UIVertex>() * MAX_VERTICES) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * MAX_INDICES) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            let stride = mem::size_of::<UIVertex>() as GLsizei;
            // position
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(UIVertex, position) as *const c_void,
            );
            gl::EnableVertexAttribArray(0);
            // color
            gl::VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(UIVertex, color) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);
            // uv
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(UIVertex, uv) as *const c_void,
            );
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);
        }
    }
    fn upload_buf_data(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.vertex_data.len() * mem::size_of::<UIVertex>()) as GLsizeiptr,
                self.vertex_data.as_ptr() as *const _,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                0,
                (self.index_data.len() * mem::size_of::<u32>()) as GLsizeiptr,
                self.index_data.as_ptr() as *const _,
            );
        }
    }
    pub fn update_projection(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
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
    pub fn begin_frame(&mut self) {
        self.vertex_data.clear();
        self.index_data.clear();

        self.shader.use_shader();

        unsafe {
            self.shader.set_mat4("projection", &self.projection_matrix);

            gl::ActiveTexture(gl::TEXTURE0);
            self.white_texture.use_texture();
            // self.shader.set_int("texture1", 0);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }
    }
    pub fn end_frame(&self) {
        if !self.vertex_data.is_empty() && !self.index_data.is_empty() {
            unsafe {
                gl::BindVertexArray(self.vao);
                self.upload_buf_data();
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.index_data.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
                gl::BindVertexArray(0);

                gl::Disable(gl::BLEND);
                gl::Enable(gl::DEPTH_TEST);
            }
        }
    }
    pub fn draw_texture(
        &mut self,
        texture: TextureRef,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &Color,
    ) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            texture.use_texture();
            let start_index = self.vertex_data.len() as u32;
            let color = color.as_array();

            let vertices = [
                UIVertex {
                    position: [x, y],
                    color,
                    uv: [0.0, 0.0],
                },
                UIVertex {
                    position: [x + width, y],
                    color,
                    uv: [1.0, 0.0],
                },
                UIVertex {
                    position: [x + width, y + height],
                    color,
                    uv: [1.0, 1.0],
                },
                UIVertex {
                    position: [x, y + height],
                    color,
                    uv: [0.0, 1.0],
                },
            ];

            let indices = [
                start_index,
                start_index + 1,
                start_index + 2,
                start_index,
                start_index + 2,
                start_index + 3,
            ];

            self.vertex_data.extend_from_slice(&vertices);
            self.index_data.extend_from_slice(&indices);
        }
    }
    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: &Color) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            self.white_texture.use_texture();
        }
        let start_index = self.vertex_data.len() as u32;
        let color = color.as_array();

        let vertices = [
            UIVertex {
                position: [x, y],
                color,
                uv: [-1.0, -1.0],
            },
            UIVertex {
                position: [x + width, y],
                color,
                uv: [-1.0, -1.0],
            },
            UIVertex {
                position: [x + width, y + height],
                color,
                uv: [-1.0, -1.0],
            },
            UIVertex {
                position: [x, y + height],
                color,
                uv: [-1.0, -1.0],
            },
        ];

        let indices = [
            start_index,
            start_index + 1,
            start_index + 2,
            start_index,
            start_index + 2,
            start_index + 3,
        ];

        self.vertex_data.extend_from_slice(&vertices);
        self.index_data.extend_from_slice(&indices);
    }
}
