use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Orientation};
use libadwaita::prelude::*;
use libadwaita::ActionRow;
use glib;
use folio_core::stats;
use crate::canvas::EditorState;

pub struct StatsPanel {
    pub widget: GBox,
}

impl StatsPanel {
    pub fn new(state: Rc<RefCell<EditorState>>) -> Self {
        let vbox = GBox::new(Orientation::Vertical, 0);

        let listbox = gtk4::ListBox::new();
        listbox.add_css_class("boxed-list");
        listbox.set_margin_start(12);
        listbox.set_margin_end(12);
        listbox.set_margin_top(12);
        listbox.set_selection_mode(gtk4::SelectionMode::None);

        let make_row = |title: &str| -> ActionRow {
            ActionRow::builder().title(title).subtitle("—").build()
        };

        let words_row   = make_row("Words");
        let chars_row   = make_row("Characters");
        let sents_row   = make_row("Sentences");
        let paras_row   = make_row("Paragraphs");
        let read_row    = make_row("Read time");
        let grade_row   = make_row("Readability");

        listbox.append(&words_row);
        listbox.append(&chars_row);
        listbox.append(&sents_row);
        listbox.append(&paras_row);
        listbox.append(&read_row);
        listbox.append(&grade_row);
        vbox.append(&listbox);

        // Refresh stats every 2 seconds.
        glib::timeout_add_local(Duration::from_secs(2), move || {
            let s     = state.borrow();
            let stats = stats::compute(&s.doc);
            words_row.set_subtitle(&stats.words.to_string());
            chars_row.set_subtitle(&stats.characters.to_string());
            sents_row.set_subtitle(&stats.sentences.to_string());
            paras_row.set_subtitle(&stats.paragraphs.to_string());
            read_row.set_subtitle(&format!("{} min", stats.read_minutes));
            grade_row.set_subtitle(&stats.readability.to_string());
            glib::ControlFlow::Continue
        });

        StatsPanel { widget: vbox }
    }
}
