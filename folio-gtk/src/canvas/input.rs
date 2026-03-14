use gtk4::prelude::*;
use gtk4::{DrawingArea, gdk};
use folio_core::{DocPosition, DocRange};
use crate::canvas::EditorState;

/// Called from the EventControllerKey handler.
/// Mutates `state` in place and requests a redraw on change.
pub fn handle_key(
    key:   gdk::Key,
    mods:  gdk::ModifierType,
    s:     &mut EditorState,
    da:    &DrawingArea,
) {
    let ctrl  = mods.contains(gdk::ModifierType::CONTROL_MASK);
    let shift = mods.contains(gdk::ModifierType::SHIFT_MASK);

    let changed = match key {
        // ── Undo / Redo ───────────────────────────────────────────────────
        gdk::Key::z | gdk::Key::Z if ctrl && !shift => undo(s),
        gdk::Key::z | gdk::Key::Z if ctrl && shift  => redo(s),
        gdk::Key::y | gdk::Key::Y if ctrl           => redo(s),

        // ── Save ──────────────────────────────────────────────────────────
        gdk::Key::s | gdk::Key::S if ctrl => { save(s); false }

        // ── Editing ───────────────────────────────────────────────────────
        gdk::Key::BackSpace => backspace(s),
        gdk::Key::Return | gdk::Key::KP_Enter => enter(s),

        // ── Navigation ────────────────────────────────────────────────────
        gdk::Key::Left  => move_cursor(s, -1),
        gdk::Key::Right => move_cursor(s,  1),
        gdk::Key::Up    => move_cursor_vertical(s, -1),
        gdk::Key::Down  => move_cursor_vertical(s,  1),
        gdk::Key::Home  => { s.cursor.byte_offset = 0; true }
        gdk::Key::End   => {
            s.cursor.byte_offset =
                s.doc.blocks[s.cursor.block_idx].plain_text().len();
            true
        }

        // ── Printable characters ──────────────────────────────────────────
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

// ── Undo / Redo ───────────────────────────────────────────────────────────────

fn undo(s: &mut EditorState) -> bool {
    match s.engine.undo() {
        Ok(Some(doc)) => { s.doc = doc; true }
        _             => false,
    }
}

fn redo(s: &mut EditorState) -> bool {
    match s.engine.redo() {
        Ok(Some(doc)) => { s.doc = doc; true }
        _             => false,
    }
}

// ── Save ──────────────────────────────────────────────────────────────────────

fn save(s: &mut EditorState) {
    if let Some(ref path) = s.save_path {
        let path = path.clone();
        if let Err(e) = folio_core::format::save_folio(&path, &s.doc, &s.engine, &[]) {
            eprintln!("Save failed: {e}");
        } else {
            s.dirty = false;
        }
    }
    // If save_path is None the document has never been saved; a Save-As dialog
    // will be wired in a later phase. For now we silently skip.
}

// ── Editing ───────────────────────────────────────────────────────────────────

fn type_char(s: &mut EditorState, ch: char) {
    let mut buf = [0u8; 4];
    let text = ch.encode_utf8(&mut buf).to_owned();
    s.engine.checkpoint(&s.doc).ok();
    let pos = s.cursor;
    s.doc.insert_text(pos, &text).ok();
    s.cursor.byte_offset += ch.len_utf8();
    s.dirty = true;
}

fn backspace(s: &mut EditorState) -> bool {
    let pos = s.cursor;
    if pos.byte_offset == 0 {
        if pos.block_idx == 0 { return false; }
        let prev_len = s.doc.blocks[pos.block_idx - 1].plain_text().len();
        s.engine.checkpoint(&s.doc).ok();
        s.doc.merge_blocks(pos.block_idx - 1).ok();
        s.cursor = DocPosition::new(pos.block_idx - 1, prev_len);
        s.dirty = true;
        return true;
    }
    let text = s.doc.blocks[pos.block_idx].plain_text();
    let prev = prev_boundary(&text, pos.byte_offset);
    s.engine.checkpoint(&s.doc).ok();
    s.doc.delete_range(
        DocRange::new(DocPosition::new(pos.block_idx, prev), pos)
    ).ok();
    s.cursor.byte_offset = prev;
    s.dirty = true;
    true
}

fn enter(s: &mut EditorState) -> bool {
    s.engine.checkpoint(&s.doc).ok();
    let pos = s.cursor;
    s.doc.split_block(pos).ok();
    s.cursor = DocPosition::block_start(pos.block_idx + 1);
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
            s.cursor.block_idx -= 1;
            s.cursor.byte_offset =
                s.doc.blocks[s.cursor.block_idx].plain_text().len();
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

// ── UTF-8 boundary helpers ────────────────────────────────────────────────────

fn prev_boundary(s: &str, from: usize) -> usize {
    (1..=from).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
}

fn next_boundary(s: &str, from: usize) -> usize {
    (from + 1..=s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len())
}
