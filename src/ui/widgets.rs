use crate::config::Config;
use std::f64::consts::PI;

pub struct AnimatedWidget {
    center_x: f64,
    center_y: f64,
    progress: f64,
    start_radius: f64,
    end_width: f64,
    end_height: f64,
    corner_radius: f64,
    #[allow(dead_code)]
    duration_ms: u64,
    showing_controls: bool,
    #[allow(dead_code)]
    config: Config,
}

impl AnimatedWidget {
    pub fn new(x: f64, y: f64, config: &Config) -> Self {
        Self {
            center_x: x,
            center_y: y,
            progress: 0.0,
            start_radius: 22.0,
            end_width: 280.0,
            end_height: 52.0,
            corner_radius: 26.0,
            duration_ms: config.gui.animation.duration_ms,
            showing_controls: false,
            config: config.clone(),
        }
    }

    pub fn update(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn set_showing_controls(&mut self, show: bool) {
        self.showing_controls = show;
    }

    fn ease_in_out_cubic(&self, t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    fn draw_rounded_rect(
        &self,
        cr: &cairo::Context,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
    ) {
        let degrees = PI / 180.0;
        let r = radius.min(width / 2.0).min(height / 2.0);

        cr.new_sub_path();
        cr.arc(x + width - r, y + r, r, -90.0 * degrees, 0.0 * degrees);
        cr.arc(
            x + width - r,
            y + height - r,
            r,
            0.0 * degrees,
            90.0 * degrees,
        );
        cr.arc(x + r, y + height - r, r, 90.0 * degrees, 180.0 * degrees);
        cr.arc(x + r, y + r, r, 180.0 * degrees, 270.0 * degrees);
        cr.close_path();
    }

    pub fn draw(&self, cr: &cairo::Context, width: f64, _height: f64) {
        let t = self.ease_in_out_cubic(self.progress);

        let current_width =
            self.start_radius * 2.0 + t * (self.end_width - self.start_radius * 2.0);
        let current_height =
            self.start_radius * 2.0 + t * (self.end_height - self.start_radius * 2.0);

        let center_x = width / 2.0;
        let x = center_x - current_width / 2.0;
        let y = self.center_y - current_height / 2.0;

        let current_radius = if t < 0.3 {
            current_width.min(current_height) / 2.0
        } else {
            let morph_progress = (t - 0.3) / 0.7;
            let circle_radius = current_width.min(current_height) / 2.0;
            circle_radius + morph_progress * (self.corner_radius - circle_radius)
        };

        cr.save().unwrap();
        cr.translate(0.0, 1.5);
        self.draw_rounded_rect(cr, x, y, current_width, current_height, current_radius);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
        cr.fill().unwrap();
        cr.restore().unwrap();

        self.draw_rounded_rect(cr, x, y, current_width, current_height, current_radius);

        let pattern = cairo::LinearGradient::new(0.0, y, 0.0, y + current_height);
        pattern.add_color_stop_rgba(0.0, 0.11, 0.11, 0.13, 0.96);
        pattern.add_color_stop_rgba(1.0, 0.08, 0.08, 0.10, 0.96);
        cr.set_source(&pattern).unwrap();
        cr.fill_preserve().unwrap();

        cr.set_source_rgba(0.25, 0.25, 0.28, 0.5);
        cr.set_line_width(0.8);
        cr.stroke().unwrap();

        if self.showing_controls && self.progress >= 0.95 {
            self.draw_controls(cr, x, y, current_width, current_height);
        }
    }

    fn draw_controls(&self, cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64) {
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(11.5);

        let controls = [("⏎ Space", "Save"), ("✕ Esc", "Cancel")];

        let section_width = width / controls.len() as f64;
        let center_y = y + height / 2.0;

        for (i, (key, _action)) in controls.iter().enumerate() {
            let section_x = x + (i as f64 * section_width);
            let center_x = section_x + section_width / 2.0;

            let extents = cr.text_extents(key).unwrap();
            cr.set_source_rgba(0.92, 0.92, 0.92, 1.0);
            cr.move_to(
                center_x - extents.width() / 2.0,
                center_y + extents.height() / 2.0 - 1.0,
            );
            cr.show_text(key).unwrap();

            if i < controls.len() - 1 {
                cr.set_source_rgba(0.28, 0.28, 0.32, 0.4);
                cr.set_line_width(0.8);
                let separator_x = section_x + section_width;
                cr.move_to(separator_x, y + 12.0);
                cr.line_to(separator_x, y + height - 12.0);
                cr.stroke().unwrap();
            }
        }
    }
}
