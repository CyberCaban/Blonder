use std::{ffi::c_void, mem::{self, offset_of}, ptr};
use anyhow::Result;

use cgmath::Matrix4;
use gl::types::{GLsizei, GLsizeiptr};
use num::Zero;

use crate::{
    render::{helpers::set_buffer_data_with_indices, vertex::Vertex},
    shader::Shader,
};

const MAX_INDICES: usize = 1024;
const MAX_VERTICES: usize = 512;

pub struct UIRenderer {
    shader: Shader,
    projection_matrix: Matrix4<f32>,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl UIRenderer {
    pub fn new() -> Result<Self> {
        let (mut vao, mut vbo, mut ebo) = (00, 00, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }
        Ok(Self {
            shader: Shader::new("assets/shaders/ui/vert.glsl", "assets/shaders/ui/frag.glsl")?,
            projection_matrix: Matrix4::zero(),
            vao,
            vbo,
            ebo,
        })
    }
    fn setup_buffers(vao: u32, vbo: u32, ebo: u32) {
        unsafe {
            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * MAX_INDICES) as GLsizeiptr,
                ptr::null(),
                gl::STATIC_DRAW,
            );
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<u32>() * MAX_VERTICES) as GLsizeiptr,
                ptr::null(),
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (std::mem::size_of::<Vertex>()) as GLsizei,
                offset_of!(Vertex, position) as *const c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (std::mem::size_of::<Vertex>()) as GLsizei,
                offset_of!(Vertex, normal) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                (std::mem::size_of::<Vertex>()) as GLsizei,
                offset_of!(Vertex, uv) as *const c_void,
            );
            gl::EnableVertexAttribArray(2);
        }
    }
}
