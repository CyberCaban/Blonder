use std::{thread, time::Duration};

use ::log::info;
use anyhow::{Context as _, Result};
use cgmath::{Angle, Array, Deg, InnerSpace, Vector3};
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
        blend_mode::BlendMode,
        helpers::init_window,
        light::{DirLight, PointLight, SpotLight},
        renderer::{RenderMaterial, RenderObject, Renderer},
        shader::{Shader, ShaderInfo},
        transform::Transform,
    },
    state::State,
    texture::TextureConfig,
};

extern crate gl;
extern crate glfw;

mod camera;
mod events;
mod log;
mod models;
mod render;
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
        // ("assets/textures/transparency.png", true),
        // ("assets/textures/colored_glass.png", true),
        ("assets/textures/liminal_space.png", false),
        ("assets/textures/white.png", false),
    ];

    let mut objIds = Vec::with_capacity(100);
    for _ in 0..10 {
        let position = [
            rng.gen_range(low, high),
            rng.gen_range(low, high),
            rng.gen_range(low, high),
        ];

        let rotation = [
            0.0, // rng.r#gen::<f32>() * 360.0,
            rng.r#gen::<f32>() * 360.0,
            0.0, // rng.r#gen::<f32>() * 360.0,
        ];
        let texture = texture_pool[rng.gen_range(0, texture_pool.len())];
        let cube = Cube::new(CubeSettings {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            texture_name: texture.0,
            texture_config: TextureConfig {
                texture_filtering: if texture.1 { gl::NEAREST } else { gl::LINEAR } as i32,
                mipmap_filtering: if texture.1 {
                    gl::NEAREST
                } else {
                    gl::LINEAR_MIPMAP_LINEAR
                } as i32,
                ..Default::default()
            },
            blend_mode: if texture.1 {
                BlendMode::AlphaTest
            } else {
                BlendMode::Opaque
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
            material: RenderMaterial {
                // specular: Some("assets/textures/specular.png".to_string()),
                // specular: Some(texture.0.to_string()),
                specular: None,
                // emission: Some("assets/textures/emission.jpg".to_string()),
                emission: None,
                shininess: 128.0,
            },
            transform: transform,
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
        transform: Transform::new(
            Some(Vector3::from_value(0.0)),
            None,
            Some(Vector3::from_value(0.5)),
        ),
        material: RenderMaterial::default(),
    };
    let light_id = renderer.add_render_object(light_ro)?;
    // let _ = renderer.set_light_src(&light_id);

    // Точечный свет 1 (теплый, слева)
    // let _ = renderer.add_point_light(PointLight {
    //     position: Vector3::new(-3.0, 2.0, 0.0), // Слева сверху
    //     constant: 1.0,
    //     linear: 0.09,
    //     quadratic: 0.032,
    //     ambient: Vector3::new(0.05, 0.03, 0.01), // Теплый ambient
    //     diffuse: Vector3::new(1.0, 0.8, 0.6) * 0.4, // Теплый оранжевый
    //     specular: Vector3::new(1.0, 0.9, 0.8) * 0.6, // Теплые блики
    // });

    // // 3. Точечный свет 2 (холодный, справа)
    // let _ = renderer.add_point_light(PointLight {
    //     position: Vector3::new(3.0, 2.0, 1.0), // Справа сверху, немного вперед
    //     constant: 1.0,
    //     linear: 0.07, // Меньше затухание
    //     quadratic: 0.017,
    //     ambient: Vector3::new(0.01, 0.02, 0.05), // Холодный ambient
    //     diffuse: Vector3::new(0.6, 0.8, 1.0) * 0.4, // Холодный синий
    //     specular: Vector3::new(0.8, 0.9, 1.0) * 0.5, // Холодные блики
    // });

    // // // 4. Точечный свет 3 (нейтральный, сзади)
    // let _ = renderer.add_point_light(PointLight {
    //     position: Vector3::new(0.0, 1.5, -3.0), // Сзади сверху
    //     constant: 1.0,
    //     linear: 0.14, // Больше затухание
    //     quadratic: 0.07,
    //     ambient: Vector3::new(0.03, 0.03, 0.03), // Нейтральный ambient
    //     diffuse: Vector3::new(0.9, 0.9, 0.9) * 0.4, // Нейтральный белый
    //     specular: Vector3::new(1.0, 1.0, 1.0) * 0.5, // Белые блики
    // });

    let _ = renderer.add_dir_light(DirLight {
        direction: Vector3::new(0.5, -0.3, -0.4).normalize(),
        ambient: Vector3::from_value(0.15),
        diffuse: Vector3::from_value(0.3),
        specular: Vector3::from_value(0.5),
    });
    let _ = renderer.add_dir_light(DirLight {
        direction: Vector3::new(-0.8, 0.3, 0.2).normalize(),
        ambient: Vector3::from_value(0.05),
        diffuse: Vector3::new(0.06, 0.05, 0.03),
        specular: Vector3::from_value(0.0),
    });

    let w = 10.0;
    let y = -1.0;
    let plane = Plane::new(
        [[w, y, w], [-w, y, w], [w, y, -w], [-w, y, -w]],
        [0.0, 0.0, 0.0],
    )?;
    let _ = renderer.add_static_drawable(plane);

    let mut update_timer = 0.0;
    while !window.should_close() {
        glfw.poll_events();
        let current_frame = glfw.get_time() as f32;
        state.delta_time = current_frame - state.last_frame;
        state.last_frame = current_frame;

        update_timer += state.delta_time;

        process_events(&mut window, &events, &mut state);
        state.camera.process_input(&mut window, state.delta_time);

        if update_timer >= 0.05 {
            update_timer = 0.0;
            for (_, obj) in objIds.iter().enumerate() {
                if let Ok(id) = obj
                    && let Some(render_object) = renderer.get_transform_mut(id)
                {
                    let tr = render_object.get_transform_mut();
                    tr.set_scale(Vector3::from_value(1.0));
                    // tr.scale.x = state.numbers[0];
                    // tr.scale.y = state.numbers[0];
                    // tr.scale.z = state.numbers[0];
                    // let r = tr.get_rotation();
                    // tr.set_rotation(Vector3::new(r.x, glfw.get_time() as f32, r.z));
                    // tr.rotation.x = (glfw.get_time() as f32) * ((i % 10) as f32) + (i * 100) as f32;
                    // tr.rotation.z = (glfw.get_time() as f32) * ((i % 5) as f32) + (i * 100) as f32;
                }
            }
        }

        renderer.render_checkerboard(&mut glfw, &mut state);

        window.swap_buffers();
        // thread::sleep(Duration::from_secs(1) / 60 - Duration::from_millis(state.delta_time as u64));
    }
    Ok(())
}
