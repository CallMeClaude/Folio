use std::path::PathBuf;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk4::{
    Box as GBox, Button, Label, ListBox, ListBoxRow, Orientation,
    Align, ScrolledWindow, Separator, PolicyType,
};
use folio_core::format::read_folio_metadata;

pub struct LaunchWindow {
    pub window: ApplicationWindow,
}

/// XDG data dir → folio/documents/
pub fn documents_dir() -> PathBuf {
    glib::user_data_dir().join("folio").join("documents")
}

impl LaunchWindow {
    pub fn new(app: &Application) -> Self {
        // Ensure documents directory exists.
        let docs_dir = documents_dir();
        let _ = std::fs::create_dir_all(&docs_dir);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Folio")
            .default_width(820)
            .default_height(560)
            .build();

        let toolbar_view = ToolbarView::new();
        let header = HeaderBar::new();

        // New Document button in header
        let btn_new = Button::with_label("New Document");
        btn_new.add_css_class("suggested-action");
        btn_new.add_css_class("pill");
        header.pack_start(&btn_new);

        toolbar_view.add_top_bar(&header);

        // ── Main layout ───────────────────────────────────────────────────
        let root = GBox::new(Orientation::Vertical, 0);

        // Title area
        let title_box = GBox::new(Orientation::Vertical, 6);
        title_box.set_margin_top(32);
        title_box.set_margin_bottom(16);
        title_box.set_margin_start(32);
        title_box.set_margin_end(32);

        let wordmark = Label::new(Some("Folio"));
        wordmark.add_css_class("title-1");
        wordmark.set_halign(Align::Start);

        let subtitle = Label::new(Some("Your documents"));
        subtitle.add_css_class("dim-label");
        subtitle.set_halign(Align::Start);

        title_box.append(&wordmark);
        title_box.append(&subtitle);
        root.append(&title_box);
        root.append(&Separator::new(Orientation::Horizontal));

        // ── Document list ─────────────────────────────────────────────────
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list = ListBox::new();
        list.add_css_class("navigation-sidebar");
        list.set_selection_mode(gtk4::SelectionMode::Single);
        list.set_margin_start(16);
        list.set_margin_end(16);
        list.set_margin_top(8);
        list.set_margin_bottom(8);

        // Populate with existing .folio files, sorted newest first.
        let mut entries: Vec<(PathBuf, String, String)> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&docs_dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("folio") {
                    let (title, modified) = match read_folio_metadata(&path) {
                        Ok(meta) => (
                            if meta.title.is_empty() { "Untitled".to_string() }
                            else { meta.title.clone() },
                            meta.modified.format("%d %b %Y, %H:%M").to_string(),
                        ),
                        Err(_) => ("Unknown document".to_string(), String::new()),
                    };
                    entries.push((path, title, modified));
                }
            }
        }
        // Sort by file modified time (newest first) using filesystem metadata.
        entries.sort_by(|a, b| {
            let ta = std::fs::metadata(&a.0).and_then(|m| m.modified()).ok();
            let tb = std::fs::metadata(&b.0).and_then(|m| m.modified()).ok();
            tb.cmp(&ta)
        });

        if entries.is_empty() {
            let empty_lbl = Label::new(Some("No documents yet. Create one to get started."));
            empty_lbl.add_css_class("dim-label");
            empty_lbl.set_margin_top(40);
            empty_lbl.set_halign(Align::Center);
            list.append(&{
                let row = ListBoxRow::new();
                row.set_child(Some(&empty_lbl));
                row.set_activatable(false);
                row.set_selectable(false);
                row
            });
        }

        for (path, title, modified) in entries {
            let row = ListBoxRow::new();
            let row_box = GBox::new(Orientation::Vertical, 2);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);

            let title_lbl = Label::new(Some(&title));
            title_lbl.set_halign(Align::Start);
            title_lbl.add_css_class("body");

            let date_lbl = Label::new(Some(&modified));
            date_lbl.set_halign(Align::Start);
            date_lbl.add_css_class("dim-label");
            date_lbl.add_css_class("caption");

            row_box.append(&title_lbl);
            row_box.append(&date_lbl);
            row.set_child(Some(&row_box));
            list.append(&row);

            // Wire row click → open document.
            let path_c   = path.clone();
            let app_c    = app.clone();
            let win_ref  = window.clone();
            row.connect_activate(move |_| {
                use folio_core::format::load_folio;
                match load_folio(&path_c) {
                    Ok((engine, doc, _)) => {
                        let canvas = crate::canvas::EditorCanvas::from_loaded(
                            doc, path_c.clone(), engine,
                        );
                        let dw = crate::window::DocumentWindow::from_canvas(&app_c, canvas);
                        dw.present();
                        win_ref.close();
                    }
                    Err(e) => eprintln!("Failed to open document: {e}"),
                }
            });
        }

        scroll.set_child(Some(&list));
        root.append(&scroll);

        toolbar_view.set_content(Some(&root));
        window.set_content(Some(&toolbar_view));

        // Wire New Document button.
        {
            let app_ref = app.clone();
            let win_ref = window.clone();
            btn_new.connect_clicked(move |_| {
                crate::dialogs::new_document::show(&app_ref, &win_ref);
            });
        }

        LaunchWindow { window }
    }

    pub fn present(&self) { self.window.present(); }
}
