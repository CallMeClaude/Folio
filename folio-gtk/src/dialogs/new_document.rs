use libadwaita::{Application, ApplicationWindow};
use folio_core::{Document, PageSettings};
use crate::window::DocumentWindow;

/// Phase 2: open a new document with default settings immediately.
/// Phase 3 will replace this with a proper paper-size picker dialog.
pub fn show(app: &Application, _parent: &ApplicationWindow) {
    let doc = Document::new("Untitled", PageSettings::default());
    let win = DocumentWindow::new(app, doc);
    win.present();
}
