use ::log::info;
use anyhow::{Context as _, Result};
use glfw::Context;
use rand::Rng;

use crate::{
    events::process_events,
    log::setup_logger,
    models::{
        cube::{Cube, CubeSettings},
        serpinsky::Serpinsky,
    },
    render::{
        gui::{
            font::FontAtlas,
            text_renderer::{self, TextRenderer},
        },
        helpers::init_window,
        renderer::Renderer,
    },
    shader::{Shader, ShaderInfo},
    state::State,
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
    state.color.1 = 0.6;
    state.color.2 = 0.6;

    let mut renderer = Renderer::new();

    // renderer.add_default_shader(Shader::new(
    //     "assets/shaders/camera/vert.glsl",
    //     "assets/shaders/camera/frag.glsl",
    // )?);
    renderer.add_default_shader(Shader::new(
        "assets/shaders/checkerboard/vert.glsl",
        "assets/shaders/checkerboard/frag.glsl",
    )?);
    // renderer.add_default_shader(Shader::new(
    //     "assets/shaders/light/vert.glsl",
    //     "assets/shaders/light/frag.glsl",
    // )?);

    let mut serp = Serpinsky::new()?;
    let d = 20.0;
    serp.serp(&[-d, -d, 0.0], &[d, -d, 0.0], &[0., d, 0.0], 3);
    serp.prepare();
    // let _ = renderer.add_drawable(serp);
    let mut rng = rand::thread_rng();
    let w = 5.0;
    let (low, high) = (-w, w);

    let texture_pool = [
        "assets/textures/transparency.png",
        "assets/textures/cooler.png",
        "assets/textures/liminal_space.png",
        "assets/textures/white.png",
    ];

    for _ in 0..1_0 {
        let cube = Cube::new(CubeSettings {
            position: [
                rng.gen_range(low, high),
                rng.gen_range(low, high),
                rng.gen_range(low, high),
            ],
            rotation: [
                rng.gen_range(low, high),
                rng.gen_range(low, high),
                rng.gen_range(low, high),
            ],
            texture_name: texture_pool[rng.gen_range(0, texture_pool.len())],
            ..Default::default()
        })
        .unwrap();
        let _ = renderer.add_drawable(cube);
    }

    let cube = Cube::new(CubeSettings {
        texture_name: "assets/textures/transparency.png",
        ..Default::default()
    })?;
    let cube2 = Cube::new(CubeSettings {
        texture_name: "assets/textures/cooler.png",
        position: [1.0, 0.0, 0.0],
        rotation: [3.0, 1.0, 0.0],
        ..Default::default()
    })?;
    // let _ = renderer.add_drawable(cube);
    // let _ = renderer.add_drawable(cube2);

    let mut text_renderer = TextRenderer::new(800.0, 600.0)?;
    let font_atlas = FontAtlas::new("assets/fonts/Montserrat-Regular.ttf", 48)?;

    while !window.should_close() {
        glfw.poll_events();
        let current_frame = glfw.get_time() as f32;
        state.delta_time = current_frame - state.last_frame;
        state.last_frame = current_frame;

        process_events(&mut window, &events, &mut state);
        state.camera.process_input(&mut window, state.delta_time);

        renderer.render_checkerboard(&mut glfw, &state);
        text_renderer.render_text(
            &font_atlas,
            "Hello world",
            100.0,
            100.0,
            1.0,
            (0.0, 0.0, 1.0),
        );

        window.swap_buffers();
    }
    Ok(())
}
