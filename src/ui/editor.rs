use crate::config::Config;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, DrawingArea, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EditorWindow {
    window: ApplicationWindow,
    pixbuf: Rc<RefCell<Pixbuf>>,
}

impl EditorWindow {
    pub fn new(app: &Application, pixbuf: Pixbuf, _config: Config) -> crate::Result<Self> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Screenshot Editor")
            .default_width(800)
            .default_height(600)
            .build();

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        window.set_child(Some(&main_box));

        let pixbuf = Rc::new(RefCell::new(pixbuf));

        let toolbar = Self::create_toolbar();
        main_box.append(&toolbar);

        let drawing_area = DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);

        {
            let pixbuf = pixbuf.clone();
            drawing_area.set_draw_func(move |_, cr, width, height| {
                Self::draw_image(cr, &pixbuf.borrow(), width, height);
            });
        }

        main_box.append(&drawing_area);

        Ok(Self { window, pixbuf })
    }

    fn create_toolbar() -> GtkBox {
        let toolbar = GtkBox::new(Orientation::Horizontal, 5);
        toolbar.set_margin_start(10);
        toolbar.set_margin_end(10);
        toolbar.set_margin_top(10);
        toolbar.set_margin_bottom(10);

        let save_btn = Button::with_label("💾 Save");
        let crop_btn = Button::with_label("✂️ Crop");
        let copy_btn = Button::with_label("📋 Copy");

        toolbar.append(&save_btn);
        toolbar.append(&crop_btn);
        toolbar.append(&copy_btn);

        toolbar
    }

    fn draw_image(cr: &cairo::Context, pixbuf: &Pixbuf, width: i32, height: i32) {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();

        let img_width = pixbuf.width() as f64;
        let img_height = pixbuf.height() as f64;

        let scale_x = width as f64 / img_width;
        let scale_y = height as f64 / img_height;
        let scale = scale_x.min(scale_y) * 0.9;

        let offset_x = (width as f64 - img_width * scale) / 2.0;
        let offset_y = (height as f64 - img_height * scale) / 2.0;

        cr.save().unwrap();
        cr.translate(offset_x, offset_y);
        cr.scale(scale, scale);
        cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
        cr.paint().unwrap();
        cr.restore().unwrap();
    }

    pub fn show(&self) {
        self.window.present();
    }
}
