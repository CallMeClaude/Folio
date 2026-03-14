use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow};
use gio;
use folio_core::format::load_folio;
use crate::window::DocumentWindow;

pub fn show(app: &Application, parent: &ApplicationWindow) {
    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Folio Documents (*.folio)"));
    filter.add_pattern("*.folio");

    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);

    let dialog = gtk4::FileDialog::builder()
        .title("Open Document")
        .filters(&filters)
        .modal(true)
        .build();

    let app_ref    = app.clone();
    let parent_ref = parent.clone();

    glib::spawn_future_local(async move {
        let result = dialog.open_future(Some(&parent_ref)).await;
        match result {
            Ok(file) => {
                let Some(path) = file.path() else { return };
                match load_folio(&path) {
                    Ok((engine, doc, _assets)) => {
                        use std::path::PathBuf;
                        let canvas = crate::canvas::EditorCanvas::from_loaded(
                            doc.clone(), PathBuf::from(&path), engine
                        );
                        let win = DocumentWindow::from_canvas(&app_ref, canvas);
                        win.present();
                    }
                    Err(e) => {
                        let alert = libadwaita::AlertDialog::builder()
                            .heading("Could not open document")
                            .body(&e.to_string())
                            .build();
                        alert.add_response("ok", "OK");
                        alert.set_default_response(Some("ok"));
                        alert.present(Some(&parent_ref));
                    }
                }
            }
            Err(_) => { /* user cancelled — do nothing */ }
        }
    });
}
