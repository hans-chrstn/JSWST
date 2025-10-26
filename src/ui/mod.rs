#[cfg(feature = "gui")]
pub mod editor;
#[cfg(feature = "gui")]
pub mod overlay;
#[cfg(feature = "gui")]
pub mod widgets;

#[cfg(feature = "gui")]
pub use editor::EditorWindow;
#[cfg(feature = "gui")]
pub use overlay::SelectionOverlay;

#[cfg(feature = "gui")]
use crate::{Result, config::Config};
#[cfg(feature = "gui")]
use std::path::PathBuf;

#[cfg(feature = "gui")]
pub async fn launch_gui(config: Config) -> Result<()> {
    use gtk4::prelude::*;

    gtk4::init()
        .map_err(|_| crate::error::ScreenshotError::Gui("Failed to init GTK".to_string()))?;

    let app = gtk4::Application::builder()
        .application_id("com.hans-chrstn.just-a-simple-wayland-screenshot-tool.editor")
        .build();

    app.connect_activate(move |app| {
        let overlay = SelectionOverlay::new(app, config.clone());
        overlay.show();
    });

    app.run();
    Ok(())
}

#[cfg(feature = "gui")]
pub async fn launch_editor(file: PathBuf, config: Config) -> Result<()> {
    use gtk4::prelude::*;

    gtk4::init()
        .map_err(|_| crate::error::ScreenshotError::Gui("Failed to init GTK".to_string()))?;

    let app = gtk4::Application::builder()
        .application_id("com.hans-chrstn.just-a-simple-wayland-screenshot-tool.editor")
        .build();

    app.connect_activate(move |app| {
        let pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_file(&file).expect("Failed to load pixbuf");

        match EditorWindow::new(app, pixbuf, config.clone()) {
            Ok(editor) => editor.show(),
            Err(e) => eprintln!("Failed to create editor: {}", e),
        }
    });

    app.run();
    Ok(())
}
