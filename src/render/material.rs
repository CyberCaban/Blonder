pub struct Material {
    shininess: f32,
    specular: Option<String>,
}

impl Material {
    pub fn new(shininess: f32, specular: Option<String>) -> Self {
        Self {
            shininess,
            specular,
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
            specular: None,
            shininess: 32.0,
        }
    }
}
