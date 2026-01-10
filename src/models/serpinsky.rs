use crate::{
    render::{drawable::Drawable, helpers::set_buffer_data, vertex::Vertex},
    shader::ShaderInfo,
    texture::Texture,
};

use anyhow::Result;
use cgmath::{InnerSpace, Vector3};

#[derive(Debug)]
pub struct Serpinsky {
    pub points: Vec<Vertex>,
    pub vao: u32,
    pub shader: ShaderInfo,
    pub texture: Texture,
}

impl Serpinsky {
    pub fn new() -> Result<Self> {
        Ok(Self {
            points: vec![],
            vao: 0,
            shader: ShaderInfo {
                name: "serpinsky".to_string(),
                vertex_path: "assets/shaders/serpinsky/vert.glsl".to_string(),
                fragment_path: "assets/shaders/serpinsky/frag.glsl".to_string(),
            },
            texture: Texture::new("assets/textures/white.png")?,
        })
    }
    pub fn make_coh(
        &mut self,
        point_a: &[f32; 3],
        point_b: &[f32; 3],
        point_c: &[f32; 3],
        depth: u32,
    ) {
        self.points.extend_from_slice(&[
            Vertex {
                position: *point_a,
                uv: [0.5, -0.5],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: *point_b,
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: *point_c,
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
        ]);
        self.coh(point_a, point_b, point_c, depth);
    }
    fn coh(&mut self, point_a: &[f32; 3], point_b: &[f32; 3], point_c: &[f32; 3], mut depth: u32) {
        if depth == 0 {
            return;
        }

        fn third_1(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
            [
                (a[0] + b[0]) / 3.0,
                (a[1] + b[1]) / 3.0,
                (a[2] + b[2]) / 3.0,
            ]
        }
        fn third_2(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
            [
                (a[0] + b[0]) / 3.0 * 2.0,
                (a[1] + b[1]) / 3.0 * 2.0,
                (a[2] + b[2]) / 3.0 * 2.0,
            ]
        }
        fn middle(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
            [
                (a[0] + b[0]) / 2.0,
                (a[1] + b[1]) / 2.0,
                (a[2] + b[2]) / 2.0,
            ]
        }
        // 1 side
        let p1 = third_1(point_a, point_b);
        let p2 = third_2(point_a, point_b);
        let mut m = middle(&p1, &p2);
        let v1 = Vector3::from([
            point_a[0] + point_b[0],
            point_a[1] + point_b[1],
            point_a[2] + point_b[2],
        ]);
        let v2 = Vector3::from([
            point_a[0] + point_c[0],
            point_a[1] + point_c[1],
            point_a[2] + point_c[2],
        ]);
        let mv = v1.cross(v2).normalize() * 2.0;
        m[0] += mv.x;
        m[1] += mv.y;
        m[2] += mv.z;

        self.points.extend_from_slice(&[
            Vertex {
                position: m,
                uv: [0.5, -0.5],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: p1,
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: p2,
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
        ]);

        depth -= 1;
        self.coh(&p1, &m, &p2, depth);
    }
    pub fn carpet(
        &mut self,
        point_a: &[f32; 3],
        point_b: &[f32; 3],
        point_c: &[f32; 3],
        mut depth: u32,
    ) {
        if depth == 0 {
            return;
        }

        self.points.extend_from_slice(&[
            Vertex {
                position: *point_a,
                uv: [0.5, -0.5],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: *point_b,
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: *point_c,
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
        ]);

        let mut copy = self.points.clone();
        copy.iter_mut().for_each(|v| {
            v.rotate_around(point_b, &[12.0, 60.0, 0.0]);
        });
        self.points.extend_from_slice(&copy);
        let p: Vec<_> = self.points.iter().rev().take(3).cloned().collect();

        depth -= 1;
        self.carpet(&p[0].position, &p[1].position, &p[2].position, depth);
        self.carpet(&p[1].position, &p[2].position, &p[0].position, depth);
        self.carpet(&p[2].position, &p[0].position, &p[1].position, depth);
    }
    pub fn triangle(
        &mut self,
        point_a: &[f32; 3],
        point_b: &[f32; 3],
        point_c: &[f32; 3],
        mut depth: u32,
    ) {
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
                uv: [0.5, -0.5],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: py,
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: pz,
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
        ]);

        depth -= 1;
        self.triangle(point_a, &px, &py, depth);
        self.triangle(&px, point_b, &pz, depth);
        self.triangle(&py, &pz, point_c, depth);
    }
    pub fn prepare(&mut self) {
        let (mut vbo, mut vao) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            set_buffer_data(vao, vbo, &self.points);
        }
        self.vao = vao;
    }
}

impl Drawable for Serpinsky {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.points.len() as i32);
            gl::Enable(gl::CULL_FACE);
        }
    }
    fn get_texture_name(&self) -> String {
        "assets/textures/cooler.png".to_string()
    }
    fn get_shader_name(&self) -> ShaderInfo {
        self.shader.clone()
    }
    fn requires_shader(&self) -> bool {
        false
    }
    fn requires_texture(&self) -> bool {
        true
    }
    fn get_blend_mode(&self) -> crate::render::blend_mode::BlendMode {
        crate::render::blend_mode::BlendMode::Opaque
    }
}
