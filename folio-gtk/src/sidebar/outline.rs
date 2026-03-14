use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Label, ListBox, ListBoxRow, Align,
           Orientation, ScrolledWindow, PolicyType};
use glib;
use folio_core::{BlockKind, DocPosition};
use crate::canvas::EditorState;

pub struct OutlinePanel {
    pub widget: ScrolledWindow,
}

impl OutlinePanel {
    pub fn new(
        state:  Rc<RefCell<EditorState>>,
        canvas: gtk4::DrawingArea,
    ) -> Self {
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list = ListBox::new();
        list.add_css_class("navigation-sidebar");
        list.set_selection_mode(gtk4::SelectionMode::Single);
        list.set_activate_on_single_click(true);
        scroll.set_child(Some(&list));

        // Initial population.
        Self::repopulate(&list, &state, &canvas);

        // Re-populate every second only when heading content changes.
        {
            let s  = state.clone();
            let l  = list.clone();
            let c  = canvas.clone();
            let last_sig: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
            glib::timeout_add_local(Duration::from_millis(1000), move || {
                let sig = Self::outline_signature(&s.borrow());
                if sig != *last_sig.borrow() {
                    *last_sig.borrow_mut() = sig;
                    Self::repopulate(&l, &s, &c);
                }
                glib::ControlFlow::Continue
            });
        }

        OutlinePanel { widget: scroll }
    }

    fn outline_signature(st: &EditorState) -> String {
        st.doc.blocks.iter()
            .filter(|b| matches!(b.kind,
                BlockKind::Title | BlockKind::Heading1 | BlockKind::Heading2))
            .map(|b| b.plain_text())
            .collect::<Vec<_>>()
            .join("|")
    }

    fn repopulate(
        list:   &ListBox,
        state:  &Rc<RefCell<EditorState>>,
        canvas: &gtk4::DrawingArea,
    ) {
        // Disconnect previous row-activated signal before rebuilding.
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let st      = state.borrow();
        let mut any = false;

        // Collect owned data before dropping the borrow.
        let rows: Vec<(usize, BlockKind, String)> = st.doc.blocks.iter()
            .enumerate()
            .filter_map(|(idx, block)| {
                if matches!(block.kind, BlockKind::Title | BlockKind::Heading1 | BlockKind::Heading2) {
                    let text = block.plain_text();
                    if !text.trim().is_empty() {
                        return Some((idx, block.kind.clone(), text));
                    }
                }
                None
            })
            .collect();
        drop(st);

        for (block_idx, kind, text) in rows {
            any = true;
            let (prefix, indent): (&str, i32) = match kind {
                BlockKind::Title    => ("◆ ", 0),
                BlockKind::Heading1 => ("▸ ", 0),
                BlockKind::Heading2 => ("  ▸ ", 8),
                _ => unreachable!(),
            };

            let row     = ListBoxRow::new();
            let row_box = GBox::new(Orientation::Horizontal, 4);
            row_box.set_margin_start(indent + 8);
            row_box.set_margin_top(4);
            row_box.set_margin_bottom(4);
            row_box.set_margin_end(8);

            let lbl = Label::new(Some(&format!("{}{}", prefix, text)));
            lbl.set_halign(Align::Start);
            lbl.set_ellipsize(pango::EllipsizeMode::End);
            lbl.set_max_width_chars(28);
            row_box.append(&lbl);
            row.set_child(Some(&row_box));

            // Wire click → move cursor to this block, grab focus.
            let s = state.clone();
            let c = canvas.clone();
            row.connect_activate(move |_| {
                let mut st = s.borrow_mut();
                st.cursor = DocPosition::block_start(block_idx);
                st.selection = None;
                st.cursor_visible = true;
                drop(st);
                c.grab_focus();
                c.queue_draw();
            });

            list.append(&row);
        }

        if !any {
            let empty = ListBoxRow::new();
            let lbl   = Label::new(Some("No headings yet"));
            lbl.add_css_class("dim-label");
            lbl.set_margin_top(16);
            lbl.set_halign(Align::Center);
            empty.set_child(Some(&lbl));
            empty.set_activatable(false);
            empty.set_selectable(false);
            list.append(&empty);
        }
    }
}
