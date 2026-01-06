use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Color([f32; 4]);

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r,g,b,a])
    }
    pub const fn white() -> Self {
        Self([1.0, 1.0, 1.0, 1.0])
    }
    pub const fn black() -> Self {
        Self([0.0, 0.0, 0.0, 1.0])
    }
}

impl Deref for Color {
    type Target = [f32; 4];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}