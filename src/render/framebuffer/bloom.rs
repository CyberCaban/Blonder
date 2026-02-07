use crate::{
    render::{framebuffer::Framebuffer, shader::Shader},
    state::State,
    texture::{TextureFilter, TextureFormatColor, TextureWrap},
};
use anyhow::Result;

#[derive(Debug)]
pub struct Bloom {
    // Ping-pong framebuffers for blur passes
    pub pingpong_fbos: [Framebuffer; 2],
    // Framebuffer for brightness extraction
    pub brightness_fbo: Framebuffer,
    // Shaders
    brightness_shader: Shader,
    blur_shader: Shader,
    combine_shader: Shader,
    // Settings
    pub enabled: bool,
    pub blur_iterations: i32,
    pub exposure: f32,
    pub bloom_intensity: f32,
    pub threshold: f32,
}

impl Bloom {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        // Create brightness extraction framebuffer (half resolution for performance)
        let mut brightness_fbo = Framebuffer::new(width / 2, height / 2)?;
        brightness_fbo.add_color_attachment(
            0,
            TextureFormatColor::RGBA16F,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
        )?;
        brightness_fbo.check_complete()?;

        // Create ping-pong framebuffers for blur (quarter resolution)
        let mut pingpong_fbo1 = Framebuffer::new(width / 4, height / 4)?;
        pingpong_fbo1.add_color_attachment(
            0,
            TextureFormatColor::RGBA16F,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
        )?;
        pingpong_fbo1.check_complete()?;

        let mut pingpong_fbo2 = Framebuffer::new(width / 4, height / 4)?;
        pingpong_fbo2.add_color_attachment(
            0,
            TextureFormatColor::RGBA16F,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
        )?;
        pingpong_fbo2.check_complete()?;

        // Load shaders
        let brightness_shader = Shader::new(
            "assets/shaders/bloom/vert.glsl",
            "assets/shaders/bloom/brightness.frag.glsl",
        )?;

        let blur_shader = Shader::new(
            "assets/shaders/bloom/vert.glsl",
            "assets/shaders/bloom/blur.frag.glsl",
        )?;

        let combine_shader = Shader::new(
            "assets/shaders/bloom/vert.glsl",
            "assets/shaders/bloom/combine.frag.glsl",
        )?;

        Ok(Self {
            pingpong_fbos: [pingpong_fbo1, pingpong_fbo2],
            brightness_fbo,
            brightness_shader,
            blur_shader,
            combine_shader,
            enabled: true,
            blur_iterations: 2,
            exposure: 1.0,
            bloom_intensity: 0.3,
            threshold: 0.8,
        })
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        self.brightness_fbo.resize(width / 2, height / 2)?;
        self.pingpong_fbos[0].resize(width / 4, height / 4)?;
        self.pingpong_fbos[1].resize(width / 4, height / 4)?;
        Ok(())
    }

    pub fn process(&self, scene_texture: &Framebuffer, state: &State) {
        if !self.enabled {
            return;
        }

        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }

        // Step 1: Extract bright areas
        self.extract_brightness(scene_texture);

        // Step 2: Blur bright areas (ping-pong between framebuffers)
        self.apply_blur();

        // Step 3: Combine scene with bloom
        // This will be done by the caller with the result framebuffer

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    fn extract_brightness(&self, scene_texture: &Framebuffer) {
        self.brightness_fbo.begin_render();

        self.brightness_shader.use_shader();

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        if let Some(color_tex) = scene_texture.get_color_texture(0) {
            color_tex.bind(0);
        }
        self.brightness_shader.set_int("image", 0);
        self.brightness_shader
            .set_float("threshold", self.threshold);

        self.render_quad();
        self.brightness_fbo.unbind();
    }

    fn apply_blur(&self) {
        let mut horizontal = true;
        let mut first_iteration = true;

        for _ in 0..self.blur_iterations {
            let current_fbo = &self.pingpong_fbos[if horizontal { 0 } else { 1 }];
            current_fbo.begin_render();

            self.blur_shader.use_shader();
            self.blur_shader
                .set_int("horizontal", if horizontal { 1 } else { 0 });

            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
            }

            if first_iteration {
                if let Some(brightness_tex) = self.brightness_fbo.get_color_texture(0) {
                    brightness_tex.bind(0);
                }
            } else {
                let source_fbo = &self.pingpong_fbos[if horizontal { 1 } else { 0 }];
                if let Some(color_tex) = source_fbo.get_color_texture(0) {
                    color_tex.bind(0);
                }
            }
            self.blur_shader.set_int("image", 0);

            self.render_quad();
            current_fbo.unbind();

            horizontal = !horizontal;
            if first_iteration {
                first_iteration = false;
            }
        }
    }

    pub fn render_final(
        &self,
        scene_texture: &Framebuffer,
        viewport: Option<(i32, i32, i32, i32)>,
    ) {
        self.combine_shader.use_shader();

        // Bind scene texture
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        if let Some(color_tex) = scene_texture.get_color_texture(0) {
            color_tex.bind(0);
        }
        self.combine_shader.set_int("scene", 0);

        // Bind bloom texture (last pingpong buffer)
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1);
        }
        let bloom_source = if self.blur_iterations % 2 == 0 {
            &self.pingpong_fbos[0]
        } else {
            &self.pingpong_fbos[1]
        };
        if let Some(bloom_tex) = bloom_source.get_color_texture(0) {
            bloom_tex.bind(1);
        }
        self.combine_shader.set_int("bloomBlur", 1);

        // Set uniforms
        self.combine_shader.set_float("exposure", self.exposure);
        self.combine_shader
            .set_float("bloomIntensity", self.bloom_intensity);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if let Some((x, y, width, height)) = viewport {
                gl::Viewport(x, y, width, height);
            }

            gl::BindVertexArray(scene_texture.screen_quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }

    fn render_quad(&self) {
        unsafe {
            gl::BindVertexArray(self.brightness_fbo.screen_quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}
