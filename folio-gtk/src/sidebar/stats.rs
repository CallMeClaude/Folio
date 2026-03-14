use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use gtk4::prelude::*;
use gtk4::{Box as GBox, Label, LevelBar, Orientation, Align};
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

        // ── Counts ────────────────────────────────────────────────────────
        let lb = Self::listbox(&vbox, 12);
        let make_row = |title: &str| -> ActionRow {
            ActionRow::builder().title(title).subtitle("—").build()
        };
        let words_row  = make_row("Words");
        let chars_row  = make_row("Characters");
        let sents_row  = make_row("Sentences");
        let paras_row  = make_row("Paragraphs");
        let pages_row  = make_row("Pages");
        let read_row   = make_row("Read time");
        let grade_row  = make_row("Readability");
        lb.append(&words_row);
        lb.append(&chars_row);
        lb.append(&sents_row);
        lb.append(&paras_row);
        lb.append(&pages_row);
        lb.append(&read_row);
        lb.append(&grade_row);

        // ── Session timer ─────────────────────────────────────────────────
        let session_lbl = Label::new(Some("Session"));
        session_lbl.add_css_class("heading");
        session_lbl.set_halign(Align::Start);
        session_lbl.set_margin_start(12);
        session_lbl.set_margin_top(12);
        session_lbl.set_margin_bottom(4);
        vbox.append(&session_lbl);

        let timer_lb   = Self::listbox(&vbox, 0);
        let timer_row  = make_row("Session time");
        timer_lb.append(&timer_row);

        // ── Word goal ─────────────────────────────────────────────────────
        let goal_lbl = Label::new(Some("Word Goal"));
        goal_lbl.add_css_class("heading");
        goal_lbl.set_halign(Align::Start);
        goal_lbl.set_margin_start(12);
        goal_lbl.set_margin_top(12);
        goal_lbl.set_margin_bottom(4);
        vbox.append(&goal_lbl);

        let goal_lb    = Self::listbox(&vbox, 8);
        let goal_row   = make_row("Progress");
        let goal_bar   = LevelBar::builder()
            .min_value(0.0).max_value(1.0).value(0.0)
            .valign(Align::Center).hexpand(true).build();
        goal_bar.set_margin_start(8);
        goal_bar.set_margin_end(8);
        goal_bar.set_margin_top(4);
        goal_bar.set_margin_bottom(8);
        goal_lb.append(&goal_row);

        // Put bar directly below the row inside the listbox vbox wrapper.
        let goal_wrapper = GBox::new(Orientation::Vertical, 0);
        goal_wrapper.append(&goal_lb);
        goal_wrapper.append(&goal_bar);
        goal_wrapper.set_margin_start(12);
        goal_wrapper.set_margin_end(12);
        vbox.append(&goal_wrapper);

        // ── Timer (session start) ─────────────────────────────────────────
        let session_start = Instant::now();
        let goal_words    = 500usize; // default goal; future: make configurable

        // Refresh every 2 seconds.
        glib::timeout_add_local(Duration::from_secs(2), move || {
            let st    = state.borrow();
            let stats = stats::compute(&st.doc);

            words_row.set_subtitle(&stats.words.to_string());
            chars_row.set_subtitle(&stats.characters.to_string());
            sents_row.set_subtitle(&stats.sentences.to_string());
            paras_row.set_subtitle(&stats.paragraphs.to_string());
            read_row.set_subtitle(&format!("{} min", stats.read_minutes));
            grade_row.set_subtitle(&stats.readability.to_string());

            // Page count from layout cache.
            let pg = st.layout_cache.borrow()
                .as_ref()
                .map(|c| c.page_count)
                .unwrap_or(1);
            pages_row.set_subtitle(&pg.to_string());

            // Session timer.
            let elapsed = session_start.elapsed().as_secs();
            let h = elapsed / 3600;
            let m = (elapsed % 3600) / 60;
            let s = elapsed % 60;
            if h > 0 {
                timer_row.set_subtitle(&format!("{}h {:02}m {:02}s", h, m, s));
            } else {
                timer_row.set_subtitle(&format!("{:02}m {:02}s", m, s));
            }

            // Word goal bar.
            let progress = (stats.words as f64 / goal_words as f64).min(1.0);
            goal_bar.set_value(progress);
            goal_row.set_subtitle(&format!("{} / {} words ({:.0}%)",
                stats.words, goal_words, progress * 100.0));

            glib::ControlFlow::Continue
        });

        StatsPanel { widget: vbox }
    }

    fn listbox(vbox: &GBox, margin_top: i32) -> gtk4::ListBox {
        let lb = gtk4::ListBox::new();
        lb.add_css_class("boxed-list");
        lb.set_margin_start(12);
        lb.set_margin_end(12);
        lb.set_margin_top(margin_top);
        lb.set_selection_mode(gtk4::SelectionMode::None);
        vbox.append(&lb);
        lb
    }
}
