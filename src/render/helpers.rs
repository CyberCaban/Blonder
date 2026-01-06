use std::{mem::offset_of, os::raw::c_void};

use anyhow::{Context as _, Result};
use gl::types::{GLsizei, GLsizeiptr};
use glfw::{Context, Glfw, PWindow};

use crate::{
    render::{
        consts::{HEIGHT, WIDTH},
        vertex::Vertex,
    },
    state::Events,
};

pub type Mat4 = cgmath::Matrix4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;

pub fn init_window(glfw: &mut Glfw) -> Result<(PWindow, Events)> {
    // MSAA x2/4/8
    // glfw.window_hint(glfw::WindowHint::Samples(Some(8)));
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.set_swap_interval(glfw::SwapInterval::None);
    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "Hello", glfw::WindowMode::Windowed)
        .context("Failed to create window")?;
    window.make_current();
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);
    window.set_drag_and_drop_polling(true);
    window.set_framebuffer_size_polling(true);
    gl::load_with(|symbol| {
        window
            .get_proc_address(symbol)
            .map(|ptr| ptr as *const c_void)
            .unwrap_or(std::ptr::null())
    });
    unsafe {
        // depth buffer
        gl::Enable(gl::DEPTH_TEST);
        // backface culling
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
        // texture blending
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::BlendEquation(gl::FUNC_ADD);
    }
    Ok((window, events))
}

pub fn set_buffer_data(vao: u32, vbo: u32, data: &[Vertex]) {
    unsafe {
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
            &data[0] as *const _ as *const c_void,
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
            4,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, color) as *const c_void,
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

pub fn set_buffer_data_with_indices(
    vao: u32,
    vbo: u32,
    ebo: u32,
    data: &[Vertex],
    indices: &[u32],
) {
    unsafe {
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
            &indices[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
            &data[0] as *const _ as *const c_void,
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
            4,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, color) as *const c_void,
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
