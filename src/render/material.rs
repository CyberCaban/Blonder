pub struct Material {
    shininess: f32,
    specular: Option<String>,
    emission: Option<String>,
}

impl Material {
    pub fn new(shininess: f32, specular: Option<String>, emission: Option<String>) -> Self {
        Self {
            shininess,
            specular,
            emission,
        }
    }
    pub fn get_specular(&self) -> Option<&String> {
        self.specular.as_ref()
    }
    pub fn get_shininess(&self) -> f32 {
        self.shininess
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shininess: 32.0,
            specular: None,
            emission: None,
        }
    }
}
