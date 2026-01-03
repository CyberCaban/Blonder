use std::{ffi::CString, mem::offset_of, os::raw::c_void, ptr};

use ::log::{error, info};
use anyhow::{Context as _, Result};
use gl::types::{GLchar, GLint, GLsizei, GLsizeiptr};
use glfw::{Context, Glfw, PWindow};

use crate::{
    events::process_events, log::setup_logger, shader::Shader, state::{Events, State}, texture::Texture
};

extern crate gl;
extern crate glfw;

mod events;
mod log;
mod state;
mod shader;
mod texture;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 3],
}

const VERTICES_NUM: i32 = 4;
const VERTICES_SIZE: i32 = 3;
#[rustfmt::skip]
const VERTICES: [Vertex; (VERTICES_NUM) as usize] = [
    // first rect
    Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 0.0, 0.0], uv: [1.0, 1.0] }, // 0
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0], uv: [1.0, 0.0] }, // 1
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0] }, // 2
    Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 0.0, 1.0], uv: [0.0, 1.0] }, // 3
];
const INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];

fn init_window(glfw: &mut Glfw) -> Result<(PWindow, Events)> {
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
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
    Ok((window, events))
}

fn set_buffer_data(vao: u32, vbo: u32, data: &[Vertex], data_size: u32) {
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
            data_size as i32,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            data_size as i32,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, color) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
    }
}

fn main() -> Result<()> {
    setup_logger()?;
    info!("Hello, world!");
    let mut glfw = glfw::init_no_callbacks().context("Failed to init glfw")?;
    let (mut window, events) = init_window(&mut glfw)?;
    let mut state = State::default();


    let shader_program = vec![
        Shader::new("shaders/vert_tex.glsl", "shaders/frag_tex.glsl")?,
        Shader::new("shaders/vert.glsl", "shaders/frag.glsl")?,
        ];
    let texture = Texture::new("textures/skebob.png")?;
    let vao = unsafe {
        #[rustfmt::skip]
        let triangle1: [Vertex; 3] = [
            Vertex { position: [-0.6, -0.6, 0.0], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },// 4
            Vertex { position: [0.7, -0.7, 0.0], color: [0.0, 0.5, 0.3], uv: [1.0, 0.0] },// 5
            Vertex { position: [0.0, -0.9, 0.0], color: [0.3, 0.0, 0.4], uv: [1.0, 1.0] }, // 6
        ];
        #[rustfmt::skip]
        let triangle2: [Vertex; 3] = [
            Vertex { position: [0.6, 0.6, 0.0], color: [0.0, 0.6, 0.0], uv: [0.0, 0.0] }, // 7
            Vertex { position: [0.6, -0.6, 0.0], color: [0.0, 0.3, 0.8], uv: [0.0, 0.0] }, // 8
            Vertex { position: [1.0, 0.0, 0.0], color: [0.0, 0.2, 0.6], uv: [0.0, 0.0] }, // 9
        ];

        // first shape
        let (mut vbo, mut vao, mut ebo) = ([0, 0, 0], [0, 0, 0], 0);
        gl::GenVertexArrays(3, vao.as_mut_ptr());
        gl::GenBuffers(3, vbo.as_mut_ptr());
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao[0]);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTICES.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
            &VERTICES[0] as *const _ as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDICES.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
            &INDICES[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );

        // position
        gl::VertexAttribPointer(
            0,
            VERTICES_SIZE,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);

        // color
        gl::VertexAttribPointer(
            1,
            VERTICES_SIZE,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, color) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        // texture
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<Vertex>()) as GLsizei,
            offset_of!(Vertex, uv) as *const c_void,
        );
        gl::EnableVertexAttribArray(2);

        // second shape
        set_buffer_data(vao[1], vbo[1], &triangle1, 3);
        // third shape
        set_buffer_data(vao[2], vbo[2], &triangle2, 3);

        // unbinding
        // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        // gl::BindVertexArray(0);

        vao
    };

    while !window.should_close() {
        process_events(&mut window, &events, &mut state);

        let State { color, wireframe, .. } = state;
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // configurable parameters
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if wireframe { gl::LINE } else { gl::FILL },
            );

            // Draw calls and such
            shader_program[0].use_shader();
            shader_program[0].set_float("xpos", color.3);
            texture.use_texture();
            gl::BindVertexArray(vao[0]);
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );

            Texture::use_empty_texture();
            shader_program[1].use_shader();
            gl::BindVertexArray(vao[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(vao[2]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
    Ok(())
}
