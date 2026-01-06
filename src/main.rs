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
        drawable::Drawable as _,
        helpers::{Mat4, Vec3, init_window},
        renderer::Renderer,
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

    let mut renderer = Renderer::new();

    renderer.add_shader(
        "camera",
        Shader::new(
            "assets/shaders/camera/vert.glsl",
            "assets/shaders/camera/frag.glsl",
        )?,
    );
    renderer.use_shader("camera")?;

    let mut serp = Serpinsky::new()?;
    let d = 0.9;
    serp.serp(&[-d, -d, 0.0], &[d, -d, 0.0], &[0., d, 0.0], 7);
    serp.prepare();

    let cube = Cube::new("assets/textures/transparency.png", &[0.0, 0.0, 0.0])?;
    let cube2 = Cube::new("assets/textures/cooler.png", &[1.0, 1.0, 1.0])?;
    renderer.add_drawable(cube);
    renderer.add_drawable(cube2);

    while !window.should_close() {
        glfw.poll_events();
        let current_frame = glfw.get_time() as f32;
        state.delta_time = current_frame - state.last_frame;
        state.last_frame = current_frame;

        process_events(&mut window, &events, &mut state);
        state.camera.process_input(&mut window, state.delta_time);

        renderer.render(&mut glfw, &state);

        window.swap_buffers();
    }
    Ok(())
}
