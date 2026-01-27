use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Arc};

use anyhow::Result;
use log::warn;
use thiserror::Error;

use crate::{
    render::{
        drawable::Drawable,
        model::mesh::Mesh,
        renderer::TextureRef,
        shader::ShaderInfo,
        vertex::{Vertex, calculate_normals, calculate_normals_indexed},
    },
    texture::Texture,
};

mod mesh;

#[derive(Error, Debug)]
pub enum LoadModelError {
    #[error("Invalid model or model file")]
    InvalidModel,
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<mesh::Mesh>,
    pub textures: HashMap<String, TextureRef>,
    directory: String,
}

impl Model {
    pub fn new<T: AsRef<OsStr>>(path: &T) -> Result<Model> {
        let mut model = Model {
            meshes: Vec::new(),
            textures: HashMap::new(),
            directory: String::new(),
        };
        model.load_model(path)?;
        Ok(model)
    }
    fn load_model<T: AsRef<OsStr>>(&mut self, path: &T) -> Result<()> {
        let path = Path::new(path);
        self.directory = path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_str()
            .unwrap()
            .into();

        let (models, materials) = tobj::load_obj(path)?;
        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
            for i in 0..num_vertices {
                vertices.push(Vertex {
                    position: [p[i * 3], p[i * 3 + 1], p[i * 3 + 2]],
                    normal: if n.len() > 0 {
                        [n[i * 3], n[i * 3 + 1], n[i * 3 + 2]]
                    } else {
                        [0.0, 0.0, 0.0]
                    },
                    uv: if t.len() > 0 {
                        [t[i * 2], t[i * 2 + 1]]
                    } else {
                        [0.0, 0.0]
                    },
                });
            }
            if n.len() == 0 {
                calculate_normals_indexed(&mut vertices, &mesh.indices)
                    .unwrap_or_else(|e| warn!("Failed to calculate normals: {e}"));
            }
            let mut textures = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];
                if !material.diffuse_texture.is_empty() {
                    let texture = &format!("{}/{}", self.directory, &material.diffuse_texture);
                    let tex_ref = match self.textures.get(texture) {
                        Some(tex) => Arc::clone(tex),
                        None => {
                            self.textures
                                .insert(texture.to_string(), Arc::new(Texture::new(texture)?));
                            Arc::clone(self.textures.get(texture).unwrap())
                        }
                    };
                    textures.push(tex_ref);
                }
            }

            if vertices.is_empty() {
                return Err(LoadModelError::InvalidModel.into());
            }
            self.meshes
                .push(Mesh::new(vertices, mesh.indices.clone(), textures));
        }

        Ok(())
    }
}

impl Drawable for Model {
    fn draw(&self, glfw: &glfw::Glfw, state: &crate::state::State) {
        for mesh in &self.meshes {
            mesh.draw(glfw, state);
        }
    }
    fn get_blend_mode(&self) -> super::blend_mode::BlendMode {
        super::blend_mode::BlendMode::Opaque
    }
    fn get_shader_name(&self) -> Option<ShaderInfo> {
        None
    }
    fn get_texture_config(&self) -> Option<crate::texture::TextureConfig> {
        None
    }
    fn get_texture_name(&self) -> Option<String> {
        None
    }
}
