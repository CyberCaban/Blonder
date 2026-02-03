use std::cell::RefCell;

use noise::{
    NoiseFn, Perlin, Vector3,
    core::perlin::{perlin_2d, perlin_3d},
    permutationtable::PermutationTable,
    utils::{NoiseMapBuilder, PlaneMapBuilder},
};

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
    pub modified: RefCell<bool>,
}

impl Chunk {
    pub fn new(position: &[i32; 3]) -> Self {
        let voxels = Self::init_voxels(position);

        Self {
            voxels,
            position: *position,
            modified: RefCell::new(true),
        }
    }
    fn init_voxels(position: &[i32; 3]) -> [Voxel; CHUNK_VOL] {
        let mut voxels = [Voxel { id: 0 }; CHUNK_VOL];
        let hasher = PermutationTable::new(4);

        for z in 0..CHUNK_D {
            for x in 0..CHUNK_W {
                let rz = z as i32 + position[2] * CHUNK_D as i32;
                let rx = x as i32 + position[0] * CHUNK_W as i32;
                let height =
                    perlin_3d(Vector3::new(rx as f64 * 0.05, rz as f64 * 0.05, 0.0), &hasher) * 10.0
                        + 20.0;
                for y in 0..CHUNK_H {
                    let ry = y as i32 + position[1] * CHUNK_H as i32;
                    let id = if (ry as f64) < height {
                        // Разные типы блоков в зависимости от высоты
                        if (ry as f64) == height - 1.0 {
                            2 // Трава сверху
                        } else if (ry as f64) > height - 5.0 {
                            3 // Земля под травой
                        } else {
                            1 // Камень глубоко
                        }
                    } else if (ry as f64) == height && height < 25.0 {
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
    pub fn get_voxel_mut(&mut self, x: C, y: C, z: C) -> Option<&mut Voxel> {
        if in_chunk_bounds(x, y, z) {
            Some(&mut self.voxels[Self::get_voxel_index(x as usize, y as usize, z as usize)])
        } else {
            None
        }
    }
    pub fn mark_modified(&self) {
        *self.modified.borrow_mut() = true;
    }

    pub fn is_modified(&self) -> bool {
        *self.modified.borrow()
    }

    pub fn reset_modified(&self) {
        *self.modified.borrow_mut() = false;
    }
}
