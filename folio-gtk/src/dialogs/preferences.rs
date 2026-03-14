//! Preferences dialog — Phase 9.
//!
//! App-wide settings: cursor style, colour scheme, word goal, font management.

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Button, CheckButton, DrawingArea,
           Entry, Label, Orientation, Align, Window};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, HeaderBar, ToolbarView};
use crate::canvas::EditorState;
use crate::canvas::cursor::CursorStyle;

pub fn show(
    parent: &Window,
    state:  Rc<RefCell<EditorState>>,
    canvas: DrawingArea,
) {
    let dialog = libadwaita::Dialog::builder()
        .title("Preferences")
        .content_width(400)
        .build();

    let tv = ToolbarView::new();
    tv.add_top_bar(&HeaderBar::new());

    let vbox = GBox::new(Orientation::Vertical, 0);

    // ── Cursor style ──────────────────────────────────────────────────────
    let cur_lbl = Label::new(Some("Cursor Style"));
    cur_lbl.add_css_class("heading");
    cur_lbl.set_halign(Align::Start);
    cur_lbl.set_margin_start(16); cur_lbl.set_margin_top(16); cur_lbl.set_margin_bottom(4);
    vbox.append(&cur_lbl);

    let cur_lb = gtk4::ListBox::new();
    cur_lb.add_css_class("boxed-list");
    cur_lb.set_margin_start(16); cur_lb.set_margin_end(16);
    cur_lb.set_selection_mode(gtk4::SelectionMode::None);

    let cur_styles: &[(&str, CursorStyle)] = &[
        ("I-Beam (default)", CursorStyle::IBeam),
        ("Block",            CursorStyle::Block),
        ("Underscore",       CursorStyle::Underscore),
    ];
    let cur_current = state.borrow().cursor_style;
    let mut first: Option<CheckButton> = None;
    for (label, style) in cur_styles {
        let row = ActionRow::builder().title(*label).build();
        let btn = match &first {
            None    => CheckButton::new(),
            Some(f) => CheckButton::builder().group(f).build(),
        };
        if first.is_none() { first = Some(btn.clone()); }
        if *style == cur_current { btn.set_active(true); }
        btn.set_valign(Align::Center);
        {
            let s = state.clone();
            let c = canvas.clone();
            let st = *style;
            btn.connect_toggled(move |b| {
                if b.is_active() {
                    s.borrow_mut().cursor_style = st;
                    c.queue_draw();
                }
            });
        }
        row.add_prefix(&btn);
        cur_lb.append(&row);
    }
    vbox.append(&cur_lb);

    // ── Appearance ────────────────────────────────────────────────────────
    let app_lbl = Label::new(Some("Appearance"));
    app_lbl.add_css_class("heading");
    app_lbl.set_halign(Align::Start);
    app_lbl.set_margin_start(16); app_lbl.set_margin_top(16); app_lbl.set_margin_bottom(4);
    vbox.append(&app_lbl);

    let app_lb = gtk4::ListBox::new();
    app_lb.add_css_class("boxed-list");
    app_lb.set_margin_start(16); app_lb.set_margin_end(16);
    app_lb.set_selection_mode(gtk4::SelectionMode::None);

    let dark_row = ActionRow::builder().title("Dark Mode").subtitle("Follow system when off").build();
    let dark_sw  = gtk4::Switch::new();
    dark_sw.set_valign(Align::Center);
    {
        let mgr = libadwaita::StyleManager::default();
        dark_sw.set_active(mgr.is_dark());
        dark_sw.connect_state_set(|_, active| {
            libadwaita::StyleManager::default().set_color_scheme(
                if active { libadwaita::ColorScheme::ForceDark }
                else      { libadwaita::ColorScheme::Default }
            );
            glib::Propagation::Stop
        });
    }
    dark_row.add_suffix(&dark_sw);
    app_lb.append(&dark_row);
    vbox.append(&app_lb);

    // ── Fonts ─────────────────────────────────────────────────────────────
    let font_lbl = Label::new(Some("Fonts"));
    font_lbl.add_css_class("heading");
    font_lbl.set_halign(Align::Start);
    font_lbl.set_margin_start(16); font_lbl.set_margin_top(16); font_lbl.set_margin_bottom(4);
    vbox.append(&font_lbl);

    let font_lb = gtk4::ListBox::new();
    font_lb.add_css_class("boxed-list");
    font_lb.set_margin_start(16); font_lb.set_margin_end(16);
    font_lb.set_selection_mode(gtk4::SelectionMode::None);

    // Install font from file
    let install_row = ActionRow::builder()
        .title("Install Font File")
        .subtitle("Drag a .ttf or .otf file to install")
        .build();
    let install_btn = Button::with_label("Browse…");
    install_btn.set_valign(Align::Center);
    install_btn.add_css_class("flat");
    {
        let p = parent.clone();
        install_btn.connect_clicked(move |_| {
            let filter = gtk4::FileFilter::new();
            filter.add_pattern("*.ttf");
            filter.add_pattern("*.otf");
            filter.set_name(Some("Font files"));
            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);
            let fd = gtk4::FileDialog::builder()
                .title("Install Font")
                .filters(&filters)
                .build();
            let pp = p.clone();
            glib::spawn_future_local(async move {
                if let Ok(file) = fd.open_future(Some(&pp)).await {
                    if let Some(path) = file.path() {
                        match crate::fonts::install_font_file(&path) {
                            Ok(dest) => eprintln!("Font installed: {}", dest.display()),
                            Err(e)   => eprintln!("Font install failed: {e}"),
                        }
                    }
                }
            });
        });
    }
    install_row.add_suffix(&install_btn);
    font_lb.append(&install_row);

    // Download from CDN URL
    let cdn_row   = ActionRow::builder().title("Download Font by URL").build();
    let cdn_entry = Entry::builder().placeholder_text("https://…/Font.ttf").hexpand(true).build();
    cdn_entry.set_valign(Align::Center);
    let cdn_btn   = Button::with_label("Download");
    cdn_btn.set_valign(Align::Center);
    cdn_btn.add_css_class("flat");
    {
        let e = cdn_entry.clone();
        cdn_btn.connect_clicked(move |_| {
            let url = e.text().to_string();
            if url.is_empty() { return; }
            glib::spawn_future_local(async move {
                match crate::fonts::download_font(&url).await {
                    Ok(dest) => eprintln!("Font downloaded: {}", dest.display()),
                    Err(err) => eprintln!("Font download failed: {err}"),
                }
            });
        });
    }
    cdn_row.add_suffix(&cdn_entry);
    cdn_row.add_suffix(&cdn_btn);
    font_lb.append(&cdn_row);
    vbox.append(&font_lb);

    // ── Close button ──────────────────────────────────────────────────────
    let close_btn = Button::with_label("Done");
    close_btn.add_css_class("pill");
    close_btn.set_margin_top(16);
    close_btn.set_margin_bottom(16);
    close_btn.set_margin_start(16);
    close_btn.set_margin_end(16);
    {
        let d = dialog.clone();
        close_btn.connect_clicked(move |_| d.close());
    }
    vbox.append(&close_btn);

    let scroll = gtk4::ScrolledWindow::builder().vexpand(true).build();
    scroll.set_child(Some(&vbox));
    tv.set_content(Some(&scroll));
    dialog.set_child(Some(&tv));
    dialog.present(Some(parent));
}
