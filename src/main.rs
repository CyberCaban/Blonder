use std::{mem::offset_of, os::raw::c_void, ptr};

use ::log::info;
use anyhow::{Context as _, Result};
use cgmath::{Deg, SquareMatrix};
use gl::types::{GLsizei, GLsizeiptr};
use glfw::{Context, Glfw, PWindow};

use crate::{
    events::process_events,
    log::setup_logger,
    shader::Shader,
    state::{Events, State},
    texture::Texture,
};

extern crate gl;
extern crate glfw;

mod events;
mod log;
mod shader;
mod state;
mod texture;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[repr(C)]
#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 3],
}

type Mat4 = cgmath::Matrix4<f32>;
type Vec3 = cgmath::Vector3<f32>;

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
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
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

fn set_buffer_data(vao: u32, vbo: u32, data: &[Vertex]) {
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
            3,
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

#[derive(Debug)]
struct Serpinsky {
    pub points: Vec<Vertex>,
    pub vao: u32,
    pub shader: Shader,
    pub texture: Texture,
}

impl Serpinsky {
    pub fn new() -> Result<Self> {
        Ok(Self {
            points: vec![],
            vao: 0,
            shader: Shader::new("shaders/serpinsky/vert.glsl", "shaders/serpinsky/frag.glsl")?,
            texture: Texture::new("textures/white.png")?,
        })
    }
    fn serp(&mut self, point_a: &[f32; 3], point_b: &[f32; 3], point_c: &[f32; 3], mut depth: u32) {
        if depth == 0 {
            return;
        }
        fn middle(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
            [
                (a[0] + b[0]) / 2.0,
                (a[1] + b[1]) / 2.0,
                (a[2] + b[2]) / 2.0,
            ]
        }
        let (px, py, pz) = (
            middle(point_a, point_b),
            middle(point_a, point_c),
            middle(point_b, point_c),
        );

        self.points.extend_from_slice(&[
            Vertex {
                position: px,
                uv: [0.0, 0.0],
                color: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: py,
                uv: [1.0, 0.0],
                color: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: pz,
                uv: [0.5, 1.0],
                color: [0.0, 0.0, 0.0],
            },
        ]);

        depth -= 1;
        self.serp(point_a, &px, &py, depth);
        self.serp(&px, point_b, &pz, depth);
        self.serp(&py, &pz, point_c, depth);
    }
    fn prepare(&mut self) {
        let (mut vbo, mut vao) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            set_buffer_data(vao, vbo, &self.points);
        }
        self.vao = vao;
    }
    fn draw(&self, glfw: &mut Glfw) {
        unsafe {
            let transform = Mat4::identity();
            self.shader.use_shader();
            self.shader.set_transform(&transform);
            self.shader.set_int("tex", 0);
            self.shader.set_float("time", glfw.get_time() as f32);
            gl::ActiveTexture(gl::TEXTURE0);
            self.texture.use_texture();
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.points.len() as i32);
        }
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
    let texture = [
        Texture::new("textures/liminal_space.png")?,
        Texture::new("textures/cooler.png")?,
    ];

    let mut serp = Serpinsky::new()?;
    serp.serp(&[-0.7, -0.7, 0.0], &[0.7, -0.7, 0.0], &[0., 0.7, 0.0], 7);
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

    while !window.should_close() {
        process_events(&mut window, &events, &mut state);

        let State {
            color, wireframe, ..
        } = state;
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // configurable parameters
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if wireframe { gl::LINE } else { gl::FILL },
            );
            // state.transform_matrix =
            //     Mat4::from_axis_angle(Vec3::unit_z(), Deg((glfw.get_time() * 75.0) as f32))
            //         * Mat4::from_translation(Vec3::unit_y() * 0.7);

            // Draw calls and such
            shader_program[0].use_shader();
            shader_program[0].set_mat4("transform", &state.transform_matrix);
            // shader_program[0].set_int("texture1", 0);
            // shader_program[0].set_int("texture2", 1);
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
            serp.draw(&mut glfw);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
    Ok(())
}
