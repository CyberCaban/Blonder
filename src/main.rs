use ::log::info;
use anyhow::{Context as _, Result};
use cgmath::{Array, Vector3};
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
        consts::{HEIGHT, WIDTH},
        framebuffer::Framebuffer,
        gui::{
            font::FontAtlas,
            text_renderer::{TextRenderParams, TextRenderer},
        },
        helpers::init_window,
        material::Material,
        renderer::{RenderMaterial, RenderObject, Renderer},
        transform::Transform,
    },
    shader::{Shader, ShaderInfo},
    state::{Screen, State},
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

    // let mut serp = Serpinsky::new()?;
    let d = 5.0;
    // serp.triangle(&[-d, -d, 0.0], &[d, -d, 0.0], &[0., d, 0.0], 3);
    // serp.make_coh(&[-d, -d, 0.0], &[d, -d, 0.0], &[0.0, d, 0.0], 2);
    // serp.prepare();
    // let _ = renderer.add_drawable(serp);
    let mut rng = rand::thread_rng();
    let r = 10.0;
    let (low, high) = (-r, r);

    let texture_pool = [
        "assets/textures/transparency.png",
        "assets/textures/skebob.png",
        "assets/textures/white.png",
    ];

    let mut objIds = Vec::with_capacity(100);
    for _ in 0..100 {
        let position = [
            rng.gen_range(low, high),
            rng.gen_range(low, high),
            rng.gen_range(low, high),
        ];

        let rotation = [
            rng.r#gen::<f32>() * 360.0,
            rng.r#gen::<f32>() * 360.0,
            rng.r#gen::<f32>() * 360.0,
        ];
        let texture = texture_pool[rng.gen_range(0, texture_pool.len())];
        let cube = Cube::new(CubeSettings {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            texture_name: texture,
            ..Default::default()
        })
        .unwrap();
        let transform = Transform {
            position: position.into(),
            rotation: rotation.into(),
            ..Default::default()
        };

        let render_object = RenderObject {
            drawable: Box::new(cube),
            material: Some(RenderMaterial {
                specular: Some("assets/textures/specular.png".to_string()),
                // specular: Some(texture.to_string()),
                // emission: Some("assets/textures/emission.jpg".to_string()),
                emission: None,
                shininess: 32.0,
            }),
            transform: Some(transform),
            is_dynamic: true,
        };
        let id = renderer.add_render_object(render_object);
        objIds.push(id);
    }

    let light_src = Cube::new(CubeSettings {
        texture_name: "assets/textures/white.png",
        shader_name: ShaderInfo {
            name: "light_cube".to_string(),
            vertex_path: "assets/shaders/light_cube/vert.glsl".to_string(),
            fragment_path: "assets/shaders/light_cube/frag.glsl".to_string(),
        },
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        ..Default::default()
    })?;
    let light_ro = RenderObject {
        drawable: Box::new(light_src),
        transform: Some(Transform {
            scale: Vector3::from_value(0.5),
            ..Default::default()
        }),
        material: Some(RenderMaterial::default()),
        is_dynamic: true,
    };
    let light_id = renderer.add_render_object(light_ro)?;

    let w = 10.0;
    let y = -1.0;
    let plane = Plane::new(
        [[w, y, w], [-w, y, w], [w, y, -w], [-w, y, -w]],
        [0.0, 0.0, 0.0],
    )?;
    let _ = renderer.add_static_drawable(plane);

    let mut text_renderer = TextRenderer::new(800.0, 600.0)?;
    let font_size = 48u32;
    let font_atlas = FontAtlas::new("assets/fonts/OpenSans.ttf", font_size)?;

    let mut framebuffer = Framebuffer::new(
        480, 360,
        &state::Screen {
            width: WIDTH,
            height: HEIGHT,
        },
    )?;

    let mut frames = 0u32;
    let mut fps_count = String::new();
    let mut fps_y = 0f32;
    while !window.should_close() {
        glfw.poll_events();
        let current_frame = glfw.get_time() as f32;
        state.delta_time = current_frame - state.last_frame;
        state.last_frame = current_frame;

        frames += 1;

        process_events(&mut window, &events, &mut state);
        state.camera.process_input(&mut window, state.delta_time);

        if frames % 2 == 0 {
            for (i, obj) in objIds.iter().enumerate() {
                if let Ok(id) = obj
                    && let Some(render_object) = renderer.get_transform_mut(id)
                    && let Some(tr) = render_object.get_transform_mut()
                {
                    tr.scale = Vector3::from_value(1.0);
                    // tr.scale.x = state.numbers[0];
                    // tr.scale.y = state.numbers[0];
                    // tr.scale.z = state.numbers[0];
                    tr.rotation.y = state.numbers[2];
                    // tr.rotation.x = (glfw.get_time() as f32) * ((i % 10) as f32) + (i * 100) as f32;
                    // tr.rotation.z = (glfw.get_time() as f32) * ((i % 5) as f32) + (i * 100) as f32;
                }
            }
        }
        if let Some(ro) = renderer.get_transform_mut(&light_id)
            && let Some(tr) = ro.get_transform_mut()
        {
            tr.position = state.light_pos;
        }

        framebuffer.begin_render();
        renderer.render_checkerboard(&mut glfw, &state);
        text_renderer.render_text(
            &font_atlas,
            &fps_count,
            0.0,
            framebuffer.render_height as f32 - (font_size as f32 / 2.0),
            &Screen {
                width: framebuffer.render_width as u32,
                height: framebuffer.render_height as u32,
            },
            &TextRenderParams {
                scale: 1.0,
                color: Color::white(),
            },
        );
        text_renderer.render_text(
            &font_atlas,
            &format!("{}", frames),
            0.0,
            fps_y - ((font_size as f32 / 2.0) - 4.0) * 2.0,
            &state.screen,
            &TextRenderParams {
                scale: 1.0,
                color: Color::white(),
            },
        );
        if frames % 10 == 0 {
            fps_count = format!("FPS: {:.0}", 1.0 / state.delta_time);
        }
        framebuffer.end_scene_render();
        framebuffer.update_screen_size(&state.screen);
        window.swap_buffers();
    }
    Ok(())
}
