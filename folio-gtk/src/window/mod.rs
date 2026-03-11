use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk4::{ScrolledWindow, Box as GBox, Align, Orientation, PolicyType};
use folio_core::Document;
use crate::canvas::EditorCanvas;

pub struct DocumentWindow {
    pub window: ApplicationWindow,
    pub canvas: EditorCanvas,
}

impl DocumentWindow {
    pub fn new(app: &Application, doc: Document) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Folio")
            .default_width(1060)
            .default_height(820)
            .build();

        // ── Shell ──────────────────────────────────────────────────────────
        let toolbar_view = ToolbarView::new();
        toolbar_view.add_top_bar(&HeaderBar::new());

        // ── Scrolled canvas ────────────────────────────────────────────────
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();

        let canvas = EditorCanvas::new(doc);

        // Centre the drawing area horizontally inside the scroll view.
        let centre = GBox::new(Orientation::Horizontal, 0);
        centre.set_halign(Align::Center);
        centre.set_valign(Align::Start);
        centre.set_margin_top(0);
        centre.append(&canvas.widget);

        scroll.set_child(Some(&centre));
        toolbar_view.set_content(Some(&scroll));
        window.set_content(Some(&toolbar_view));

        // Canvas must be focusable to receive key events.
        canvas.widget.grab_focus();

        DocumentWindow { window, canvas }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
