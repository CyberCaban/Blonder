use std::{mem::offset_of, os::raw::c_void, ptr};

use ::log::info;
use anyhow::{Context as _, Result};
use cgmath::{Deg, Rad, SquareMatrix, perspective};
use gl::types::{GLsizei, GLsizeiptr};
use glfw::{Context, Glfw, PWindow};

use crate::{
    events::process_events,
    log::setup_logger,
    models::{cube::Cube, serpinsky::Serpinsky},
    render::{
        helpers::{Mat4, Vec3, init_window, set_buffer_data},
        vertex::Vertex,
    },
    shader::Shader,
    state::{Events, Screen, State},
    texture::Texture,
};

extern crate gl;
extern crate glfw;

mod events;
mod log;
mod models;
mod render;
mod shader;
mod state;
mod texture;

const VERTICES_NUM: i32 = 4;
const VERTICES_SIZE: i32 = 3;
#[rustfmt::skip]
const VERTICES: [Vertex; (VERTICES_NUM) as usize] = [
    // first rect
    Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [1.0, 1.0] }, // 0
    Vertex { position: [0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [1.0, 0.0] }, // 1
    Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [0.0, 0.0] }, // 2
    Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [0.0, 1.0] }, // 3
];
const INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];

fn main() -> Result<()> {
    setup_logger()?;
    info!("Hello, world!");
    let mut glfw = glfw::init_no_callbacks().context("Failed to init glfw")?;
    let (mut window, events) = init_window(&mut glfw)?;
    let mut state = State::default();

    let shader_program = vec![
        Shader::new("shaders/camera/vert.glsl", "shaders/camera/frag.glsl")?,
        Shader::new("shaders/vert_tex.glsl", "shaders/frag_tex.glsl")?,
        Shader::new("shaders/vert.glsl", "shaders/frag.glsl")?,
    ];
    let texture = [
        Texture::new("textures/liminal_space.png")?,
        Texture::new("textures/cooler.png")?,
    ];

    let mut serp = Serpinsky::new()?;
    let d = 0.9;
    serp.serp(&[-d, -d, 0.0], &[d, -d, 0.0], &[0., d, 0.0], 7);
    serp.prepare();

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
        set_buffer_data(vao[1], vbo[1], &triangle1);
        // third shape
        set_buffer_data(vao[2], vbo[2], &triangle2);

        // unbinding
        // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        // gl::BindVertexArray(0);

        vao
    };
    let cube = Cube::new("textures/white.png")?;

    while !window.should_close() {
        process_events(&mut window, &events, &mut state);

        let State {
            color,
            wireframe,
            screen: Screen { width, height },
            ..
        } = state;
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
            let projection_matrix = perspective(Deg(35.0), (width / height) as f32, 0.01, 100.0);
            let model_matrix = Mat4::from_angle_x(Deg(-55.0))
                * Mat4::from_axis_angle(
                    Vec3::new(0.0, 1.0, 0.0),
                    Rad(1.0) * glfw.get_time() as f32,
                );
            let view_matrix = Mat4::from_translation(Vec3::unit_z() * -3.0);
            shader_program[0].set_mat4("model", &model_matrix);
            shader_program[0].set_mat4("view", &view_matrix);
            shader_program[0].set_mat4("projection", &projection_matrix);
            gl::ActiveTexture(gl::TEXTURE0);
            texture[0].use_texture();
            gl::ActiveTexture(gl::TEXTURE1);
            texture[1].use_texture();
            gl::BindVertexArray(vao[0]);
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            
            // Texture::use_empty_texture();
            // shader_program[1].use_shader();
            gl::BindVertexArray(vao[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(vao[2]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            cube.draw();
        }

        window.swap_buffers();
        glfw.poll_events();
    }
    Ok(())
}
