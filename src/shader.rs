#[cfg(debug_assertions)]
use std::time::Instant;
use std::{ffi::CString, fs::File, io::Read, ptr};

use anyhow::{Context, Result};
use cgmath::{Matrix, Matrix4, Vector3, Vector4};
use gl::types::{GLchar, GLint};
use log::{error, info, warn};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ShaderInfo {
    pub name: String,
    pub vertex_path: String,
    pub fragment_path: String,
}
impl ShaderInfo {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            vertex_path: String::new(),
            fragment_path: String::new(),
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug)]
pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Result<Self> {
        #[cfg(debug_assertions)]
        let now = Instant::now();

        let mut vertex_source = String::new();
        let mut fragment_source = String::new();
        File::open(vertex_path)
            .context(format!("Cannot find shader [{}]", vertex_path))?
            .read_to_string(&mut vertex_source)
            .context(format!("Cannot read shader [{}]", vertex_path))?;
        File::open(fragment_path)
            .context(format!("Cannot find shader [{}]", fragment_path))?
            .read_to_string(&mut fragment_source)
            .context(format!("Cannot read shader [{}]", fragment_path))?;
        let id = Self::create_shader_program(&vertex_source, &fragment_source);

        #[cfg(debug_assertions)]
        info!(
            "Creating shader program [{}:{}] took {}ms",
            vertex_path,
            fragment_path,
            (Instant::now() - now).as_millis()
        );

        Ok(Self { id })
    }
    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
    pub fn empty_shader() {
        unsafe {
            gl::UseProgram(0);
        }
    }
    pub fn set_float(&self, name: &str, value: f32) {
        let name = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1f(Self::get_uniform_location(self.id, &name), value);
        }
    }
    pub fn set_int(&self, name: &str, value: i32) {
        let name = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1i(Self::get_uniform_location(self.id, &name), value);
        }
    }
    pub fn set_uint(&self, name: &str, value: u32) {
        let name = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1ui(Self::get_uniform_location(self.id, &name), value);
        }
    }
    pub fn set_vec3(&self, name: &str, value: &Vector3<f32>) {
        let name = CString::new(name).unwrap();
        unsafe {
            gl::Uniform3f(
                Self::get_uniform_location(self.id, &name),
                value.x,
                value.y,
                value.z,
            );
        }
    }
    pub fn set_vec4(&self, name: &str, value: &Vector4<f32>) {
        let name = CString::new(name).unwrap();
        unsafe {
            gl::Uniform4f(
                Self::get_uniform_location(self.id, &name),
                value.x,
                value.y,
                value.z,
                value.w,
            );
        }
    }
    pub fn set_mat4(&self, name: &str, value: &Matrix4<f32>) {
        let name = CString::new(name).unwrap();
        unsafe {
            gl::UniformMatrix4fv(
                Self::get_uniform_location(self.id, &name),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }
    pub fn set_transform(&self, transform: &Matrix4<f32>) {
        self.set_mat4("transform", transform);
    }
    fn get_uniform_location(id: u32, name: &CString) -> i32 {
        let location = unsafe { gl::GetUniformLocation(id, name.as_ptr()) };
        if location == -1 {
            // #[cfg(debug_assertions)]
            // warn!("No uniform with name: {:?} found", name);
        }
        location
    }
    fn check_shader_compile_errors(shader: u32) -> bool {
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
                return true;
            }
        }
        false
    }

    fn create_shader_program(vertex_source: &str, fragment_source: &str) -> u32 {
        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_source.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);
            if Self::check_shader_compile_errors(vertex_shader) {
                error!("Vertex shader failed to compile [{}]", vertex_source);
            }

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_vert = CString::new(fragment_source.as_bytes()).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);
            if Self::check_shader_compile_errors(fragment_shader) {
                error!("Fragment shader failed to compile [{}]", fragment_source);
            }

            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            // gl::UseProgram(shader_program);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            shader_program
        }
    }
}
