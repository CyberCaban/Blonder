use std::cell::Cell;

use cgmath::{Array, Matrix4, Rad, SquareMatrix, Vector3};
use num::Zero;

#[derive(Debug, Clone)]
pub struct Transform {
    position: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
    cached_model: Cell<Option<Matrix4<f32>>>,
}

impl Transform {
    pub fn new(position: Option<Vector3<f32>>, rotation: Option<Vector3<f32>>, scale: Option<Vector3<f32>>) -> Self {
        let t = Transform {
            position: position.unwrap_or(Vector3::zero()),
            rotation: rotation.unwrap_or(Vector3::zero()),
            scale: scale.unwrap_or(Vector3::from_value(1.0)),
            cached_model: None.into(),
        };
        t.calculate_model();
        t
    }
    pub fn get_position(&self) -> Vector3<f32> {
        self.position
    }
    pub fn get_rotation(&self) -> Vector3<f32> {
        self.rotation
    }
    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale
    }
    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.cached_model.set(None);
    }

    pub fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.rotation = rotation;
        self.cached_model.set(None);
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
        self.cached_model.set(None);
    }
    pub fn calculate_model(&self) -> Matrix4<f32> {
        if let Some(cached) = self.cached_model.get() {
            return cached;
        }
        let model = self.calculate_model_internal();
        self.cached_model.set(Some(model));
        model
    }
    fn calculate_model_internal(&self) -> Matrix4<f32> {
        if self.position == Vector3::zero()
            && self.rotation == Vector3::zero()
            && self.scale == Vector3::new(1.0, 1.0, 1.0)
        {
            return Matrix4::identity();
        }

        let (sin_x, cos_x) = self.rotation.x.sin_cos();
        let (sin_y, cos_y) = self.rotation.y.sin_cos();
        let (sin_z, cos_z) = self.rotation.z.sin_cos();

        Matrix4::new(
            self.scale.x * (cos_y * cos_z + sin_x * sin_y * sin_z),
            self.scale.x * (cos_x * sin_z),
            self.scale.x * (cos_y * sin_x * sin_z - cos_z * sin_y),
            0.0,
            self.scale.y * (cos_z * sin_x * sin_y - cos_y * sin_z),
            self.scale.y * (cos_x * cos_z),
            self.scale.y * (cos_y * cos_z * sin_x + sin_y * sin_z),
            0.0,
            self.scale.z * (cos_x * sin_y),
            -self.scale.z * (sin_x),
            self.scale.z * (cos_x * cos_y),
            0.0,
            self.position.x,
            self.position.y,
            self.position.z,
            1.0,
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            cached_model: Cell::new(None),
        }
    }
}
