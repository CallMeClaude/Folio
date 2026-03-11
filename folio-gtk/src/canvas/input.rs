use gtk4::prelude::*;
use gtk4::{DrawingArea, gdk};
use folio_core::{DocPosition, DocRange};
use crate::canvas::EditorState;

/// Called from the EventControllerKey handler.
/// Mutates `state` in place and requests a redraw on change.
pub fn handle_key(
    key:  gdk::Key,
    _mod: gdk::ModifierType,
    s:    &mut EditorState,
    da:   &DrawingArea,
) {
    let changed = match key {
        gdk::Key::BackSpace              => backspace(s),
        gdk::Key::Return | gdk::Key::KP_Enter => enter(s),
        gdk::Key::Left                   => move_cursor(s, -1),
        gdk::Key::Right                  => move_cursor(s,  1),
        gdk::Key::Home                   => { s.cursor.byte_offset = 0; true }
        gdk::Key::End                    => {
            s.cursor.byte_offset =
                s.doc.blocks[s.cursor.block_idx].plain_text().len();
            true
        }
        k => match k.to_unicode() {
            Some(ch) if !ch.is_control() => { type_char(s, ch); true }
            _ => false,
        },
    };
    if changed {
        s.cursor_visible = true; // always show cursor after any keystroke
        da.queue_draw();
    }
}

// ─── individual operations ────────────────────────────────────────────────────

fn type_char(s: &mut EditorState, ch: char) {
    let mut buf = [0u8; 4];
    let text = ch.encode_utf8(&mut buf).to_owned();
    s.engine.checkpoint(&s.doc);
    let pos = s.cursor;
    s.doc.insert_text(pos, &text).ok();
    s.cursor.byte_offset += ch.len_utf8();
}

fn backspace(s: &mut EditorState) -> bool {
    let pos = s.cursor;
    if pos.byte_offset == 0 {
        if pos.block_idx == 0 { return false; }
        let prev_len = s.doc.blocks[pos.block_idx - 1].plain_text().len();
        s.engine.checkpoint(&s.doc);
        s.doc.merge_blocks(pos.block_idx - 1).ok();
        s.cursor = DocPosition::new(pos.block_idx - 1, prev_len);
        return true;
    }
    let text = s.doc.blocks[pos.block_idx].plain_text();
    let prev = prev_boundary(&text, pos.byte_offset);
    s.engine.checkpoint(&s.doc);
    s.doc.delete_range(
        DocRange::new(DocPosition::new(pos.block_idx, prev), pos)
    ).ok();
    s.cursor.byte_offset = prev;
    true
}

fn enter(s: &mut EditorState) -> bool {
    s.engine.checkpoint(&s.doc);
    let pos = s.cursor;
    s.doc.split_block(pos).ok();
    s.cursor = DocPosition::block_start(pos.block_idx + 1);
    true
}

fn move_cursor(s: &mut EditorState, dir: i32) -> bool {
    let text = s.doc.blocks[s.cursor.block_idx].plain_text();
    let off  = s.cursor.byte_offset;
    s.cursor.byte_offset = if dir < 0 {
        if off == 0 { return false; }
        prev_boundary(&text, off)
    } else {
        if off >= text.len() { return false; }
        next_boundary(&text, off)
    };
    true
}

// ─── UTF-8 boundary helpers ───────────────────────────────────────────────────

fn prev_boundary(s: &str, from: usize) -> usize {
    (1..=from).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
}

fn next_boundary(s: &str, from: usize) -> usize {
    (from + 1..=s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len())
}
