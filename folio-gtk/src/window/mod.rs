use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, OverlaySplitView, ToolbarView};
use gtk4::{Box as GBox, Button, Orientation, ScrolledWindow, Align,
           PolicyType, ToggleButton, PackType};
use glib;
use gio;
use std::time::Duration;
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

        // Export button
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

        // Focus mode toggle
        let focus_btn = ToggleButton::new();
        focus_btn.set_icon_name("view-fullscreen-symbolic");
        focus_btn.set_tooltip_text(Some("Focus Mode (Ctrl+Shift+F)"));
        focus_btn.add_css_class("flat");
        {
            let s = state.clone();
            let c = canvas.widget.clone();
            let tb = toolbar.widget.clone();
            let sp = split.clone();
            focus_btn.connect_toggled(move |b| {
                let active = b.is_active();
                s.borrow_mut().focus_mode = active;
                tb.set_visible(!active);
                // Hide sidebar in focus mode
                if active { sp.set_show_sidebar(false); }
                c.queue_draw();
            });
        }
        header.pack_end(&focus_btn);

        // Typewriter mode toggle
        let typo_btn = ToggleButton::new();
        typo_btn.set_icon_name("format-text-direction-ltr-symbolic");
        typo_btn.set_tooltip_text(Some("Typewriter Mode (Ctrl+Shift+T)"));
        typo_btn.add_css_class("flat");
        {
            let s = state.clone();
            typo_btn.connect_toggled(move |b| {
                s.borrow_mut().typewriter_mode = b.is_active();
            });
        }
        header.pack_end(&typo_btn);

        // Dark mode toggle
        let dark_btn = ToggleButton::new();
        dark_btn.set_icon_name("weather-clear-night-symbolic");
        dark_btn.set_tooltip_text(Some("Dark Mode"));
        dark_btn.add_css_class("flat");
        dark_btn.connect_toggled(|b| {
            let mgr = libadwaita::StyleManager::default();
            mgr.set_color_scheme(if b.is_active() {
                libadwaita::ColorScheme::ForceDark
            } else {
                libadwaita::ColorScheme::PreferLight
            });
        });
        header.pack_end(&dark_btn);

        let sidebar_btn = ToggleButton::new();
        sidebar_btn.set_icon_name("view-sidebar-end-symbolic");
        sidebar_btn.add_css_class("flat");
        sidebar_btn.set_tooltip_text(Some("Toggle sidebar"));
        let split_ref = split.clone();
        sidebar_btn.connect_toggled(move |b| split_ref.set_show_sidebar(b.is_active()));
        header.pack_end(&sidebar_btn);

        // Preferences button
        let prefs_btn = Button::from_icon_name("preferences-system-symbolic");
        prefs_btn.set_tooltip_text(Some("Preferences"));
        prefs_btn.add_css_class("flat");
        {
            let s = state.clone();
            let c = canvas.widget.clone();
            let w = window.clone();
            prefs_btn.connect_clicked(move |_| {
                crate::dialogs::preferences::show(
                    w.upcast_ref::<gtk4::Window>(), s.clone(), c.clone()
                );
            });
        }
        header.pack_end(&prefs_btn);

        // ── Shell ──────────────────────────────────────────────────────────
        let tv = ToolbarView::new();
        tv.add_top_bar(&header);
        tv.add_top_bar(&toolbar.widget);
        tv.set_content(Some(&split));

        window.set_content(Some(&tv));

        // Typewriter scroll: every 100ms, if typewriter_mode is on,
        // scroll so the cursor block is vertically centred in the viewport.
        {
            let s  = state.clone();
            let sc = scroll.clone();
            glib::timeout_add_local(Duration::from_millis(100), move || {
                let st = s.borrow();
                if !st.typewriter_mode { return glib::ControlFlow::Continue; }
                let cache_borrow = st.layout_cache.borrow();
                if let Some(cache) = cache_borrow.as_ref() {
                    let idx = st.cursor.block_idx;
                    if let Some(cb) = cache.blocks.get(idx) {
                        let block_mid   = (cb.y_top + cb.y_bot) / 2.0;
                        let view_height = sc.height() as f64;
                        let target      = (block_mid - view_height / 2.0).max(0.0);
                        if let Some(vadj) = sc.vadjustment() {
                            vadj.set_value(target);
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }

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
        ("Plain Text (.txt)", "*.txt",  "txt"),
        ("Markdown (.md)",    "*.md",   "md"),
        ("HTML (.html)",      "*.html", "html"),
        ("PDF (.pdf)",        "*.pdf",  "pdf"),
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
                "pdf"  => { /* handled separately below */ String::new() }
                _      => return,
            };
            let ext      = fmt_str.clone();
            let content2 = content.clone();
            let p2       = p_ref.clone();
            let d2       = d_ref.clone();
            let dc3      = doc_clone.clone();
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
                        if ext == "pdf" {
                            folio_core::export::export_pdf(&dc3, &path).ok();
                        } else {
                            std::fs::write(&path, content2.as_bytes()).ok();
                        }
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
