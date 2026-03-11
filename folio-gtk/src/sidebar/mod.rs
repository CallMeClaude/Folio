pub mod format;
pub mod outline;
pub mod stats;

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Orientation, Stack, StackSwitcher, StackTransitionType};
use crate::canvas::EditorState;
use format::FormatPanel;
use outline::OutlinePanel;
use stats::StatsPanel;

pub struct DocumentSidebar {
    pub widget: GBox,
}

impl DocumentSidebar {
    pub fn new(state: Rc<RefCell<EditorState>>, canvas: gtk4::DrawingArea) -> Self {
        let vbox = GBox::new(Orientation::Vertical, 0);
        vbox.set_width_request(240);

        // Tab switcher at the top of the sidebar.
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::SlideLeftRight);
        stack.set_vexpand(true);

        let switcher = StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_margin_top(6);
        switcher.set_margin_bottom(6);
        switcher.set_margin_start(8);
        switcher.set_margin_end(8);

        // ── Format panel ───────────────────────────────────────────────────
        let fmt = FormatPanel::new(state.clone(), canvas.clone());
        stack.add_titled(&fmt.widget, Some("format"), "Format");

        // ── Outline panel ──────────────────────────────────────────────────
        let outline = OutlinePanel::new(state.clone());
        stack.add_titled(&outline.widget, Some("outline"), "Outline");

        // ── Stats panel ────────────────────────────────────────────────────
        let stats = StatsPanel::new(state.clone());
        stack.add_titled(&stats.widget, Some("stats"), "Stats");

        // Separator between switcher and content.
        let sep = gtk4::Separator::new(Orientation::Horizontal);

        vbox.append(&switcher);
        vbox.append(&sep);
        vbox.append(&stack);

        DocumentSidebar { widget: vbox }
    }
}
