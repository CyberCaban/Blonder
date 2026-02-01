use crate::models::cubeworld::{
    chunk::{Chunk, Voxel},
    consts::{C, CHUNK_D, CHUNK_H, CHUNK_W},
};

pub struct RaycastResult {
    pub hit_pos: [f32; 3],
    pub hit_coords: [i32; 3],
    pub normal: [f32; 3],
    pub voxel: Voxel,
    pub is_hit: bool,
}

pub struct WorldDimensions {
    pub width_in_chunks: usize,
    pub depth_in_chunks: usize,
    pub height_in_chunks: usize,
}

pub struct Chunks {
    pub chunks: Vec<Chunk>,
    pub dimensions: WorldDimensions,
    volume: usize,
}

impl Chunks {
    pub fn new(dimensions: WorldDimensions) -> Self {
        let volume =
            dimensions.width_in_chunks * dimensions.depth_in_chunks * dimensions.height_in_chunks;
        let mut chunks = Vec::with_capacity(volume);
        for y in 0..dimensions.height_in_chunks {
            for z in 0..dimensions.depth_in_chunks {
                for x in 0..dimensions.width_in_chunks {
                    let position = [(x) as i32, (y) as i32, (z) as i32];
                    let chunk = Chunk::new(&position);
                    chunks.push(chunk);
                }
            }
        }
        Self {
            chunks,
            dimensions,
            volume,
        }
    }
    pub fn get_volume(&self) -> usize {
        self.volume
    }
    pub fn get_chunk_index(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.dimensions.width_in_chunks * (z + self.dimensions.depth_in_chunks * y)
    }
    pub fn get_chunk(&self, x: C, y: C, z: C) -> Option<&Chunk> {
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dimensions.width_in_chunks as i32
            || y >= self.dimensions.height_in_chunks as i32
            || z >= self.dimensions.depth_in_chunks as i32
        {
            None
        } else {
            let index = self.get_chunk_index(x as usize, y as usize, z as usize);
            Some(&self.chunks[index])
        }
    }
    pub fn get_chunk_mut(&mut self, x: C, y: C, z: C) -> Option<&mut Chunk> {
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dimensions.width_in_chunks as i32
            || y >= self.dimensions.height_in_chunks as i32
            || z >= self.dimensions.depth_in_chunks as i32
        {
            None
        } else {
            let index = self.get_chunk_index(x as usize, y as usize, z as usize);
            Some(&mut self.chunks[index])
        }
    }
    pub fn get_voxel(&self, x: C, y: C, z: C) -> Option<Voxel> {
        let mut chunk_x = x / CHUNK_W as i32;
        let mut chunk_y = y / CHUNK_H as i32;
        let mut chunk_z = z / CHUNK_D as i32;
        if x < 0 {
            chunk_x -= 1;
        }
        if y < 0 {
            chunk_y -= 1;
        }
        if z < 0 {
            chunk_z -= 1;
        }

        if chunk_x < 0
            || chunk_y < 0
            || chunk_z < 0
            || chunk_x >= self.dimensions.width_in_chunks as i32
            || chunk_y >= self.dimensions.height_in_chunks as i32
            || chunk_z >= self.dimensions.depth_in_chunks as i32
        {
            return None;
        }

        let chunk = self.get_chunk(chunk_x, chunk_y, chunk_z);
        if chunk.is_none() {
            return None;
        }
        let chunk = chunk.unwrap();
        let local_x = x - chunk_x * CHUNK_W as C;
        let local_y = y - chunk_y * CHUNK_H as C;
        let local_z = z - chunk_z * CHUNK_D as C;
        chunk.get_voxel(local_x as i32, local_y as i32, local_z as i32)
    }
    pub fn set_voxel(&mut self, x: C, y: C, z: C, voxel: Voxel) {
        let mut chunk_x = x / CHUNK_W as i32;
        let mut chunk_y = y / CHUNK_H as i32;
        let mut chunk_z = z / CHUNK_D as i32;
        if x < 0 {
            chunk_x -= 1;
        }
        if y < 0 {
            chunk_y -= 1;
        }
        if z < 0 {
            chunk_z -= 1;
        }

        if chunk_x < 0
            || chunk_y < 0
            || chunk_z < 0
            || chunk_x >= self.dimensions.width_in_chunks as i32
            || chunk_y >= self.dimensions.height_in_chunks as i32
            || chunk_z >= self.dimensions.depth_in_chunks as i32
        {
            return;
        }

        let chunk = self.get_chunk_mut(chunk_x, chunk_y, chunk_z);
        if chunk.is_none() {
            return;
        }
        let chunk = chunk.unwrap();
        let local_x = x - chunk_x * CHUNK_W as C;
        let local_y = y - chunk_y * CHUNK_H as C;
        let local_z = z - chunk_z * CHUNK_D as C;
        if let Some(v) = chunk.get_voxel_mut(local_x as i32, local_y as i32, local_z as i32) {
            *v = voxel;
        }
        chunk.mark_modified();
        if local_x == 0
            && let Some(chunk) = self.get_chunk_mut(chunk_x - 1, chunk_y, chunk_z)
        {
            chunk.mark_modified();
        }
        if local_y == 0
            && let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_y - 1, chunk_z)
        {
            chunk.mark_modified();
        }
        if local_z == 0
            && let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_y, chunk_z - 1)
        {
            chunk.mark_modified();
        }
        if local_x == (CHUNK_W as C - 1)
            && let Some(chunk) = self.get_chunk_mut(chunk_x + 1, chunk_y, chunk_z)
        {
            chunk.mark_modified();
        }
        if local_y == (CHUNK_H as C - 1)
            && let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_y + 1, chunk_z)
        {
            chunk.mark_modified();
        }
        if local_z == (CHUNK_D as C - 1)
            && let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_y, chunk_z + 1)
        {
            chunk.mark_modified();
        }
    }
    pub fn raycast(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        max_distance: f32,
    ) -> RaycastResult {
        let (mut px, mut py, mut pz) = (origin[0], origin[1], origin[2]);
        let (dx, dy, dz) = (direction[0], direction[1], direction[2]);

        // Нормализуем направление
        let dir_len = (dx * dx + dy * dy + dz * dz).sqrt();
        let (dx, dy, dz) = (dx / dir_len, dy / dir_len, dz / dir_len);

        let mut t = 0.0;
        let (mut ix, mut iy, mut iz) = (px.floor() as i32, py.floor() as i32, pz.floor() as i32);

        let (step_x, step_y, step_z) = (
            if dx > 0.0 { 1 } else { -1 },
            if dy > 0.0 { 1 } else { -1 },
            if dz > 0.0 { 1 } else { -1 },
        );

        let inf = f32::INFINITY;

        let (tx_delta, ty_delta, tz_delta) = (
            if dx != 0.0 { (1.0 / dx).abs() } else { inf },
            if dy != 0.0 { (1.0 / dy).abs() } else { inf },
            if dz != 0.0 { (1.0 / dz).abs() } else { inf },
        );

        // Исправлено: нужно использовать ix, iy, iz как f32 для правильного вычисления
        let (xdist, ydist, zdist) = (
            if step_x > 0 {
                (ix as f32 + 1.0) - px
            } else {
                px - ix as f32
            },
            if step_y > 0 {
                (iy as f32 + 1.0) - py
            } else {
                py - iy as f32
            },
            if step_z > 0 {
                (iz as f32 + 1.0) - pz
            } else {
                pz - iz as f32
            },
        );

        let (mut tx_max, mut ty_max, mut tz_max) = (
            if tx_delta < inf {
                tx_delta * xdist
            } else {
                inf
            },
            if ty_delta < inf {
                ty_delta * ydist
            } else {
                inf
            },
            if tz_delta < inf {
                tz_delta * zdist
            } else {
                inf
            },
        );

        let mut stepped_index = -1;

        while t <= max_distance {
            // Получаем воксель в текущих координатах
            let voxel = self.get_voxel(ix, iy, iz);

            // Проверяем: если воксель существует и не пустой (id != 0)
            if let Some(voxel) = voxel {
                if voxel.id != 0 {
                    // Попали в непустой воксель
                    let hit_pos = [
                        origin[0] + direction[0] * t,
                        origin[1] + direction[1] * t,
                        origin[2] + direction[2] * t,
                    ];

                    let mut normal = [0.0, 0.0, 0.0];
                    match stepped_index {
                        0 => normal[0] = -step_x as f32,
                        1 => normal[1] = -step_y as f32,
                        2 => normal[2] = -step_z as f32,
                        _ => {}
                    }

                    return RaycastResult {
                        hit_pos,
                        hit_coords: [ix, iy, iz],
                        normal,
                        voxel,
                        is_hit: true,
                    };
                }
            } else {
                // Выход за границы мира - считаем что попали
                let hit_pos = [
                    origin[0] + direction[0] * t,
                    origin[1] + direction[1] * t,
                    origin[2] + direction[2] * t,
                ];

                let mut normal = [0.0, 0.0, 0.0];
                match stepped_index {
                    0 => normal[0] = -step_x as f32,
                    1 => normal[1] = -step_y as f32,
                    2 => normal[2] = -step_z as f32,
                    _ => {}
                }

                return RaycastResult {
                    hit_pos,
                    hit_coords: [ix, iy, iz],
                    normal,
                    voxel: Voxel { id: 0 },
                    is_hit: true,
                };
            }

            // Выбираем следующую ось для шага
            if tx_max < ty_max {
                if tx_max < tz_max {
                    // Двигаемся по X
                    t = tx_max;
                    tx_max += tx_delta;
                    ix += step_x;
                    stepped_index = 0;
                } else {
                    // Двигаемся по Z
                    t = tz_max;
                    tz_max += tz_delta;
                    iz += step_z;
                    stepped_index = 2;
                }
            } else {
                if ty_max < tz_max {
                    // Двигаемся по Y
                    t = ty_max;
                    ty_max += ty_delta;
                    iy += step_y;
                    stepped_index = 1;
                } else {
                    // Двигаемся по Z
                    t = tz_max;
                    tz_max += tz_delta;
                    iz += step_z;
                    stepped_index = 2;
                }
            }
        }

        // Не попали ни во что
        RaycastResult {
            hit_pos: [
                origin[0] + direction[0] * t.min(max_distance),
                origin[1] + direction[1] * t.min(max_distance),
                origin[2] + direction[2] * t.min(max_distance),
            ],
            hit_coords: [ix, iy, iz],
            normal: [0.0, 0.0, 0.0],
            voxel: Voxel { id: 0 },
            is_hit: false,
        }
    }
}
