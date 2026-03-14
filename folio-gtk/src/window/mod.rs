use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, OverlaySplitView, ToolbarView};
use gtk4::{Box as GBox, Orientation, ScrolledWindow, Align, PolicyType, ToggleButton, PackType};
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
