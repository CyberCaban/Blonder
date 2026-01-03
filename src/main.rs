use std::{ffi::CString, os::raw::c_void, ptr};

use ::log::{error, info, warn};
use anyhow::{Context as _, Result};
use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr};
use glfw::{Context, Glfw, PWindow};

use crate::{
    events::process_events,
    log::setup_logger,
    state::{Events, State},
};

extern crate gl;
extern crate glfw;

mod events;
mod log;
mod state;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTICES_NUM: i32 = 4;
const VERTICES_SIZE: i32 = 3;
const VERTICES: [f32; (VERTICES_NUM * VERTICES_SIZE) as usize] = [
    // first rect
    0.5, 0.5, 0.0, // 0
    0.5, -0.5, 0.0, // 1
    -0.5, -0.5, 0.0, // 2
    -0.5, 0.5, 0.0, // 3
];
const INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];
const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    uniform vec4 ourColor;
    out vec4 FragColor;
    void main() {
       FragColor = ourColor;
    }
"#;

const FRAGMENT_SHADER_SOURCE_YELLOW: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);
    }
"#;

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

fn check_shader_compile_errors(shader: u32) {
    unsafe {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        // info_log.set_len(512 - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            error!(
                "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                str::from_utf8(&info_log).unwrap()
            );
        }
    }
}

fn create_shader_program(vertex_source: &str, fragment_source: &str) -> u32 {
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(vertex_source.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
        gl::CompileShader(vertex_shader);
        check_shader_compile_errors(vertex_shader);

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_vert = CString::new(fragment_source.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
        gl::CompileShader(fragment_shader);
        check_shader_compile_errors(fragment_shader);

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        shader_program
    }
}

fn set_buffer_data(vao: u32, vbo: u32, data: &[f32], data_size: u32) {
    unsafe {
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(data) as GLsizeiptr,
            &data[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            data_size as i32,
            gl::FLOAT,
            gl::FALSE,
            (data_size as usize * std::mem::size_of::<f32>()) as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }
}

fn main() -> Result<()> {
    setup_logger()?;
    info!("Hello, world!");
    let mut glfw = glfw::init_no_callbacks().context("Failed to init glfw")?;
    let (mut window, events) = init_window(&mut glfw)?;
    let mut state = State::default();

    let (shader_program, vao) = unsafe {
        let shader_program = create_shader_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        let shader_program2 =
            create_shader_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_YELLOW);

        let triangle1: [f32; 9] = [
            -0.6, -0.6, 0.0, // 4
            0.7, -0.7, 0.0, // 5
            0.0, -0.9, 0.0, // 6
        ];
        let triangle2: [f32; 9] = [
            0.6, 0.6, 0.0, // 7
            0.6, -0.6, 0.0, // 8
            0.8, 0.0, 0.0, // 9
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
            (VERTICES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            &VERTICES[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDICES.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
            &INDICES[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            VERTICES_SIZE,
            gl::FLOAT,
            gl::FALSE,
            (VERTICES_SIZE as usize * std::mem::size_of::<GLfloat>()) as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // second shape
        set_buffer_data(vao[1], vbo[1], &triangle1, 3);
        // third shape
        set_buffer_data(vao[2], vbo[2], &triangle2, 3);

        // unbinding
        // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        // gl::BindVertexArray(0);

        (vec![shader_program, shader_program2], vao)
    };

    while !window.should_close() {
        process_events(&mut window, &events, &mut state);

        let State { color, wireframe } = state;
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if wireframe { gl::LINE } else { gl::FILL },
            );
            gl::UseProgram(shader_program[0]);
            let time = glfw.get_time() as f32;
            let green = (time.sin() / 2.0) + 0.5;
            let our_color = CString::new("ourColor").unwrap();
            let vertex_color_location =
                gl::GetUniformLocation(shader_program[0], our_color.as_ptr());
            if vertex_color_location == -1 {
                warn!("Could not find uniform location {}", shader_program[0]);
            }
            gl::Uniform4f(vertex_color_location, 0.0, green, 0.0, 1.0);
            gl::BindVertexArray(vao[0]);
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );

            gl::BindVertexArray(vao[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::UseProgram(shader_program[1]);
            gl::BindVertexArray(vao[2]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
    Ok(())
}
