use crate::models::cubeworld::{
    chunk_mesh::in_chunk_bounds,
    consts::{C, CHUNK_D, CHUNK_H, CHUNK_VOL, CHUNK_W},
};

#[derive(Debug, Clone, Copy)]
pub struct Voxel {
    pub id: u32,
}

#[derive(Debug)]
pub struct Chunk {
    voxels: [Voxel; CHUNK_VOL],
    pub position: [i32; 3],
}

impl Chunk {
    pub fn new(position: &[i32; 3]) -> Self {
        let voxels = Self::init_voxels(position);

        Self {
            voxels,
            position: *position,
        }
    }
    fn init_voxels(position: &[i32; 3]) -> [Voxel; CHUNK_VOL] {
        let mut voxels = [Voxel { id: 0 }; CHUNK_VOL];
        for y in 0..CHUNK_H {
            for z in 0..CHUNK_D {
                for x in 0..CHUNK_W {
                    let rx = x as i32 + position[0] * CHUNK_W as i32;
                    let ry = y as i32 + position[1] * CHUNK_H as i32;
                    let rz = z as i32 + position[2] * CHUNK_D as i32;
                    let height = ((rx as f32 * 0.1).sin() * 10.0
                        + (rz as f32 * 0.15).cos() * 8.0
                        + ((rx as f32 * 0.05) * (rz as f32 * 0.05)).sin() * 5.0)
                        as i32
                        + 00;
                    let id = if ry < height {
                        // Разные типы блоков в зависимости от высоты
                        if ry == height - 1 {
                            2 // Трава сверху
                        } else if ry > height - 5 {
                            3 // Земля под травой
                        } else {
                            1 // Камень глубоко
                        }
                    } else if ry == height && height < 25 {
                        2 // Трава на поверхности
                    } else {
                        0 // Воздух
                    };
                    voxels[Self::get_voxel_index(x, y, z)].id = id;
                }
            }
        }
        voxels
    }
    pub fn get_voxel_index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_D + z) * CHUNK_W + x
    }
    pub fn get_voxels(&self) -> &[Voxel; CHUNK_VOL] {
        &self.voxels
    }
    pub fn get_voxel(&self, x: C, y: C, z: C) -> Option<Voxel> {
        if in_chunk_bounds(x, y, z) {
            Some(self.voxels[Self::get_voxel_index(x as usize, y as usize, z as usize)])
        } else {
            None
        }
    }
}
