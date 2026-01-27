use cgmath::{InnerSpace, Vector3};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn add_pos(&mut self, position: &[f32; 3]) {
        self.position[0] += position[0];
        self.position[1] += position[1];
        self.position[2] += position[2];
    }
    pub fn rotate_around(&mut self, pivot: &[f32; 3], angles: &[f32; 3]) {
        let (pitch, yaw, roll) = (angles[0], angles[1], angles[2]);

        let mut x = self.position[0] - pivot[0];
        let mut y = self.position[1] - pivot[1];
        let mut z = self.position[2] - pivot[2];

        if roll != 0.0 {
            let cos_r = roll.cos();
            let sin_r = roll.sin();
            let x_new = x * cos_r - y * sin_r;
            let y_new = x * sin_r + y * cos_r;
            x = x_new;
            y = y_new;
        }

        if pitch != 0.0 {
            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let y_new = y * cos_p - z * sin_p;
            let z_new = y * sin_p + z * cos_p;
            y = y_new;
            z = z_new;
        }

        if yaw != 0.0 {
            let cos_y = yaw.cos();
            let sin_y = yaw.sin();
            let x_new = x * cos_y + z * sin_y;
            let z_new = -x * sin_y + z * cos_y;
            x = x_new;
            z = z_new;
        }

        self.position[0] = x + pivot[0];
        self.position[1] = y + pivot[1];
        self.position[2] = z + pivot[2];
    }
}

pub fn calculate_normals(vertices: &mut [Vertex]) {
    for i in (0..vertices.len()).step_by(3) {
        let v0 = Vector3::from(vertices[i].position);
        let v1 = Vector3::from(vertices[i + 1].position);
        let v2 = Vector3::from(vertices[i + 2].position);
        let s0 = v1 - v0;
        let s1 = v2 - v0;
        let face_normal = s0.cross(s1).normalize();
        vertices[i].normal = face_normal.into();
        vertices[i + 1].normal = face_normal.into();
        vertices[i + 2].normal = face_normal.into();
    }
}

pub fn calculate_normals_indexed(vertices: &mut [Vertex], indices: &[u32]) -> Result<(), String> {
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;

        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            return Err(format!(
                "Index out of bounds: indices [{}, {}, {}], vertex count {}",
                i0,
                i1,
                i2,
                vertices.len()
            ));
        }

        let v0 = Vector3::from(vertices[i0].position);
        let v1 = Vector3::from(vertices[i1].position);
        let v2 = Vector3::from(vertices[i2].position);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2).normalize();

        vertices[i0].normal[0] += face_normal.x;
        vertices[i0].normal[1] += face_normal.y;
        vertices[i0].normal[2] += face_normal.z;

        vertices[i1].normal[0] += face_normal.x;
        vertices[i1].normal[1] += face_normal.y;
        vertices[i1].normal[2] += face_normal.z;

        vertices[i2].normal[0] += face_normal.x;
        vertices[i2].normal[1] += face_normal.y;
        vertices[i2].normal[2] += face_normal.z;
    }

    for vertex in vertices.iter_mut() {
        let normal_vec = Vector3::new(vertex.normal[0], vertex.normal[1], vertex.normal[2]);
        if normal_vec.magnitude() > 0.0 {
            let normalized = normal_vec.normalize();
            vertex.normal = [normalized.x, normalized.y, normalized.z];
        } else {
            vertex.normal = [0.0, 1.0, 0.0];
        }
    }

    Ok(())
}
