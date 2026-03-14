use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Orientation, SpinButton, Adjustment, Align,
           Button, DropDown, StringList};
use libadwaita::prelude::*;
use libadwaita::ActionRow;
use crate::canvas::EditorState;

pub struct FormatPanel {
    pub widget: GBox,
}

const FONT_NAMES: &[&str] = &[
    "Lora", "Newsreader", "Fraunces", "Inter", "IBM Plex Mono",
];

impl FormatPanel {
    pub fn new(state: Rc<RefCell<EditorState>>, canvas: gtk4::DrawingArea) -> Self {
        let vbox = GBox::new(Orientation::Vertical, 0);

        // ── Font section ──────────────────────────────────────────────────
        Self::section_label(&vbox, "Font");
        let font_lb = Self::listbox(&vbox);

        // Font family dropdown
        let font_row = ActionRow::builder().title("Family").build();
        let font_list = StringList::new(FONT_NAMES);
        let font_dd = DropDown::new(Some(font_list), gtk4::Expression::NONE);
        font_dd.set_valign(Align::Center);
        {
            let cur = state.borrow().doc.typography.font_family.clone();
            let idx = FONT_NAMES.iter().position(|&f| f == cur).unwrap_or(0);
            font_dd.set_selected(idx as u32);
        }
        {
            let s = state.clone(); let c = canvas.clone();
            font_dd.connect_selected_notify(move |dd| {
                if let Some(obj) = dd.selected_item() {
                    if let Some(sobj) = obj.downcast_ref::<gtk4::StringObject>() {
                        let mut st = s.borrow_mut();
                        st.doc.typography.font_family = sobj.string().to_string();
                        st.invalidate_layout();
                        c.queue_draw();
                    }
                }
            });
        }
        font_row.add_suffix(&font_dd);
        font_lb.append(&font_row);

        // Font size
        let size_row = ActionRow::builder().title("Size (pt)").build();
        let size_adj = Adjustment::new(12.0, 0.1, 500.0, 0.5, 1.0, 0.0);
        let size_spin = SpinButton::new(Some(&size_adj), 0.5, 1);
        size_spin.set_valign(Align::Center);
        {
            let v = state.borrow().doc.typography.font_size_pt;
            size_spin.set_value(v);
            let s = state.clone(); let c = canvas.clone();
            size_spin.connect_value_changed(move |sp| {
                let mut st = s.borrow_mut();
                st.doc.typography.font_size_pt = sp.value();
                st.invalidate_layout(); c.queue_draw();
            });
        }
        size_row.add_suffix(&size_spin);
        font_lb.append(&size_row);

        // Line height
        let lh_row = ActionRow::builder().title("Line Height").build();
        let lh_adj = Adjustment::new(1.82, 1.0, 3.0, 0.05, 0.1, 0.0);
        let lh_spin = SpinButton::new(Some(&lh_adj), 0.05, 2);
        lh_spin.set_valign(Align::Center);
        {
            let v = state.borrow().doc.typography.line_height;
            lh_spin.set_value(v);
            let s = state.clone(); let c = canvas.clone();
            lh_spin.connect_value_changed(move |sp| {
                let mut st = s.borrow_mut();
                st.doc.typography.line_height = sp.value();
                st.invalidate_layout(); c.queue_draw();
            });
        }
        lh_row.add_suffix(&lh_spin);
        font_lb.append(&lh_row);

        // Letter spacing
        let ls_row = ActionRow::builder().title("Tracking (em)").build();
        let ls_adj = Adjustment::new(0.0, -0.1, 0.5, 0.005, 0.01, 0.0);
        let ls_spin = SpinButton::new(Some(&ls_adj), 0.005, 3);
        ls_spin.set_valign(Align::Center);
        {
            let v = state.borrow().doc.typography.tracking_em;
            ls_spin.set_value(v);
            let s = state.clone(); let c = canvas.clone();
            ls_spin.connect_value_changed(move |sp| {
                let mut st = s.borrow_mut();
                st.doc.typography.tracking_em = sp.value();
                st.invalidate_layout(); c.queue_draw();
            });
        }
        ls_row.add_suffix(&ls_spin);
        font_lb.append(&ls_row);

        // ── Page section ──────────────────────────────────────────────────
        Self::section_label(&vbox, "Page");
        let page_lb = Self::listbox(&vbox);

        // Margins (top)
        let mt_row = ActionRow::builder().title("Margin top (mm)").build();
        let mt_adj = Adjustment::new(25.4, 0.0, 100.0, 1.0, 5.0, 0.0);
        let mt_spin = SpinButton::new(Some(&mt_adj), 1.0, 1);
        mt_spin.set_valign(Align::Center);
        {
            let v = state.borrow().doc.page.margins.top_mm;
            mt_spin.set_value(v);
            let s = state.clone(); let c = canvas.clone();
            mt_spin.connect_value_changed(move |sp| {
                let mut st = s.borrow_mut();
                st.doc.page.margins.top_mm = sp.value();
                st.invalidate_layout(); c.queue_draw();
            });
        }
        mt_row.add_suffix(&mt_spin);
        page_lb.append(&mt_row);

        // Margins (bottom)
        let mb_row = ActionRow::builder().title("Margin bottom (mm)").build();
        let mb_adj = Adjustment::new(25.4, 0.0, 100.0, 1.0, 5.0, 0.0);
        let mb_spin = SpinButton::new(Some(&mb_adj), 1.0, 1);
        mb_spin.set_valign(Align::Center);
        {
            let v = state.borrow().doc.page.margins.bottom_mm;
            mb_spin.set_value(v);
            let s = state.clone(); let c = canvas.clone();
            mb_spin.connect_value_changed(move |sp| {
                let mut st = s.borrow_mut();
                st.doc.page.margins.bottom_mm = sp.value();
                st.invalidate_layout(); c.queue_draw();
            });
        }
        mb_row.add_suffix(&mb_spin);
        page_lb.append(&mb_row);

        // Margins (left/right together)
        let ml_row = ActionRow::builder().title("Margin sides (mm)").build();
        let ml_adj = Adjustment::new(25.4, 0.0, 100.0, 1.0, 5.0, 0.0);
        let ml_spin = SpinButton::new(Some(&ml_adj), 1.0, 1);
        ml_spin.set_valign(Align::Center);
        {
            let v = state.borrow().doc.page.margins.left_mm;
            ml_spin.set_value(v);
            let s = state.clone(); let c = canvas.clone();
            ml_spin.connect_value_changed(move |sp| {
                let mut st = s.borrow_mut();
                st.doc.page.margins.left_mm  = sp.value();
                st.doc.page.margins.right_mm = sp.value();
                st.invalidate_layout(); c.queue_draw();
            });
        }
        ml_row.add_suffix(&ml_spin);
        page_lb.append(&ml_row);

        // Page setup button
        let setup_btn = Button::with_label("Paper Size & Orientation…");
        setup_btn.add_css_class("flat");
        setup_btn.set_margin_start(12);
        setup_btn.set_margin_end(12);
        setup_btn.set_margin_top(4);
        setup_btn.set_margin_bottom(8);
        {
            let s = state.clone();
            let c = canvas.clone();
            setup_btn.connect_clicked(move |btn| {
                if let Some(win) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                    crate::dialogs::page_setup::show(&win, s.clone(), c.clone());
                }
            });
        }
        vbox.append(&setup_btn);

        FormatPanel { widget: vbox }
    }

    fn section_label(vbox: &GBox, text: &str) {
        let lbl = gtk4::Label::new(Some(text));
        lbl.add_css_class("heading");
        lbl.set_halign(Align::Start);
        lbl.set_margin_start(12);
        lbl.set_margin_top(12);
        lbl.set_margin_bottom(4);
        vbox.append(&lbl);
    }

    fn listbox(vbox: &GBox) -> gtk4::ListBox {
        let lb = gtk4::ListBox::new();
        lb.add_css_class("boxed-list");
        lb.set_margin_start(12);
        lb.set_margin_end(12);
        lb.set_selection_mode(gtk4::SelectionMode::None);
        vbox.append(&lb);
        lb
    }
}
