use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk4::{Box, Button, Label, Orientation, Align};

/// The document launch / browser screen.
/// Shown on startup and when the user closes a document.
pub struct LaunchWindow {
    pub window: ApplicationWindow,
}

impl LaunchWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Folio")
            .default_width(780)
            .default_height(540)
            .build();

        let toolbar_view = ToolbarView::new();
        let header = HeaderBar::new();
        toolbar_view.add_top_bar(&header);

        // Main content area
        let content = Box::new(Orientation::Vertical, 0);
        content.set_valign(Align::Center);
        content.set_halign(Align::Center);
        content.set_spacing(24);

        // Wordmark
        let wordmark = Label::new(Some("Folio"));
        wordmark.add_css_class("title-1");

        // Subtitle
        let sub = Label::new(Some("Your documents, beautifully."));
        sub.add_css_class("body");

        // Buttons row
        let btn_row = Box::new(Orientation::Horizontal, 12);
        btn_row.set_halign(Align::Center);

        let btn_new = Button::with_label("New Document");
        btn_new.add_css_class("suggested-action");
        btn_new.add_css_class("pill");

        let btn_open = Button::with_label("Open…");
        btn_open.add_css_class("pill");

        btn_row.append(&btn_new);
        btn_row.append(&btn_open);

        content.append(&wordmark);
        content.append(&sub);
        content.append(&btn_row);

        toolbar_view.set_content(Some(&content));
        window.set_content(Some(&toolbar_view));

        // Wire up New Document button
        {
            let win_ref = window.clone();
            let app_ref = app.clone();
            btn_new.connect_clicked(move |_| {
                crate::dialogs::new_document::show(&app_ref, &win_ref);
            });
        }

        // Wire up Open button
        {
            let win_ref = window.clone();
            let app_ref = app.clone();
            btn_open.connect_clicked(move |_| {
                crate::dialogs::open_document::show(&app_ref, &win_ref);
            });
        }

        LaunchWindow { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
