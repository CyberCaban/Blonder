use std::{mem::offset_of, os::raw::c_void, ptr};

use ::log::info;
use anyhow::{Context as _, Result};
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3, ortho, perspective};
use gl::types::{GLsizei, GLsizeiptr};
use glfw::{Context, Glfw, PWindow};

use crate::{
    events::process_events,
    log::setup_logger,
    models::{cube::Cube, serpinsky::Serpinsky},
    render::{
        helpers::{Mat4, Vec3, init_window},
    },
    shader::Shader,
    state::{Screen, State},
    texture::Texture,
};

extern crate gl;
extern crate glfw;

mod camera;
mod events;
mod log;
mod models;
mod render;
mod shader;
mod state;
mod texture;

fn main() -> Result<()> {
    setup_logger()?;
    info!("Hello, world!");
    let mut glfw = glfw::init_no_callbacks().context("Failed to init glfw")?;
    let (mut window, events) = init_window(&mut glfw)?;
    let mut state = State::default();

    let shader_program = vec![
        Shader::new(
            "assets/shaders/camera/vert.glsl",
            "assets/shaders/camera/frag.glsl",
        )?,
        Shader::new(
            "assets/shaders/vert_tex.glsl",
            "assets/shaders/frag_tex.glsl",
        )?,
        Shader::new("assets/shaders/vert.glsl", "assets/shaders/frag.glsl")?,
    ];
    let texture = [
        Texture::new("assets/textures/liminal_space.png")?,
        Texture::new("assets/textures/cooler.png")?,
    ];

    let mut serp = Serpinsky::new()?;
    let d = 0.9;
    serp.serp(&[-d, -d, 0.0], &[d, -d, 0.0], &[0., d, 0.0], 7);
    serp.prepare();

    let cube = Cube::new("assets/textures/transparency.png", &[0.0, 0.0, 0.0])?;
    let cube2 = Cube::new("assets/textures/cooler.png", &[1.0, 1.0, 1.0])?;

    // let projection_matrix = ortho(
    //     -(aspect as f32) * 2.0,
    //     aspect as f32 * 2.0,
    //     -2.0,
    //     2.0,
    //     -10.0,
    //     10.0,
    // );
    while !window.should_close() {
        glfw.poll_events();
        let current_frame = glfw.get_time() as f32;
        state.delta_time = current_frame - state.last_frame;
        state.last_frame = current_frame;

        process_events(&mut window, &events, &mut state);
        state.camera.process_input(&mut window, state.delta_time);

        let State {
            color,
            wireframe,
            screen: Screen { width, height },
            ..
        } = state;
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // configurable parameters
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if wireframe { gl::LINE } else { gl::FILL },
            );

            // Draw calls and such
            shader_program[0].use_shader();
            let aspect = if state.screen.height > 0 {
                state.screen.width as f32 / state.screen.height as f32
            } else {
                1.0
            };
            let model_matrix =
                Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0).normalize(), Rad(0.0));
            let view_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, -3.0));
            let projection_matrix = perspective(Deg(45.0), aspect, 0.01, 100.0);

            let view_matrix = state.camera.view_matrix();
            let mvp = projection_matrix * view_matrix * model_matrix;
            shader_program[0].set_mat4("mvp", &mvp);

            gl::ActiveTexture(gl::TEXTURE0);
            texture[0].use_texture();
            gl::ActiveTexture(gl::TEXTURE1);
            texture[1].use_texture();

            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            cube.draw(&glfw, &state);
            cube2.draw(&glfw, &state);
        }

        window.swap_buffers();
    }
    Ok(())
}
