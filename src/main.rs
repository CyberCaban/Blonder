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
        renderer::{RenderMaterial, RenderObject, Renderer},
        transform::Transform,
    },
    shader::{Shader, ShaderInfo},
    state::{Screen, State},
    texture::TextureConfig,
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

    let mut renderer = Renderer::new()?;

    renderer.add_default_shader(Shader::new(
        "assets/shaders/light/vert.glsl",
        "assets/shaders/light/frag.glsl",
    )?);

    let mut rng = rand::thread_rng();
    let r = 10.0;
    let (low, high) = (-r, r);

    let texture_pool = [
        ("assets/textures/transparency.png", true),
        ("assets/textures/skebob.png", false),
        ("assets/textures/white.png", false),
    ];

    let mut objIds = Vec::with_capacity(100);
    for _ in 0..1000 {
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
            texture_name: texture.0,
            texture_config: TextureConfig {
                texture_filtering: if texture.1 { gl::NEAREST } else { gl::LINEAR } as i32,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let transform = Transform::new(
            Some(position.into()),
            Some(rotation.into()),
            Some(Vector3::from_value(1.0)),
        );

        let render_object = RenderObject {
            drawable: Box::new(cube),
            material: Some(RenderMaterial {
                // specular: Some("assets/textures/specular.png".to_string()),
                specular: Some(texture.0.to_string()),
                // emission: Some("assets/textures/emission.jpg".to_string()),
                emission: None,
                shininess: 32.0,
            }),
            transform: Some(transform),
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
        transform: Some(Transform::new(None, None, Some(Vector3::from_value(0.5)))),
        material: Some(RenderMaterial::default()),
    };
    let light_id = renderer.add_render_object(light_ro)?;

    let w = 10.0;
    let y = -1.0;
    let plane = Plane::new(
        [[w, y, w], [-w, y, w], [w, y, -w], [-w, y, -w]],
        [0.0, 0.0, 0.0],
    )?;
    let _ = renderer.add_static_drawable(plane);


    let mut frames = 0u32;
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
                    tr.set_scale(Vector3::from_value(1.0));
                    // tr.scale.x = state.numbers[0];
                    // tr.scale.y = state.numbers[0];
                    // tr.scale.z = state.numbers[0];
                    let r = tr.get_rotation();
                    tr.set_rotation(Vector3::new(r.x, state.numbers[2], r.z));
                    // tr.rotation.x = (glfw.get_time() as f32) * ((i % 10) as f32) + (i * 100) as f32;
                    // tr.rotation.z = (glfw.get_time() as f32) * ((i % 5) as f32) + (i * 100) as f32;
                }
            }
        }
        if let Some(ro) = renderer.get_transform_mut(&light_id)
            && let Some(tr) = ro.get_transform_mut()
        {
            tr.set_position(state.light_pos);
        }

        renderer.render_checkerboard(&mut glfw, &state);

        window.swap_buffers();
    }
    Ok(())
}
