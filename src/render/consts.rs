pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;

pub const MAX_PITCH_ANGLE: f32 = 89.0 * 1.75;
pub const MAX_POINT_LIGHTS: usize = 8;
pub const MAX_DIR_LIGHTS: usize = 4;
pub const MAX_SPOT_LIGHTS: usize = 4;

pub const DEFAULT_FONT: &str = "OpenSans";
pub const DEFAULT_WHITE_TEXTURE: &str = "DEFAULT";
pub const DEFAULT_BLACK_TEXTURE: &str = "DEFAULT_BLACK";
pub const DEFAULT_SHADER_NAME: &str = "DEFAULT";
pub const DEFAULT_SHADER_VERT: &str = "assets/shaders/light/vert.glsl";
pub const DEFAULT_SHADER_FRAG: &str = "assets/shaders/light/frag.glsl";

pub const DEFAULT_SHADOW_SHADER_NAME: &str = "SHADOW";
pub const DEFAULT_SHADOW_SHADER_VERT: &str = "assets/shaders/shadow/vert.glsl";
pub const DEFAULT_SHADOW_SHADER_FRAG: &str = "assets/shaders/shadow/frag.glsl";
