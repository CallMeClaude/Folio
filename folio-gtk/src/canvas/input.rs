use gtk4::prelude::*;
use gtk4::{DrawingArea, gdk};
use folio_core::{DocPosition, DocRange};
use crate::canvas::EditorState;
use crate::canvas::selection::SelectionState;

pub fn handle_key(
    key:   gdk::Key,
    mods:  gdk::ModifierType,
    s:     &mut EditorState,
    da:    &DrawingArea,
) {
    let ctrl  = mods.contains(gdk::ModifierType::CONTROL_MASK);
    let shift = mods.contains(gdk::ModifierType::SHIFT_MASK);

    let changed = match key {
        gdk::Key::z | gdk::Key::Z if ctrl && !shift => undo(s),
        gdk::Key::z | gdk::Key::Z if ctrl &&  shift => redo(s),
        gdk::Key::y | gdk::Key::Y if ctrl           => redo(s),
        gdk::Key::s | gdk::Key::S if ctrl           => { save(s); false }
        gdk::Key::BackSpace                          => backspace(s),
        gdk::Key::Return | gdk::Key::KP_Enter        => enter(s),
        gdk::Key::Left  if shift => extend_selection(s, -1, false),
        gdk::Key::Right if shift => extend_selection(s,  1, false),
        gdk::Key::Up    if shift => extend_selection(s, -1, true),
        gdk::Key::Down  if shift => extend_selection(s,  1, true),
        gdk::Key::Left  => { s.selection = None; move_cursor(s, -1) }
        gdk::Key::Right => { s.selection = None; move_cursor(s,  1) }
        gdk::Key::Up    => { s.selection = None; move_cursor_vertical(s, -1) }
        gdk::Key::Down  => { s.selection = None; move_cursor_vertical(s,  1) }
        gdk::Key::Home  => {
            s.selection = None;
            s.cursor.byte_offset = 0;
            true
        }
        gdk::Key::End => {
            s.selection = None;
            s.cursor.byte_offset = s.doc.blocks[s.cursor.block_idx].plain_text().len();
            true
        }
        k => match k.to_unicode() {
            Some(ch) if !ch.is_control() => { type_char(s, ch); true }
            _ => false,
        },
    };

    if changed {
        s.cursor_visible = true;
        da.queue_draw();
    }
}

// ── checkpoint helper ─────────────────────────────────────────────────────────
// Clones the doc before passing to engine so we never hold two borrows of `s`.
fn checkpoint(s: &mut EditorState) {
    let snap = s.doc.clone();
    s.engine.checkpoint(&snap).ok();
}

// ── Undo / Redo ───────────────────────────────────────────────────────────────

fn undo(s: &mut EditorState) -> bool {
    match s.engine.undo() {
        Ok(Some(doc)) => { s.doc = doc; s.selection = None; s.invalidate_layout(); true }
        _             => false,
    }
}

fn redo(s: &mut EditorState) -> bool {
    match s.engine.redo() {
        Ok(Some(doc)) => { s.doc = doc; s.selection = None; s.invalidate_layout(); true }
        _             => false,
    }
}

// ── Save ──────────────────────────────────────────────────────────────────────

fn save(s: &mut EditorState) {
    if let Some(path) = s.save_path.clone() {
        match folio_core::format::save_folio(&path, &s.doc, &s.engine, &[]) {
            Ok(_)  => s.dirty = false,
            Err(e) => eprintln!("Save failed: {e}"),
        }
    }
}

// ── Editing ───────────────────────────────────────────────────────────────────

fn type_char(s: &mut EditorState, ch: char) {
    let mut buf = [0u8; 4];
    let text = ch.encode_utf8(&mut buf).to_owned();
    checkpoint(s);
    let pos = s.cursor;
    s.doc.insert_text(pos, &text).ok();
    s.cursor.byte_offset += ch.len_utf8();
    s.selection = None;
    s.invalidate_layout();
    s.dirty = true;
}

fn backspace(s: &mut EditorState) -> bool {
    // Delete selection if one exists.
    if let Some(sel) = s.selection.clone() {
        if !sel.is_collapsed() {
            let range = sel.to_range();
            checkpoint(s);
            s.doc.delete_range(range).ok();
            s.cursor    = range.start;
            s.selection = None;
            s.invalidate_layout();
            s.dirty = true;
            return true;
        }
    }
    let pos = s.cursor;
    if pos.byte_offset == 0 {
        if pos.block_idx == 0 { return false; }
        let prev_len = s.doc.blocks[pos.block_idx - 1].plain_text().len();
        checkpoint(s);
        s.doc.merge_blocks(pos.block_idx - 1).ok();
        s.cursor = DocPosition::new(pos.block_idx - 1, prev_len);
        s.invalidate_layout();
        s.dirty = true;
        return true;
    }
    let text = s.doc.blocks[pos.block_idx].plain_text();
    let prev = prev_boundary(&text, pos.byte_offset);
    checkpoint(s);
    s.doc.delete_range(DocRange::new(DocPosition::new(pos.block_idx, prev), pos)).ok();
    s.cursor.byte_offset = prev;
    s.invalidate_layout();
    s.dirty = true;
    true
}

fn enter(s: &mut EditorState) -> bool {
    checkpoint(s);
    let pos = s.cursor;
    s.doc.split_block(pos).ok();
    s.cursor    = DocPosition::block_start(pos.block_idx + 1);
    s.selection = None;
    s.invalidate_layout();
    s.dirty = true;
    true
}

// ── Cursor movement ───────────────────────────────────────────────────────────

fn move_cursor(s: &mut EditorState, dir: i32) -> bool {
    let text = s.doc.blocks[s.cursor.block_idx].plain_text();
    let off  = s.cursor.byte_offset;
    if dir < 0 {
        if off == 0 {
            if s.cursor.block_idx == 0 { return false; }
            s.cursor.block_idx  -= 1;
            s.cursor.byte_offset = s.doc.blocks[s.cursor.block_idx].plain_text().len();
        } else {
            s.cursor.byte_offset = prev_boundary(&text, off);
        }
    } else {
        if off >= text.len() {
            if s.cursor.block_idx + 1 >= s.doc.blocks.len() { return false; }
            s.cursor.block_idx  += 1;
            s.cursor.byte_offset = 0;
        } else {
            s.cursor.byte_offset = next_boundary(&text, off);
        }
    }
    true
}

fn move_cursor_vertical(s: &mut EditorState, dir: i32) -> bool {
    let idx = s.cursor.block_idx;
    if dir < 0 {
        if idx == 0 { return false; }
        s.cursor.block_idx   = idx - 1;
        s.cursor.byte_offset = s.cursor.byte_offset
            .min(s.doc.blocks[idx - 1].plain_text().len());
    } else {
        if idx + 1 >= s.doc.blocks.len() { return false; }
        s.cursor.block_idx   = idx + 1;
        s.cursor.byte_offset = s.cursor.byte_offset
            .min(s.doc.blocks[idx + 1].plain_text().len());
    }
    true
}

fn extend_selection(s: &mut EditorState, dir: i32, vertical: bool) -> bool {
    let anchor = s.selection.as_ref().map(|sel| sel.anchor).unwrap_or(s.cursor);
    let moved  = if vertical { move_cursor_vertical(s, dir) } else { move_cursor(s, dir) };
    if moved {
        s.selection = Some(SelectionState { anchor, active: s.cursor });
    }
    moved
}

// ── UTF-8 helpers ─────────────────────────────────────────────────────────────

fn prev_boundary(s: &str, from: usize) -> usize {
    (1..=from).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
}

fn next_boundary(s: &str, from: usize) -> usize {
    (from + 1..=s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len())
}
