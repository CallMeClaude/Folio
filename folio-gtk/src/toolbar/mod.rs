use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, DrawingArea, DropDown, Orientation, Separator, ToggleButton};
use folio_core::{BlockKind, InlineAttr, Alignment};
use crate::canvas::EditorState;

pub struct FormattingToolbar {
    pub widget: GBox,
}

const STYLE_NAMES: &[&str] = &[
    "Paragraph", "Title", "Heading 1", "Heading 2", "Caption", "Quote", "Code",
];

fn idx_to_kind(i: u32) -> BlockKind {
    match i {
        1 => BlockKind::Title,
        2 => BlockKind::Heading1,
        3 => BlockKind::Heading2,
        4 => BlockKind::Caption,
        5 => BlockKind::Quote,
        6 => BlockKind::Code,
        _ => BlockKind::Paragraph,
    }
}

impl FormattingToolbar {
    pub fn new(state: Rc<RefCell<EditorState>>, canvas: DrawingArea) -> Self {
        let bar = GBox::new(Orientation::Horizontal, 4);
        bar.add_css_class("toolbar");
        bar.set_margin_start(8);
        bar.set_margin_end(8);
        bar.set_margin_top(2);
        bar.set_margin_bottom(2);

        // ── Block style dropdown ───────────────────────────────────────────
        let style_dd = DropDown::from_strings(STYLE_NAMES);
        style_dd.set_tooltip_text(Some("Block style"));
        {
            let s = state.clone();
            let c = canvas.clone();
            style_dd.connect_selected_notify(move |dd| {
                let mut st = s.borrow_mut();
                let idx  = st.cursor.block_idx;
                let snap = st.doc.clone();
                st.engine.checkpoint(&snap).ok();
                st.doc.set_block_kind(idx, idx_to_kind(dd.selected())).ok();
                st.invalidate_layout();
                c.queue_draw();
            });
        }
        bar.append(&style_dd);
        bar.append(&Separator::new(Orientation::Vertical));

        // ── Inline formatting buttons ──────────────────────────────────────
        // If text is selected: apply to selection.
        // If nothing is selected: toggle the pending_attrs typing mode.
        let inline_defs: &[(&str, &str, InlineAttr)] = &[
            ("format-text-bold-symbolic",          "Bold",          InlineAttr::Bold),
            ("format-text-italic-symbolic",        "Italic",        InlineAttr::Italic),
            ("format-text-underline-symbolic",     "Underline",     InlineAttr::Underline),
            ("format-text-strikethrough-symbolic", "Strikethrough", InlineAttr::Strikethrough),
        ];

        for (icon, tip, attr) in inline_defs {
            let btn = ToggleButton::new();
            btn.set_icon_name(icon);
            btn.set_tooltip_text(Some(tip));
            btn.add_css_class("flat");

            let s = state.clone();
            let c = canvas.clone();
            let a = attr.clone();
            btn.connect_toggled(move |b| {
                let mut st = s.borrow_mut();
                let has_sel = st.selection.as_ref()
                    .map(|sel| !sel.is_collapsed() && sel.to_range().is_single_block())
                    .unwrap_or(false);

                if has_sel {
                    // Apply/remove on the selection.
                    let range = st.selection.as_ref().unwrap().to_range();
                    let snap  = st.doc.clone();
                    st.engine.checkpoint(&snap).ok();
                    if b.is_active() {
                        st.doc.apply_inline_attr(range, a.clone()).ok();
                    } else {
                        st.doc.remove_inline_attr(range, &a).ok();
                    }
                    st.invalidate_layout();
                    c.queue_draw();
                } else {
                    // No selection — toggle the pending typing attribute.
                    st.toggle_pending_attr(a.clone());
                    // Grab canvas focus so the user can start typing immediately.
                    c.grab_focus();
                }
            });
            bar.append(&btn);
        }

        bar.append(&Separator::new(Orientation::Vertical));

        // ── Alignment buttons ─────────────────────────────────────────────
        let align_box = GBox::new(Orientation::Horizontal, 0);
        align_box.add_css_class("linked");

        let align_defs: &[(&str, &str, Alignment)] = &[
            ("format-justify-left-symbolic",   "Align left",      Alignment::Left),
            ("format-justify-center-symbolic", "Center",          Alignment::Center),
            ("format-justify-right-symbolic",  "Align right",     Alignment::Right),
            ("format-justify-fill-symbolic",   "Justify",         Alignment::Justified),
        ];

        let mut first_btn: Option<ToggleButton> = None;
        for (icon, tip, align) in align_defs {
            let btn = match &first_btn {
                None    => ToggleButton::new(),
                Some(f) => ToggleButton::builder().group(f).build(),
            };
            btn.set_icon_name(icon);
            btn.set_tooltip_text(Some(tip));
            btn.add_css_class("flat");
            if first_btn.is_none() {
                btn.set_active(true);
                first_btn = Some(btn.clone());
            }

            let s = state.clone();
            let c = canvas.clone();
            let al = align.clone();
            btn.connect_toggled(move |b| {
                if !b.is_active() { return; }
                let mut st  = s.borrow_mut();
                let idx     = st.cursor.block_idx;
                st.doc.blocks[idx].layout.alignment = al.clone();
                st.invalidate_layout();
                c.queue_draw();
            });
            align_box.append(&btn);
        }
        bar.append(&align_box);

        FormattingToolbar { widget: bar }
    }
}
