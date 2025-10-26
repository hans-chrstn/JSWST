use crate::{Region, cli, config::Config};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, CssProvider, DrawingArea, EventControllerKey,
    EventControllerMotion, GestureClick, gdk, glib,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tracing::info;

use super::widgets::AnimatedWidget;

pub struct SelectionOverlay {
    window: ApplicationWindow,
    selection: Rc<RefCell<Option<Region>>>,
    drag_start: Rc<RefCell<Option<(f64, f64)>>>,
    is_dragging: Rc<RefCell<bool>>,
    animated_widget: Rc<RefCell<AnimatedWidget>>,
    #[allow(dead_code)]
    screen_width: f64,
    config: Config,
}

impl SelectionOverlay {
    pub fn new(app: &Application, config: Config) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Select Screenshot Area")
            .decorated(false)
            .fullscreened(true)
            .build();

        let display = gdk::Display::default().expect("Could not get default display");
        let provider = CssProvider::new();
        provider.load_from_data("window { background: transparent; }");
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let monitor = display
            .monitors()
            .item(0)
            .and_then(|obj| obj.downcast::<gdk::Monitor>().ok())
            .expect("Could not get monitor");
        let geometry = monitor.geometry();
        let screen_width = geometry.width() as f64;

        let overlay = gtk4::Overlay::new();
        window.set_child(Some(&overlay));

        let selection_drawing_area = DrawingArea::new();
        selection_drawing_area.set_hexpand(true);
        selection_drawing_area.set_vexpand(true);
        overlay.set_child(Some(&selection_drawing_area));

        let widget_drawing_area = DrawingArea::new();
        widget_drawing_area.set_hexpand(true);
        widget_drawing_area.set_vexpand(true);
        widget_drawing_area.set_can_target(false); // Don't block mouse events
        overlay.add_overlay(&widget_drawing_area);

        let selection = Rc::new(RefCell::new(None));
        let drag_start = Rc::new(RefCell::new(None));
        let is_dragging = Rc::new(RefCell::new(false));

        let initial_widget = AnimatedWidget::new(screen_width / 2.0, 35.0, &config);
        let animated_widget = Rc::new(RefCell::new(initial_widget));

        {
            let animated_widget = animated_widget.clone();
            let widget_drawing_area = widget_drawing_area.clone();
            let start_time = Instant::now();
            let duration = config.gui.animation.duration_ms;

            glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                let elapsed = start_time.elapsed().as_millis() as f64;
                let progress = (elapsed / duration as f64).min(1.0);

                animated_widget.borrow_mut().update(progress);
                widget_drawing_area.queue_draw();

                if progress >= 1.0 {
                    animated_widget.borrow_mut().set_showing_controls(true);
                    widget_drawing_area.queue_draw();
                    return glib::ControlFlow::Break;
                }
                glib::ControlFlow::Continue
            });
        }

        {
            let selection = selection.clone();
            selection_drawing_area.set_draw_func(move |_, cr, _width, _height| {
                cr.set_operator(cairo::Operator::Clear);
                cr.paint().unwrap();
                cr.set_operator(cairo::Operator::Over);

                Self::draw_selection(cr, &selection.borrow());
            });
        }

        {
            let animated_widget = animated_widget.clone();
            widget_drawing_area.set_draw_func(move |_, cr, width, height| {
                cr.set_operator(cairo::Operator::Clear);
                cr.paint().unwrap();
                cr.set_operator(cairo::Operator::Over);

                animated_widget
                    .borrow()
                    .draw(cr, width as f64, height as f64);
            });
        }

        let click = GestureClick::new();
        {
            let drag_start = drag_start.clone();
            let selection = selection.clone();
            let is_dragging = is_dragging.clone();
            let selection_drawing_area = selection_drawing_area.clone();

            click.connect_pressed(move |_, _, x, y| {
                *drag_start.borrow_mut() = Some((x, y));
                *selection.borrow_mut() = None;
                *is_dragging.borrow_mut() = true;
                selection_drawing_area.queue_draw();
            });
        }

        {
            let drag_start = drag_start.clone();
            let selection = selection.clone();
            let is_dragging = is_dragging.clone();
            let selection_drawing_area = selection_drawing_area.clone();

            click.connect_released(move |_, _, x, y| {
                if let Some((start_x, start_y)) = *drag_start.borrow() {
                    let sel = Region::new(
                        start_x.min(x) as i32,
                        start_y.min(y) as i32,
                        (x - start_x).abs() as u32,
                        (y - start_y).abs() as u32,
                    );
                    *selection.borrow_mut() = Some(sel);
                    selection_drawing_area.queue_draw();
                }
                *is_dragging.borrow_mut() = false;
            });
        }

        selection_drawing_area.add_controller(click);

        let motion = EventControllerMotion::new();
        {
            let drag_start = drag_start.clone();
            let selection = selection.clone();
            let is_dragging = is_dragging.clone();
            let selection_drawing_area = selection_drawing_area.clone();

            motion.connect_motion(move |_, x, y| {
                if *is_dragging.borrow() {
                    if let Some((start_x, start_y)) = *drag_start.borrow() {
                        let sel = Region::new(
                            start_x.min(x) as i32,
                            start_y.min(y) as i32,
                            (x - start_x).abs() as u32,
                            (y - start_y).abs() as u32,
                        );
                        *selection.borrow_mut() = Some(sel);
                        selection_drawing_area.queue_draw();
                    }
                }
            });
        }

        selection_drawing_area.add_controller(motion);

        let key_controller = EventControllerKey::new();
        {
            let selection = selection.clone();
            let window = window.clone();
            let monitor_geom = geometry;
            key_controller.connect_key_pressed(move |_, key, _, _| match key {
                gdk::Key::space => {
                    if let Some(mut sel) = *selection.borrow() {
                        let window_clone = window.clone();

                        glib::MainContext::default().spawn_local(async move {
                            sel.x += monitor_geom.x();
                            sel.y += monitor_geom.y();

                            info!("Region selected via GUI: {:?}", sel);

                            let args = cli::Args {
                                mode: Some("screen".to_string()),
                                output: None,
                                format: None,
                                delay: None,
                                clipboard: false,
                                cursor: false,
                                quiet: false,
                                json: false,
                                headless: true, // This is important!
                                region: Some(format!(
                                    "{},{},{},{}",
                                    sel.x, sel.y, sel.width, sel.height
                                )),
                                monitor: None,
                                command: None,
                            };

                            if let Err(e) = cli::execute(args).await {
                                eprintln!("Failed to capture and save: {}", e);
                            }

                            window_clone.close();
                        });
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    window.close();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            });
        }

        window.add_controller(key_controller);

        Self {
            window,
            selection,
            drag_start,
            is_dragging,
            animated_widget,
            screen_width,
            config,
        }
    }

    fn draw_selection(cr: &cairo::Context, selection: &Option<Region>) {
        if let Some(sel) = selection {
            let norm = sel.normalize();

            cr.set_source_rgba(0.2, 0.5, 1.0, 0.9);
            cr.set_line_width(2.0);
            cr.rectangle(
                norm.x as f64,
                norm.y as f64,
                norm.width as f64,
                norm.height as f64,
            );
            cr.stroke().unwrap();

            let handle_size = 8.0;
            let handles = [
                (norm.x as f64, norm.y as f64),
                (norm.x as f64 + norm.width as f64, norm.y as f64),
                (norm.x as f64, norm.y as f64 + norm.height as f64),
                (
                    norm.x as f64 + norm.width as f64,
                    norm.y as f64 + norm.height as f64,
                ),
            ];

            cr.set_source_rgba(0.2, 0.5, 1.0, 0.9);
            for (hx, hy) in handles.iter() {
                cr.arc(*hx, *hy, handle_size / 2.0, 0.0, 2.0 * std::f64::consts::PI);
                cr.fill().unwrap();
            }

            let text = format!("{}×{}", norm.width, norm.height);
            cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(13.0);

            let extents = cr.text_extents(&text).unwrap();
            let text_x = norm.x as f64 + norm.width as f64 / 2.0 - extents.width() / 2.0;
            let text_y = norm.y as f64 - 10.0;

            cr.set_source_rgba(0.1, 0.1, 0.12, 0.95);
            let padding = 6.0;
            cr.rectangle(
                text_x - padding,
                text_y - extents.height() - padding,
                extents.width() + padding * 2.0,
                extents.height() + padding * 2.0,
            );
            cr.fill().unwrap();

            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.move_to(text_x, text_y - 2.0);
            cr.show_text(&text).unwrap();
        }
    }

    pub fn show(&self) {
        self.window.present();

        self.window.set_opacity(1.0);
    }
}
