use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Button, CheckButton, DrawingArea, Label,
           Orientation, Align, SpinButton, Adjustment, Window};
use libadwaita::prelude::*;
use libadwaita::{HeaderBar, ToolbarView};
use folio_core::document::page::{PaperSize, Orientation as DocOrientation};
use crate::canvas::EditorState;

pub fn show(
    parent: &Window,
    state:  Rc<RefCell<EditorState>>,
    canvas: DrawingArea,
) {
    let dialog = libadwaita::Dialog::builder()
        .title("Page Setup")
        .content_width(380)
        .build();

    let tv = ToolbarView::new();
    tv.add_top_bar(&HeaderBar::new());

    let vbox = GBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    // ── Paper size ────────────────────────────────────────────────────────
    let size_lbl = Label::new(Some("Paper Size"));
    size_lbl.add_css_class("heading");
    size_lbl.set_halign(Align::Start);
    vbox.append(&size_lbl);

    let sizes: &[(&str, PaperSize)] = &[
        ("A4  (210 × 297 mm)",   PaperSize::A4),
        ("A5  (148 × 210 mm)",   PaperSize::A5),
        ("A3  (297 × 420 mm)",   PaperSize::A3),
        ("Letter (8.5 × 11 in)", PaperSize::Letter),
        ("Legal (8.5 × 14 in)",  PaperSize::Legal),
        ("Tabloid (11 × 17 in)", PaperSize::Tabloid),
    ];

    let cur_size = state.borrow().doc.page.paper_size.clone();
    let mut first_btn: Option<CheckButton> = None;
    let radio_box = GBox::new(Orientation::Vertical, 4);
    for (label, paper) in sizes {
        let btn = match &first_btn {
            None    => CheckButton::with_label(label),
            Some(f) => CheckButton::builder().label(*label).group(f).build(),
        };
        if first_btn.is_none() { first_btn = Some(btn.clone()); }
        if *paper == cur_size  { btn.set_active(true); }
        radio_box.append(&btn);
    }
    vbox.append(&radio_box);

    // ── Orientation ───────────────────────────────────────────────────────
    let ori_lbl = Label::new(Some("Orientation"));
    ori_lbl.add_css_class("heading");
    ori_lbl.set_halign(Align::Start);
    vbox.append(&ori_lbl);

    let portrait_btn  = CheckButton::with_label("Portrait");
    let landscape_btn = CheckButton::builder()
        .label("Landscape")
        .group(&portrait_btn)
        .build();
    match state.borrow().doc.page.orientation {
        DocOrientation::Portrait  => portrait_btn.set_active(true),
        DocOrientation::Landscape => landscape_btn.set_active(true),
    }
    let ori_box = GBox::new(Orientation::Horizontal, 16);
    ori_box.append(&portrait_btn);
    ori_box.append(&landscape_btn);
    vbox.append(&ori_box);

    // ── Margins ───────────────────────────────────────────────────────────
    let marg_lbl = Label::new(Some("Margins (mm)"));
    marg_lbl.add_css_class("heading");
    marg_lbl.set_halign(Align::Start);
    vbox.append(&marg_lbl);

    let margin_grid = gtk4::Grid::new();
    margin_grid.set_column_spacing(8);
    margin_grid.set_row_spacing(6);

    let mk_spin = |val: f64| -> SpinButton {
        let adj = Adjustment::new(val, 0.0, 150.0, 1.0, 5.0, 0.0);
        SpinButton::new(Some(&adj), 1.0, 1)
    };
    let pg = &state.borrow().doc.page;
    let top_spin    = mk_spin(pg.margins.top_mm);
    let bot_spin    = mk_spin(pg.margins.bottom_mm);
    let left_spin   = mk_spin(pg.margins.left_mm);
    let right_spin  = mk_spin(pg.margins.right_mm);

    for (col, lbl) in [(0, "Top"), (1, "Bottom"), (2, "Left"), (3, "Right")] {
        let l = Label::new(Some(lbl));
        l.add_css_class("caption");
        margin_grid.attach(&l, col, 0, 1, 1);
    }
    margin_grid.attach(&top_spin,   0, 1, 1, 1);
    margin_grid.attach(&bot_spin,   1, 1, 1, 1);
    margin_grid.attach(&left_spin,  2, 1, 1, 1);
    margin_grid.attach(&right_spin, 3, 1, 1, 1);
    vbox.append(&margin_grid);

    // ── Apply button ──────────────────────────────────────────────────────
    let apply_btn = Button::with_label("Apply");
    apply_btn.add_css_class("suggested-action");
    apply_btn.add_css_class("pill");
    {
        let s          = state.clone();
        let c          = canvas.clone();
        let d          = dialog.clone();
        let radio_ref  = radio_box.clone();
        let port_ref   = portrait_btn.clone();
        let tops       = top_spin.clone();
        let bots       = bot_spin.clone();
        let lefts      = left_spin.clone();
        let rights     = right_spin.clone();

        apply_btn.connect_clicked(move |_| {
            let mut st = s.borrow_mut();

            // Paper size
            let mut i = 0usize;
            let mut child = radio_ref.first_child();
            while let Some(w) = child {
                if w.downcast_ref::<CheckButton>().map(|b| b.is_active()).unwrap_or(false) {
                    st.doc.page.paper_size = sizes[i].1.clone();
                }
                child = w.next_sibling(); i += 1;
            }

            // Orientation
            st.doc.page.orientation = if port_ref.is_active() {
                DocOrientation::Portrait
            } else {
                DocOrientation::Landscape
            };

            // Margins
            st.doc.page.margins.top_mm    = tops.value();
            st.doc.page.margins.bottom_mm = bots.value();
            st.doc.page.margins.left_mm   = lefts.value();
            st.doc.page.margins.right_mm  = rights.value();

            st.invalidate_layout();
            st.dirty = true;
            drop(st);
            c.queue_draw();
            d.close();
        });
    }
    vbox.append(&apply_btn);

    tv.set_content(Some(&vbox));
    dialog.set_child(Some(&tv));
    dialog.present(Some(parent));
}
