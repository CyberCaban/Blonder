use crate::render::color::Color;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: Color,
}

impl Vertex {
    pub fn add_pos(&mut self, position: &[f32; 3]) {
        self.position[0] += position[0];
        self.position[1] += position[1];
        self.position[2] += position[2];
    }
}
