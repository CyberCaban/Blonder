use anyhow::Result;

use crate::{
    render::{
        color::Color,
        consts::{HEIGHT, WIDTH},
        gui::{
            picking_texture::PickingTexture,
            text_renderer::{TextRenderParams, TextRenderer},
            ui_renderer::UIRenderer,
        },
        renderer::{FontAtlasRef, TextureRef},
    },
    state::{Screen, State},
};

pub enum ButtonState {
    Nothing,
    Hovered,
    Clicked,
}

pub struct UIManager {
    ui_renderer: UIRenderer,
    text_renderer: TextRenderer,

    pub picking_texture: PickingTexture,

    clicked_buttons: Vec<u32>,
    active_slider: Option<u32>,
}

impl UIManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ui_renderer: UIRenderer::new()?,
            text_renderer: TextRenderer::new()?,
            picking_texture: PickingTexture::new(WIDTH as i32, HEIGHT as i32)?,
            clicked_buttons: Vec::new(),
            active_slider: None,
        })
    }
    pub fn begin_frame(&mut self, state: &State, render_screen: &Screen) {
        let State { .. } = state;
        self.ui_renderer
            .update_projection(render_screen.width as f32, render_screen.height as f32);
        self.text_renderer
            .set_projection(render_screen.width as f32, render_screen.height as f32);

        self.ui_renderer.begin_frame();

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }
    }
    pub fn end_frame(&mut self) {
        self.ui_renderer.end_frame();

        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.ui_renderer.draw_rect(x, y, width, height, &color);
    }

    pub fn draw_texture(
        &mut self,
        texture: TextureRef,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        self.ui_renderer
            .draw_texture(texture, x, y, width, height, &color);
    }

    pub fn draw_text(
        &mut self,
        font: &FontAtlasRef,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: Color,
    ) {
        self.ui_renderer.end_frame();

        // unsafe {
        //     gl::Enable(gl::BLEND);
        //     gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        //     gl::Disable(gl::DEPTH_TEST);
        // }

        let params = TextRenderParams { scale, color };

        self.text_renderer.render_text(font, text, x, y, &params);

        self.ui_renderer.begin_frame();
    }
    pub fn panel(
        &mut self,
        id: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
        is_captured: bool,
    ) -> bool {
        let is_hovered = mouse_x >= x
            && mouse_x <= x + width
            && mouse_y >= y
            && mouse_y <= y + height
            && !is_captured;

        let was_clicked = if is_hovered && mouse_pressed {
            self.clicked_buttons.push(id);
            true
        } else {
            false
        };

        self.draw_rect(x, y, width, height, color);

        was_clicked
    }
    pub fn button(
        &mut self,
        id: u32,
        text: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        font: &FontAtlasRef,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
        is_captured: bool,
    ) -> bool {
        let is_hovered = mouse_x >= x
            && mouse_x <= x + width
            && mouse_y >= y
            && mouse_y <= y + height
            && !is_captured;

        let was_clicked = if is_hovered && mouse_pressed {
            self.clicked_buttons.push(id);
            true
        } else {
            false
        };
        let color = if is_hovered {
            if mouse_pressed {
                Color::new(0.1, 0.3, 0.7, 1.0)
            } else {
                Color::new(0.3, 0.5, 0.9, 1.0)
            }
        } else {
            Color::new(0.2, 0.4, 0.8, 1.0)
        };

        self.draw_rect(x, y, width, height, color);

        let scale = 0.3;
        let text_width = font.measure_line(text, scale);
        let text_x = x + (width - text_width) / 2.0;
        let text_y = y + (height - font.size as f32 * scale) / 2.0;

        let text_color = if is_hovered && mouse_pressed {
            Color::white()
        } else {
            Color::black()
        };

        self.draw_text(font, text, text_x, text_y, scale, text_color);
        was_clicked
    }
    pub fn slider(
        &mut self,
        id: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        current_value: f32,
        min: f32,
        max: f32,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
        is_captured: bool,
    ) -> Option<f32> {
        let mut new_value = current_value;
        let mut value_changed = false;
        let is_active = self.active_slider == Some(id);

        let is_hovered = mouse_x >= x
            && mouse_x <= x + width
            && mouse_y >= y
            && mouse_y <= y + height
            && !is_captured;

        if is_hovered && mouse_pressed && !is_active {
            self.active_slider = Some(id);
        }

        if is_active && mouse_pressed {
            let relative_x = (mouse_x - x).max(0.0).min(width);
            new_value = min + (relative_x / width) * (min - max).abs();
            value_changed = true;
        }
        if !mouse_pressed && is_active {
            self.active_slider = None;
        }

        let is_dragging = is_active && mouse_pressed;
        self.draw_slider(
            x,
            y,
            width,
            height,
            (new_value - min) / (min - max).abs(),
            is_hovered,
            is_dragging,
        );

        if value_changed { Some(new_value) } else { None }
    }
    pub fn slider_with_label(
        &mut self,
        id: u32,
        label: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        current_value: f32,
        min: f32,
        max: f32,
        font: &FontAtlasRef,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
        is_captured: bool,
    ) -> Option<f32> {
        let scale = 0.25;
        let label_x = x - 10.0;
        let label_y = y + (height);

        self.draw_text(font, label, label_x, label_y, scale, Color::white());

        self.slider(
            id,
            x,
            y,
            width,
            height,
            current_value,
            min,
            max,
            mouse_x,
            mouse_y,
            mouse_pressed,
            is_captured,
        )
    }
    pub fn slider_with_value(
        &mut self,
        id: u32,
        label: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        current_value: f32,
        min: f32,
        max: f32,
        font: &FontAtlasRef,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
        is_captured: bool,
    ) -> Option<f32> {
        let result = self.slider(
            id,
            x,
            y,
            width,
            height,
            current_value,
            min,
            max,
            mouse_x,
            mouse_y,
            mouse_pressed,
            is_captured,
        );

        let value_text = format!("{current_value:.2}");
        let scale = 0.25;
        let text_width = font.measure_line(&value_text, scale);
        let text_x = x + width + 10.0;
        let text_y = y + (height - font.size as f32 * scale) / 2.0;

        self.draw_text(font, &value_text, text_x, text_y, scale, Color::white());

        result
    }
    fn draw_slider(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        value: f32,
        is_hovered: bool,
        is_dragging: bool,
    ) {
        let value = value.clamp(0.0, 1.0);
        let track_height = height * 0.3;
        let track_y = y + (height - track_height) / 2.0;

        let track_color = if is_hovered || is_dragging {
            Color::new(0.4, 0.4, 0.5, 1.0)
        } else {
            Color::new(0.3, 0.3, 0.4, 1.0)
        };
        self.draw_rect(x, track_y, width, track_height, track_color);

        let fill_color = if is_dragging {
            Color::new(0.2, 0.6, 0.9, 1.0)
        } else {
            Color::new(0.1, 0.5, 0.8, 1.0)
        };
        self.draw_rect(x, track_y, width * value, track_height, fill_color);

        let handle_width = height * 0.8;
        let handle_height = height * 0.8;
        let handle_x = x + width * value - handle_width / 2.0;
        let handle_y = y + (height - handle_height) / 2.0;

        let handle_color = if is_dragging {
            Color::new(0.9, 0.9, 1.0, 1.0)
        } else if is_hovered {
            Color::new(0.8, 0.8, 1.0, 1.0)
        } else {
            Color::new(0.7, 0.7, 1.0, 1.0)
        };

        self.draw_rect(
            handle_x,
            handle_y,
            handle_width,
            handle_height,
            handle_color,
        );

        let outline_color = if is_dragging {
            Color::new(0.3, 0.6, 0.9, 1.0)
        } else {
            Color::new(0.2, 0.4, 0.7, 1.0)
        };
        // self.draw_rect_outline(
        //     handle_x,
        //     handle_y,
        //     handle_width,
        //     handle_height,
        //     2.0,
        //     outline_color,
        // );

        let line_width = handle_width * 0.6;
        let line_height = handle_height * 0.1;
        let line_x = handle_x + (handle_width - line_width) / 2.0;
        let line_y = handle_y + (handle_height - line_height) / 2.0;

        let line_color = if is_dragging {
            Color::new(0.5, 0.5, 0.7, 1.0)
        } else {
            Color::new(0.4, 0.4, 0.6, 1.0)
        };
        self.draw_rect(line_x, line_y, line_width, line_height, line_color);
    }
}
