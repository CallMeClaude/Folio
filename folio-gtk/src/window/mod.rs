use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, OverlaySplitView, ToolbarView};
use gtk4::{Box as GBox, Button, FileDialog, FileFilter, ListStore,
           Orientation, ScrolledWindow, Align, PolicyType,
           ToggleButton, PackType};
use gio;
use folio_core::Document;
use crate::canvas::EditorCanvas;
use crate::toolbar::FormattingToolbar;
use crate::sidebar::DocumentSidebar;

pub struct DocumentWindow {
    pub window: ApplicationWindow,
}

impl DocumentWindow {
    /// Open a brand-new (unsaved) document.
    pub fn new(app: &Application, doc: Document) -> Self {
        Self::from_canvas(app, EditorCanvas::new(doc))
    }

    /// Open a document that was already loaded from disk (with engine + path).
    pub fn from_canvas(app: &Application, canvas: EditorCanvas) -> Self {
        let doc   = canvas.state.borrow().doc.clone();
        let title = if doc.title.is_empty() { "Untitled".to_string() } else { doc.title.clone() };

        let window = ApplicationWindow::builder()
            .application(app)
            .title(&title)
            .default_width(1120)
            .default_height(860)
            .build();

        // canvas already constructed — just use it directly

        // ── Shared state refs for toolbar + sidebar ────────────────────────
        let state = canvas.state.clone();

        // ── Secondary toolbar ──────────────────────────────────────────────
        let toolbar = FormattingToolbar::new(state.clone(), canvas.widget.clone());

        // ── Sidebar ────────────────────────────────────────────────────────
        let sidebar = DocumentSidebar::new(state.clone(), canvas.widget.clone());

        // ── Scrolled canvas (content side) ─────────────────────────────────
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true).vexpand(true)
            .build();
        let centre = GBox::new(Orientation::Horizontal, 0);
        centre.set_halign(Align::Center);
        centre.set_valign(Align::Start);
        centre.append(&canvas.widget);
        scroll.set_child(Some(&centre));

        // ── Split view ─────────────────────────────────────────────────────
        let split = OverlaySplitView::new();
        split.set_sidebar_position(PackType::End);
        split.set_sidebar_width_fraction(0.23);
        split.set_show_sidebar(false);
        split.set_content(Some(&scroll));
        split.set_sidebar(Some(&sidebar.widget));

        // ── Header bar ─────────────────────────────────────────────────────
        let header = HeaderBar::new();

        // Find button
        let find_btn = Button::from_icon_name("edit-find-symbolic");
        find_btn.set_tooltip_text(Some("Find & Replace (Ctrl+F)"));
        find_btn.add_css_class("flat");
        {
            let s = state.clone();
            let c = canvas.widget.clone();
            let w = window.clone();
            find_btn.connect_clicked(move |_| {
                crate::dialogs::find_replace::show(
                    w.upcast_ref::<gtk4::Window>(), s.clone(), c.clone()
                );
            });
        }
        header.pack_start(&find_btn);

        // Export button (menu)
        let export_btn = Button::from_icon_name("document-save-as-symbolic");
        export_btn.set_tooltip_text(Some("Export document"));
        export_btn.add_css_class("flat");
        {
            let s = state.clone();
            let w = window.clone();
            export_btn.connect_clicked(move |_| {
                show_export_dialog(w.upcast_ref::<gtk4::Window>(), &s.borrow().doc);
            });
        }
        header.pack_start(&export_btn);

        let sidebar_btn = ToggleButton::new();
        sidebar_btn.set_icon_name("view-sidebar-end-symbolic");
        sidebar_btn.add_css_class("flat");
        sidebar_btn.set_tooltip_text(Some("Toggle sidebar"));
        let split_ref = split.clone();
        sidebar_btn.connect_toggled(move |b| split_ref.set_show_sidebar(b.is_active()));
        header.pack_end(&sidebar_btn);

        // ── Shell ──────────────────────────────────────────────────────────
        let tv = ToolbarView::new();
        tv.add_top_bar(&header);
        tv.add_top_bar(&toolbar.widget);
        tv.set_content(Some(&split));

        window.set_content(Some(&tv));

        // Grab keyboard focus after the window is actually shown/mapped.
        {
            let c = canvas.widget.clone();
            window.connect_show(move |_| { c.grab_focus(); });
        }

        DocumentWindow { window }
    }

    pub fn present(&self) { self.window.present(); }
}

// ── Export dialog ─────────────────────────────────────────────────────────────

fn show_export_dialog(parent: &gtk4::Window, doc: &folio_core::Document) {
    use folio_core::export::{export_txt, export_md, export_html};

    // Format choices presented as an AdwDialog.
    let dialog = libadwaita::Dialog::builder()
        .title("Export Document")
        .content_width(320)
        .build();

    let tv = libadwaita::ToolbarView::new();
    tv.add_top_bar(&libadwaita::HeaderBar::new());

    let vbox = GBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    let formats: &[(&str, &str, &str)] = &[
        ("Plain Text (.txt)", "*.txt", "txt"),
        ("Markdown (.md)",    "*.md",  "md"),
        ("HTML (.html)",      "*.html","html"),
    ];

    for (label, _glob, fmt) in formats {
        let btn = gtk4::Button::with_label(label);
        btn.add_css_class("flat");
        let doc_clone = doc.clone();
        let d_ref     = dialog.clone();
        let p_ref     = parent.clone();
        let fmt_str   = fmt.to_string();
        btn.connect_clicked(move |_| {
            let content = match fmt_str.as_str() {
                "txt"  => export_txt(&doc_clone).unwrap_or_default(),
                "md"   => export_md(&doc_clone).unwrap_or_default(),
                "html" => export_html(&doc_clone).unwrap_or_default(),
                _      => return,
            };
            let ext      = fmt_str.clone();
            let content2 = content.clone();
            let p2       = p_ref.clone();
            let d2       = d_ref.clone();
            glib::spawn_future_local(async move {
                let filter = gtk4::FileFilter::new();
                filter.add_pattern(&format!("*.{}", ext));
                let filters = gio::ListStore::new::<gtk4::FileFilter>();
                filters.append(&filter);
                let fd = gtk4::FileDialog::builder()
                    .title("Save Export")
                    .initial_name(&format!("document.{}", ext))
                    .filters(&filters)
                    .build();
                if let Ok(file) = fd.save_future(Some(&p2)).await {
                    if let Some(path) = file.path() {
                        std::fs::write(&path, content2.as_bytes()).ok();
                    }
                }
                d2.close();
            });
        });
        vbox.append(&btn);
    }

    tv.set_content(Some(&vbox));
    dialog.set_child(Some(&tv));
    dialog.present(Some(parent));
}
