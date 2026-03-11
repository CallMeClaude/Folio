use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Label, ListBox, ListBoxRow, Align, ScrolledWindow};
use folio_core::BlockKind;
use crate::canvas::EditorState;

pub struct OutlinePanel {
    pub widget: ScrolledWindow,
    list:       ListBox,
    state:      Rc<RefCell<EditorState>>,
}

impl OutlinePanel {
    pub fn new(state: Rc<RefCell<EditorState>>) -> Self {
        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);

        let list = ListBox::new();
        list.add_css_class("navigation-sidebar");
        list.set_selection_mode(gtk4::SelectionMode::Single);

        scroll.set_child(Some(&list));

        let panel = OutlinePanel { widget: scroll, list, state };
        panel.refresh();
        panel
    }

    /// Re-populate the outline list from the current document.
    pub fn refresh(&self) {
        // Remove all existing rows.
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        let st = self.state.borrow();
        for (_i, block) in st.doc.blocks.iter().enumerate() {
            let (prefix, indent) = match &block.kind {
                BlockKind::Title    => ("◆ ", 0),
                BlockKind::Heading1 => ("▸ ", 0),
                BlockKind::Heading2 => ("  ▸ ", 8),
                _ => continue,
            };
            let text = block.plain_text();
            if text.trim().is_empty() { continue; }

            let row = ListBoxRow::new();
            let lbl = Label::new(Some(&format!("{}{}", prefix, text)));
            lbl.set_halign(Align::Start);
            lbl.set_margin_start(indent + 8);
            lbl.set_margin_top(4);
            lbl.set_margin_bottom(4);
            row.set_child(Some(&lbl));
            self.list.append(&row);
        }
    }
}
