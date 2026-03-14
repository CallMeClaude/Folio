//! Cairo paint pass.
//!
//! On each draw call:
//!   1. Ensure the LayoutCache is valid (recompute if None).
//!   2. Paint canvas background + page shadow + white page.
//!   3. Paint selection highlights (behind text).
//!   4. Paint text for each block.
//!   5. Paint the cursor.

use cairo::Context;
use folio_core::BlockKind;
use crate::canvas::EditorState;
use crate::canvas::layout::{
    LayoutCache, CONTENT_X, CONTENT_Y, PAGE_PAD, PAGE_W, PAGE_H,
};
use crate::canvas::selection::paint_selection_for_block;

pub fn draw(cr: &Context, state: &EditorState) {
    // ── Ensure layout cache is valid ──────────────────────────────────────
    {
        let mut cache = state.layout_cache.borrow_mut();
        if cache.is_none() {
            let pctx = pangocairo::functions::create_context(cr);
            *cache = Some(LayoutCache::build(&state.doc, &pctx));
        }
    }

    // ── Canvas background ─────────────────────────────────────────────────
    cr.set_source_rgb(0.925, 0.922, 0.918);
    cr.paint().ok();

    // ── Page drop-shadow ──────────────────────────────────────────────────
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.10);
    cr.rectangle(PAGE_PAD + 3.0, PAGE_PAD + 3.0, PAGE_W, PAGE_H);
    cr.fill().ok();

    // ── White page ────────────────────────────────────────────────────────
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.rectangle(PAGE_PAD, PAGE_PAD, PAGE_W, PAGE_H);
    cr.fill().ok();

    let cache_ref = state.layout_cache.borrow();
    let cache = match cache_ref.as_ref() {
        Some(c) => c,
        None    => return,
    };

    // ── Selection highlights (drawn before text) ──────────────────────────
    if let Some(sel_state) = &state.selection {
        if !sel_state.is_collapsed() {
            let range = sel_state.to_range();
            for (i, cb) in cache.blocks.iter().enumerate() {
                paint_selection_for_block(cr, i, &range, cb);
            }
        }
    }

    // ── Text ──────────────────────────────────────────────────────────────
    cr.set_source_rgb(0.11, 0.11, 0.11);
    for (i, cb) in cache.blocks.iter().enumerate() {
        cr.move_to(CONTENT_X, cb.y_top);
        pangocairo::functions::show_layout(cr, &cb.layout);

        // Cursor
        if i == state.cursor.block_idx && state.cursor_visible
            && state.selection.as_ref().map(|s| s.is_collapsed()).unwrap_or(true)
        {
            paint_cursor(cr, &cb.layout, state.cursor.byte_offset, cb.y_top);
        }
    }
}

fn paint_cursor(cr: &Context, layout: &pango::Layout, byte_off: usize, block_y: f64) {
    let (strong, _) = layout.cursor_pos(byte_off as i32);
    let ps = pango::SCALE as f64;
    let x  = CONTENT_X + strong.x()      as f64 / ps;
    let y  = block_y   + strong.y()      as f64 / ps;
    let h  =             strong.height() as f64 / ps;

    cr.set_source_rgb(0.106, 0.431, 0.929);
    cr.set_line_width(1.5);
    cr.move_to(x, y);
    cr.line_to(x, y + h);
    cr.stroke().ok();
}
