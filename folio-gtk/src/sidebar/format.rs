use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Orientation, SpinButton, Adjustment, Label, Align};
use libadwaita::prelude::*;
use libadwaita::ActionRow;
use crate::canvas::EditorState;

pub struct FormatPanel {
    pub widget: GBox,
}

impl FormatPanel {
    pub fn new(state: Rc<RefCell<EditorState>>, canvas: gtk4::DrawingArea) -> Self {
        let vbox = GBox::new(Orientation::Vertical, 0);

        // ── Typography section ─────────────────────────────────────────────
        let sec_label = Label::new(Some("Typography"));
        sec_label.add_css_class("heading");
        sec_label.set_halign(Align::Start);
        sec_label.set_margin_start(12);
        sec_label.set_margin_top(12);
        sec_label.set_margin_bottom(4);
        vbox.append(&sec_label);

        let listbox = gtk4::ListBox::new();
        listbox.add_css_class("boxed-list");
        listbox.set_margin_start(12);
        listbox.set_margin_end(12);
        listbox.set_selection_mode(gtk4::SelectionMode::None);

        // Font size spinner
        let size_row = ActionRow::builder().title("Font Size").build();
        let adj = Adjustment::new(12.0, 6.0, 72.0, 0.5, 1.0, 0.0);
        let spin = SpinButton::new(Some(&adj), 0.5, 1);
        spin.set_valign(Align::Center);
        {
            let s = state.clone();
            let cur_size = s.borrow().doc.typography.font_size_pt;
            spin.set_value(cur_size);
            let c = canvas.clone();
            spin.connect_value_changed(move |sp| {
                s.borrow_mut().doc.typography.font_size_pt = sp.value();
                c.queue_draw();
            });
        }
        size_row.add_suffix(&spin);
        listbox.append(&size_row);

        // Line height
        let lh_row = ActionRow::builder().title("Line Height").build();
        let lh_adj = Adjustment::new(1.82, 1.0, 3.0, 0.05, 0.1, 0.0);
        let lh_spin = SpinButton::new(Some(&lh_adj), 0.05, 2);
        lh_spin.set_valign(Align::Center);
        {
            let s = state.clone();
            let c = canvas.clone();
            lh_spin.connect_value_changed(move |sp| {
                s.borrow_mut().doc.typography.line_height = sp.value();
                c.queue_draw();
            });
        }
        lh_row.add_suffix(&lh_spin);
        listbox.append(&lh_row);

        vbox.append(&listbox);
        FormatPanel { widget: vbox }
    }
}
