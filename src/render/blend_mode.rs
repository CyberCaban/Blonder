#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Opaque,
    AlphaTest,
    AlphaBlend,
    Additive,
    Multiplicative,
}

impl BlendMode {
    pub fn apply(&self) {
        unsafe {
            match self {
                BlendMode::Opaque => {
                    gl::Disable(gl::BLEND);
                    gl::DepthMask(gl::TRUE);
                }
                BlendMode::AlphaTest => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    gl::DepthMask(gl::TRUE);
                }
                BlendMode::AlphaBlend => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    gl::DepthMask(gl::FALSE);
                }
                BlendMode::Additive => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE);
                    gl::BlendEquation(gl::FUNC_ADD);
                }
                BlendMode::Multiplicative => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::DST_COLOR, gl::ZERO);
                }
            }
        }
    }
}
