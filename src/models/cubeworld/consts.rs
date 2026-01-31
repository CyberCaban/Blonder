pub const CHUNK_W: usize = 16;
pub const CHUNK_H: usize = 16;
pub const CHUNK_D: usize = 16;
pub const CHUNK_VOL: usize = CHUNK_W * CHUNK_H * CHUNK_D;

pub const ATLAS_SIDE: f32 = 16.0;
pub const UV_SIZE: f32 = 1.0 / ATLAS_SIDE;
pub type C = i32;
