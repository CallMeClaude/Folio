//! Find & Replace dialog — Phase 7.
//!
//! Non-modal AdwDialog that floats over the document window.
//! Mode 1: literal text (case-sensitive toggle).
//! Mode 2: regex (same toggle applies).
//! Navigation: Previous / Next with "N of M" display.
//! Replace: replaces current match; Replace All: replaces all.

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Button, CheckButton, DrawingArea,
           Entry, Label, Orientation, Align, Window};
use libadwaita::prelude::*;
use libadwaita::{HeaderBar, ToolbarView};
use folio_core::{SearchQuery, find_next, find_prev, find_all,
                 replace_match, replace_all, DocPosition};
use crate::canvas::EditorState;

/// Show (or re-present) the Find & Replace dialog.
pub fn show(
    parent:   &Window,
    state:    Rc<RefCell<EditorState>>,
    canvas:   DrawingArea,
) {
    let dialog = libadwaita::Dialog::builder()
        .title("Find & Replace")
        .content_width(480)
        .build();

    let tv = ToolbarView::new();
    tv.add_top_bar(&HeaderBar::new());

    let vbox = GBox::new(Orientation::Vertical, 10);
    vbox.set_margin_top(14);
    vbox.set_margin_bottom(14);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    // ── Mode toggle (Literal / Regex) ─────────────────────────────────────
    let mode_box = GBox::new(Orientation::Horizontal, 8);
    mode_box.add_css_class("linked");
    let lit_btn = gtk4::ToggleButton::with_label("Literal");
    let rex_btn = gtk4::ToggleButton::builder().label("Regex").group(&lit_btn).build();
    lit_btn.set_active(true);
    mode_box.append(&lit_btn);
    mode_box.append(&rex_btn);

    let case_btn = CheckButton::with_label("Match case");
    let top_row = GBox::new(Orientation::Horizontal, 8);
    top_row.append(&mode_box);
    top_row.append(&case_btn);
    vbox.append(&top_row);

    // ── Find row ──────────────────────────────────────────────────────────
    let find_entry = Entry::builder()
        .placeholder_text("Find…")
        .hexpand(true)
        .build();
    find_entry.add_css_class("search");

    let status_lbl = Label::new(Some(""));
    status_lbl.add_css_class("dim-label");
    status_lbl.add_css_class("caption");
    status_lbl.set_halign(Align::End);
    status_lbl.set_width_chars(8);

    let prev_btn = Button::from_icon_name("go-up-symbolic");
    let next_btn = Button::from_icon_name("go-down-symbolic");
    prev_btn.set_tooltip_text(Some("Previous match"));
    next_btn.set_tooltip_text(Some("Next match"));
    prev_btn.add_css_class("flat");
    next_btn.add_css_class("flat");

    let find_row = GBox::new(Orientation::Horizontal, 4);
    find_row.append(&find_entry);
    find_row.append(&status_lbl);
    find_row.append(&prev_btn);
    find_row.append(&next_btn);
    vbox.append(&find_row);

    // ── Replace row ───────────────────────────────────────────────────────
    let repl_entry = Entry::builder()
        .placeholder_text("Replace with…")
        .hexpand(true)
        .build();

    let repl_btn     = Button::with_label("Replace");
    let repl_all_btn = Button::with_label("Replace All");
    repl_btn.add_css_class("flat");
    repl_all_btn.add_css_class("flat");

    let repl_row = GBox::new(Orientation::Horizontal, 4);
    repl_row.append(&repl_entry);
    repl_row.append(&repl_btn);
    repl_row.append(&repl_all_btn);
    vbox.append(&repl_row);

    tv.set_content(Some(&vbox));
    dialog.set_child(Some(&tv));

    // ── Helper: build query from current UI state ─────────────────────────
    let build_query = {
        let fe = find_entry.clone();
        let rb = rex_btn.clone();
        let cb = case_btn.clone();
        move || SearchQuery {
            pattern:        fe.text().to_string(),
            use_regex:      rb.is_active(),
            case_sensitive: cb.is_active(),
        }
    };

    // ── Helper: update the "N of M" status label ──────────────────────────
    let update_status = {
        let s  = state.clone();
        let sl = status_lbl.clone();
        let fe = find_entry.clone();
        let rb = rex_btn.clone();
        let cb = case_btn.clone();
        move || {
            let q = SearchQuery {
                pattern:        fe.text().to_string(),
                use_regex:      rb.is_active(),
                case_sensitive: cb.is_active(),
            };
            if q.pattern.is_empty() { sl.set_text(""); return; }
            match find_all(&s.borrow().doc, &q) {
                Ok(all) => sl.set_text(&format!("{} found", all.len())),
                Err(_)  => sl.set_text("bad regex"),
            }
        }
    };

    // Update status whenever the find entry text changes.
    {
        let us = update_status.clone();
        find_entry.connect_changed(move |_| us());
    }

    // ── Next button ───────────────────────────────────────────────────────
    {
        let s  = state.clone();
        let c  = canvas.clone();
        let bq = build_query.clone();
        next_btn.connect_clicked(move |_| {
            let q = bq();
            if q.pattern.is_empty() { return; }
            let cursor = s.borrow().cursor;
            match find_next(&s.borrow().doc, &q, cursor) {
                Ok(Some((m, _))) => {
                    let mut st = s.borrow_mut();
                    st.cursor    = m.end_pos();
                    st.selection = Some(crate::canvas::selection::SelectionState {
                        anchor: m.start_pos(),
                        active: m.end_pos(),
                    });
                    st.cursor_visible = true;
                    st.invalidate_layout();
                    drop(st);
                    c.queue_draw();
                }
                _ => {}
            }
        });
    }

    // ── Prev button ───────────────────────────────────────────────────────
    {
        let s  = state.clone();
        let c  = canvas.clone();
        let bq = build_query.clone();
        prev_btn.connect_clicked(move |_| {
            let q = bq();
            if q.pattern.is_empty() { return; }
            let cursor = s.borrow().cursor;
            match find_prev(&s.borrow().doc, &q, cursor) {
                Ok(Some((m, _))) => {
                    let mut st = s.borrow_mut();
                    st.cursor    = m.end_pos();
                    st.selection = Some(crate::canvas::selection::SelectionState {
                        anchor: m.start_pos(),
                        active: m.end_pos(),
                    });
                    st.cursor_visible = true;
                    st.invalidate_layout();
                    drop(st);
                    c.queue_draw();
                }
                _ => {}
            }
        });
    }

    // ── Enter in find entry → Next ────────────────────────────────────────
    {
        let nb = next_btn.clone();
        find_entry.connect_activate(move |_| nb.emit_clicked());
    }

    // ── Replace button ────────────────────────────────────────────────────
    {
        let s   = state.clone();
        let c   = canvas.clone();
        let bq  = build_query.clone();
        let re  = repl_entry.clone();
        let us  = update_status.clone();
        repl_btn.connect_clicked(move |_| {
            let q           = bq();
            let replacement = re.text().to_string();
            if q.pattern.is_empty() { return; }

            // Only replace if there's a current selection that matches.
            let maybe_m = {
                let st = s.borrow();
                let cursor = st.cursor;
                // Re-run find_next to confirm the current match.
                find_next(&st.doc, &q, DocPosition::new(
                    if cursor.block_idx > 0 { cursor.block_idx - 1 } else { 0 }, 0
                )).ok().flatten()
            };
            if let Some((m, _)) = maybe_m {
                let snap = s.borrow().doc.clone();
                s.borrow_mut().engine.checkpoint(&snap).ok();
                replace_match(&mut s.borrow_mut().doc, &m, &replacement).ok();
                {
                    let mut st = s.borrow_mut();
                    st.cursor    = m.start_pos();
                    st.selection = None;
                    st.invalidate_layout();
                    st.dirty = true;
                }
                us();
                c.queue_draw();
            }
        });
    }

    // ── Replace All button ────────────────────────────────────────────────
    {
        let s   = state.clone();
        let c   = canvas.clone();
        let bq  = build_query.clone();
        let re  = repl_entry.clone();
        let us  = update_status.clone();
        let sl  = status_lbl.clone();
        repl_all_btn.connect_clicked(move |_| {
            let q           = bq();
            let replacement = re.text().to_string();
            if q.pattern.is_empty() { return; }

            let snap = s.borrow().doc.clone();
            s.borrow_mut().engine.checkpoint(&snap).ok();

            match replace_all(&mut s.borrow_mut().doc, &q, &replacement) {
                Ok(n) => {
                    sl.set_text(&format!("Replaced {}", n));
                    let mut st = s.borrow_mut();
                    st.invalidate_layout();
                    st.dirty = true;
                    drop(st);
                    c.queue_draw();
                }
                Err(e) => sl.set_text(&format!("Error: {}", e)),
            }
        });
    }

    dialog.present(Some(parent));

    // Focus the find entry immediately.
    find_entry.grab_focus();
}
