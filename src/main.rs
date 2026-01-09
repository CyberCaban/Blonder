use ::log::info;
use anyhow::{Context as _, Result};
use glfw::Context;
use rand::Rng;

use crate::{
    events::process_events,
    log::setup_logger,
    models::{
        cube::{Cube, CubeSettings},
        plane::Plane,
        serpinsky::Serpinsky,
    },
    render::{
        color::Color,
        gui::{
            font::FontAtlas,
            text_renderer::{TextRenderParams, TextRenderer},
        },
        helpers::init_window,
        renderer::Renderer,
    },
    shader::Shader,
    state::State,
};

extern crate gl;
extern crate glfw;

mod camera;
mod controls;
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
    // renderer.add_default_shader(Shader::new(
    //     "assets/shaders/checkerboard/vert.glsl",
    //     "assets/shaders/checkerboard/frag.glsl",
    // )?);
    renderer.add_default_shader(Shader::new(
        "assets/shaders/light/vert.glsl",
        "assets/shaders/light/frag.glsl",
    )?);

    let mut serp = Serpinsky::new()?;
    let d = 5.0;
    // serp.triangle(&[-d, -d, 0.0], &[d, -d, 0.0], &[0., d, 0.0], 3);
    serp.make_coh(&[-d, -d, 0.0], &[d, -d, 0.0], &[0.0, d, 0.0], 2);
    serp.prepare();
    // let _ = renderer.add_drawable(serp);
    let mut rng = rand::thread_rng();
    let r = 10.0;
    let (low, high) = (-r, r);

    let texture_pool = [
        "assets/textures/cooler.png",
        "assets/textures/skebob.png",
        "assets/textures/white.png",
    ];

    let mut objIds = Vec::with_capacity(100);
    for _ in 0..1000 {
        let cube = Cube::new(CubeSettings {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            texture_name: texture_pool[rng.gen_range(0, texture_pool.len())],
            ..Default::default()
        })
        .unwrap();
        let id = renderer.add_dynamic_drawable(cube);
        if let Ok(id) = id
            && let Some(render_object) = renderer.get_transform_mut(&id)
            && let Some(tr) = render_object.get_transform_mut()
        {
            tr.position.x = rng.gen_range(low, high);
            tr.position.y = rng.gen_range(low, high);
            tr.position.z = rng.gen_range(low, high);
        }
        objIds.push(id);
    }

    let cube2 = Cube::new(CubeSettings {
        texture_name: "assets/textures/cooler.png",
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        ..Default::default()
    })?;
    let w = 10.0;
    let y = -1.0;
    let plane = Plane::new(
        [[w, y, w], [-w, y, w], [w, y, -w], [-w, y, -w]],
        [0.0, 0.0, 0.0],
    )?;
    let obj_id = renderer.add_dynamic_drawable(cube2);
    let _ = renderer.add_static_drawable(plane);
    if let Ok(id) = obj_id
        && let Some(render_object) = renderer.get_transform_mut(&id)
        && let Some(tr) = render_object.get_transform_mut()
    {
        tr.position.x = 1.0;
        // tr.scale.x = 0.1;
        // tr.scale.y = 0.1;
    }

    let mut text_renderer = TextRenderer::new(800.0, 600.0)?;
    let font_size = 48u32;
    let font_atlas = FontAtlas::new("assets/fonts/Montserrat-Regular.ttf", font_size)?;

    let mut frames = 0u32;
    let mut fps_count = String::new();
    let fps_render_params = TextRenderParams {
        scale: 1.0,
        color: Color::white(),
    };
    while !window.should_close() {
        glfw.poll_events();
        let current_frame = glfw.get_time() as f32;
        state.delta_time = current_frame - state.last_frame;
        state.last_frame = current_frame;

        frames += 1;

        process_events(&mut window, &events, &mut state);
        state.camera.process_input(&mut window, state.delta_time);

        if frames % 1 == 0 {
            for (i, obj) in objIds.iter().enumerate() {
                if let Ok(id) = obj
                    && let Some(render_object) = renderer.get_transform_mut(&id)
                    && let Some(tr) = render_object.get_transform_mut()
                {
                    if state.numbers[0] > 0.0 {
                        tr.scale.x = state.numbers[0];
                        tr.scale.y = state.numbers[0];
                        tr.scale.z = state.numbers[0];
                    }
                    tr.rotation.x = (glfw.get_time() as f32) * ((i % 10) as f32) + (i * 100) as f32;
                    tr.rotation.z = (glfw.get_time() as f32) * ((i % 5) as f32) + (i * 100) as f32;
                }
            }
        }

        renderer.render_checkerboard(&mut glfw, &state);
        text_renderer.render_text(
            &font_atlas,
            &fps_count,
            0.0,
            state.screen.height as f32 - font_size as f32 / 2.0 - 4.0,
            &state.screen,
            &TextRenderParams {
                scale: 1.0,
                color: Color::white(),
            },
        );
        if frames % 10 == 0 {
            fps_count = format!("FPS: {:.0}", 1.0 / state.delta_time);
        }
        window.swap_buffers();
    }
    Ok(())
}
