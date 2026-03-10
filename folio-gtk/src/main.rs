mod app;
mod launch;
mod window;
mod canvas;
mod toolbar;
mod sidebar;
mod dialogs;
mod fonts;

fn main() {
    // Initialise GTK and libadwaita, then hand off to the application.
    let app = app::FolioApp::new();
    app.run();
}
