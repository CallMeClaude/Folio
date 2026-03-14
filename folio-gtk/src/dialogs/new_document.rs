use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk4::{Box as GBox, Button, CheckButton, Entry, Label, Orientation, Align};
use folio_core::{Document, PageSettings, PaperSize, CrdtEngine};
use crate::launch::documents_dir;
use crate::window::DocumentWindow;

pub fn show(app: &Application, parent: &ApplicationWindow) {
    let dialog = libadwaita::Dialog::builder()
        .title("New Document")
        .content_width(360)
        .build();

    let tv = ToolbarView::new();
    tv.add_top_bar(&HeaderBar::new());

    let vbox = GBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    // ── Title field ────────────────────────────────────────────────────────
    let title_lbl = Label::new(Some("Title"));
    title_lbl.set_halign(Align::Start);
    let title_entry = Entry::builder().placeholder_text("Untitled").build();
    vbox.append(&title_lbl);
    vbox.append(&title_entry);

    // ── Paper size ─────────────────────────────────────────────────────────
    let size_lbl = Label::new(Some("Paper size"));
    size_lbl.set_halign(Align::Start);
    vbox.append(&size_lbl);

    let sizes: &[(&str, PaperSize)] = &[
        ("A4  (210 × 297 mm)",   PaperSize::A4),
        ("A5  (148 × 210 mm)",   PaperSize::A5),
        ("Letter (8.5 × 11 in)", PaperSize::Letter),
        ("Legal (8.5 × 14 in)",  PaperSize::Legal),
    ];

    let mut first: Option<CheckButton> = None;
    let radio_box = GBox::new(Orientation::Vertical, 4);
    for (label, _) in sizes {
        let btn = match &first {
            None    => CheckButton::with_label(label),
            Some(f) => CheckButton::builder().label(*label).group(f).build(),
        };
        if first.is_none() { btn.set_active(true); first = Some(btn.clone()); }
        radio_box.append(&btn);
    }
    vbox.append(&radio_box);

    // ── Create button ──────────────────────────────────────────────────────
    let create_btn = Button::with_label("Create Document");
    create_btn.add_css_class("suggested-action");
    create_btn.add_css_class("pill");

    {
        let app_ref    = app.clone();
        let d_ref      = dialog.clone();
        let parent_ref = parent.clone();
        let entry_ref  = title_entry.clone();
        let radios_ref = radio_box.clone();

        create_btn.connect_clicked(move |_| {
            let title = entry_ref.text().to_string();
            let title = if title.is_empty() { "Untitled".to_string() } else { title };

            // Determine chosen paper size.
            let mut chosen = PaperSize::A4;
            let mut i = 0usize;
            let mut child = radios_ref.first_child();
            while let Some(w) = child {
                if w.downcast_ref::<CheckButton>().map(|b| b.is_active()).unwrap_or(false) {
                    chosen = sizes[i].1.clone();
                }
                child = w.next_sibling();
                i += 1;
            }

            let doc = Document::new(title.clone(), PageSettings {
                paper_size: chosen,
                ..Default::default()
            });

            // Assign a save path immediately so auto-save works from the start.
            let docs_dir = documents_dir();
            let _ = std::fs::create_dir_all(&docs_dir);
            let safe_title: String = title.chars()
                .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
                .take(40)
                .collect();
            let filename = format!("{}-{}.folio", safe_title.trim(), doc.id);
            let save_path = docs_dir.join(filename);

            // Create engine, take initial checkpoint, and save.
            let mut engine = CrdtEngine::new();
            engine.checkpoint(&doc).ok();
            folio_core::format::save_folio(&save_path, &doc, &engine, &[]).ok();

            let canvas = crate::canvas::EditorCanvas::from_loaded(
                doc, save_path, engine,
            );
            let win = DocumentWindow::from_canvas(&app_ref, canvas);
            win.present();

            // Close launch window if it is the parent.
            parent_ref.close();
            d_ref.close();
        });
    }
    vbox.append(&create_btn);

    tv.set_content(Some(&vbox));
    dialog.set_child(Some(&tv));
    dialog.present(Some(parent));
}
