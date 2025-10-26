use crate::{Region, Result, ScreenshotError, config::Config};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use super::widgets::AnimatedWidget;

pub struct SelectionOverlay {
    window: ApplicationWindow,
    animated_widget: Rc<RefCell<AnimatedWidget>>,
    config: Config,
}

impl SelectionOverlay {
    pub fn new(app: &Application, config: Config) -> Self {
        let window = Self::create_window(app);
        let display = gdk::Display::default().expect("Could not get default display");

        Self::apply_transparency(&display);

        let screen_width = Self::get_screen_width(&display);
        let animated_widget = Rc::new(RefCell::new(AnimatedWidget::new(
            screen_width / 2.0,
            35.0,
            &config,
        )));

        Self::setup_ui(&window, &animated_widget, &config);
        Self::setup_keyboard_handler(&window, &config);

        Self {
            window,
            animated_widget,
            config,
        }
    }

    fn create_window(app: &Application) -> ApplicationWindow {
        ApplicationWindow::builder()
            .application(app)
            .title("Screenshot Tool")
            .decorated(false)
            .fullscreened(true)
            .build()
    }

    fn apply_transparency(display: &gdk::Display) {
        let provider = CssProvider::new();
        provider.load_from_data("window { background: transparent; }");
        gtk4::style_context_add_provider_for_display(
            display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn get_screen_width(display: &gdk::Display) -> f64 {
        display
            .monitors()
            .item(0)
            .and_then(|obj| obj.downcast::<gdk::Monitor>().ok())
            .map(|m| m.geometry().width() as f64)
            .unwrap_or(1920.0)
    }

    fn setup_ui(
        window: &ApplicationWindow,
        animated_widget: &Rc<RefCell<AnimatedWidget>>,
        config: &Config,
    ) {
        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        let widget = animated_widget.clone();
        drawing_area.set_draw_func(move |_, cr, width, height| {
            cr.set_operator(cairo::Operator::Clear);
            cr.paint().unwrap();
            cr.set_operator(cairo::Operator::Over);

            widget.borrow().draw(cr, width as f64, height as f64);
        });

        window.set_child(Some(&drawing_area));

        Self::start_animation(
            animated_widget,
            &drawing_area,
            config.gui.animation.duration_ms,
        );
    }

    fn start_animation(
        widget: &Rc<RefCell<AnimatedWidget>>,
        area: &gtk4::DrawingArea,
        duration: u64,
    ) {
        let widget = widget.clone();
        let area = area.clone();
        let start_time = Instant::now();

        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let elapsed = start_time.elapsed().as_millis() as f64;
            let progress = (elapsed / duration as f64).min(1.0);

            widget.borrow_mut().update(progress);
            area.queue_draw();

            if progress >= 1.0 {
                widget.borrow_mut().set_showing_controls(true);
                area.queue_draw();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    fn setup_keyboard_handler(window: &ApplicationWindow, config: &Config) {
        let key_controller = gtk4::EventControllerKey::new();
        let window_clone = window.clone();
        let config = config.clone();

        key_controller.connect_key_pressed(move |_, key, _, _| match key {
            gdk::Key::space => {
                let window = window_clone.clone();
                let config = config.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Err(e) = ScreenshotCapture::capture_interactive(&config).await {
                        eprintln!("Screenshot failed: {}", e);
                    }
                    window.close();
                });

                glib::Propagation::Stop
            }
            gdk::Key::Escape => {
                window_clone.close();
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        });

        window.add_controller(key_controller);
    }

    pub fn show(&self) {
        self.window.present();
        self.window.set_opacity(1.0);
    }
}

struct ScreenshotCapture;

impl ScreenshotCapture {
    async fn capture_interactive(config: &Config) -> Result<()> {
        use ashpd::desktop::screenshot::ScreenshotRequest;

        let response = ScreenshotRequest::default()
            .interactive(true)
            .send()
            .await
            .map_err(|e| ScreenshotError::Portal(e.to_string()))?
            .response()
            .map_err(|e| ScreenshotError::Portal(e.to_string()))?;

        let uri = response.uri();
        let path = uri
            .to_file_path()
            .map_err(|_| ScreenshotError::Portal("Invalid file path".to_string()))?;

        let img = image::open(&path).map_err(ScreenshotError::Image)?;
        let screenshot = img.to_rgba8();

        let filename = format!("{}.png", config.generate_filename());
        let output_path = config.save_directory.join(filename);

        screenshot
            .save(&output_path)
            .map_err(ScreenshotError::Image)?;
        println!("{}", output_path.display());

        let _ = std::fs::remove_file(path);

        Ok(())
    }
}
