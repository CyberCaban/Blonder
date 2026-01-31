use crate::models::cubeworld::chunk::Chunk;

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
        for chunk in &chunks {
            println!("Created chunk at position: {:?}", chunk.position);
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
    pub fn get_chunk(&self, x: usize, y: usize, z: usize) -> &Chunk {
        let index = self.get_chunk_index(x, y, z);
        &self.chunks[index]
    }
    pub fn get_voxel(
        &self,
        x: usize,
        y: usize,
        z: usize,
    ) -> Option<crate::models::cubeworld::chunk::Voxel> {
        let chunk_x = x / crate::models::cubeworld::consts::CHUNK_W;
        let chunk_y = y / crate::models::cubeworld::consts::CHUNK_H;
        let chunk_z = z / crate::models::cubeworld::consts::CHUNK_D;

        if chunk_x >= self.dimensions.width_in_chunks
            || chunk_y >= self.dimensions.height_in_chunks
            || chunk_z >= self.dimensions.depth_in_chunks
        {
            return None;
        }

        let chunk = self.get_chunk(chunk_x, chunk_y, chunk_z);
        let local_x = x % crate::models::cubeworld::consts::CHUNK_W;
        let local_y = y % crate::models::cubeworld::consts::CHUNK_H;
        let local_z = z % crate::models::cubeworld::consts::CHUNK_D;
        chunk.get_voxel(local_x as i32, local_y as i32, local_z as i32)
    }
}
