use std::{ffi::CString, os::raw::c_void, ptr};

use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr};
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};

extern crate gl;
extern crate glfw;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTICES_NUM: i32 = 3;
const VERTICES_SIZE: i32 = 3;
const VERTICES: [f32; (VERTICES_NUM * VERTICES_SIZE) as usize] = [
    -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, -0.5, 0.5,
    0.5,
    // -0.5, 0.5, 0.0,
    // 0.5, 0.5, 0.0,
    // 0.5, -0.5, 0.5,
];
const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

type Events = GlfwReceiver<(f64, WindowEvent)>;

fn init_window(glfw: &mut Glfw) -> (PWindow, Events) {
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "Hello", glfw::WindowMode::Windowed)
        .expect("Failed to create window");
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
    (window, events)
}

fn check_shader_compile_errors(shader: u32) {
    unsafe {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                str::from_utf8(&info_log).unwrap()
            );
        }
    }
}

fn main() {
    println!("Hello, world!");
    let mut glfw = glfw::init_no_callbacks().unwrap();
    let (mut window, events) = init_window(&mut glfw);
    let mut color = (0.0, 0.0, 0.0, 1.0);

    let (shader_program, vao) = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), 0 as *const _);
        gl::CompileShader(vertex_shader);
        check_shader_compile_errors(vertex_shader);

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_vert = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_vert.as_ptr(), 0 as *const _);
        gl::CompileShader(fragment_shader);
        check_shader_compile_errors(fragment_shader);

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTICES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            &VERTICES[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            VERTICES_NUM,
            gl::FLOAT,
            gl::FALSE,
            (VERTICES_NUM as usize * std::mem::size_of::<GLfloat>()) as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        (shader_program, vao)
    };

    while !window.should_close() {
        process_events(&mut window, &events, &mut color);

        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, VERTICES_NUM);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Events, color: &mut (f32, f32, f32, f32)) {
    for (msg, event) in glfw::flush_messages(events) {
        println!("Message: {}\nEvent: {:?}", msg, event);
        match event {
            WindowEvent::FileDrop(param) => {
                for p in param {
                    println!("{}", p.to_string_lossy());
                }
                println!("Decrement color BLUE {}", color.2);
            }
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }

            WindowEvent::Key(Key::A, _, Action::Press, _) => {
                color.0 += 0.1;
                color.0 = color.0.clamp(0.0, 1.0);
                println!("Increment color RED {}", color.0);
            }
            WindowEvent::Key(Key::D, _, Action::Press, _) => {
                color.0 -= 0.1;
                color.0 = color.0.clamp(0.0, 1.0);
                println!("Decrement color RED {}", color.0);
            }
            WindowEvent::Key(Key::W, _, Action::Press, _) => {
                color.1 += 0.1;
                color.1 = color.1.clamp(0.0, 1.0);
                println!("Increment color GREEN {}", color.1);
            }
            WindowEvent::Key(Key::S, _, Action::Press, _) => {
                color.1 -= 0.1;
                color.1 = color.1.clamp(0.0, 1.0);
                println!("Decrement color GREEN {}", color.1);
            }
            WindowEvent::Scroll(w, h) => {
                color.2 += (h * 0.01) as f32;
                color.2 = color.2.clamp(0.0, 1.0);
                println!("color BLUE {}", color.2);
            }
            _ => {}
        }
    }
}
