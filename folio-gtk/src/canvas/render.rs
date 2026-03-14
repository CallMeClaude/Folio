//! Cairo paint pass.
//!
//! On each draw call:
//!   1. Ensure the LayoutCache is valid (recompute if None).
//!   2. Paint canvas background + page shadow + white page.
//!   3. Paint selection highlights (behind text).
//!   4. Paint text for each block.
//!   5. Paint the cursor.

use cairo::Context;
use crate::canvas::EditorState;
use crate::canvas::layout::{
    LayoutCache, CONTENT_X, CONTENT_W, PAGE_PAD, PAGE_W, PAGE_H,
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
    for (i, cb) in cache.blocks.iter().enumerate() {
        let block = &state.doc.blocks[i];

        // Block-kind decorations drawn before text.
        match &block.kind {
            folio_core::BlockKind::Quote => {
                // Left accent bar in Adwaita blue.
                cr.set_source_rgba(0.212, 0.522, 0.894, 0.7);
                let (_, h) = cb.layout.pixel_size();
                cr.rectangle(CONTENT_X - 12.0, cb.y_top, 3.0, h as f64);
                cr.fill().ok();
                cr.set_source_rgba(0.212, 0.522, 0.894, 0.07);
                cr.rectangle(CONTENT_X - 12.0, cb.y_top, CONTENT_W + 12.0, h as f64);
                cr.fill().ok();
            }
            folio_core::BlockKind::Code => {
                // Subtle grey background with rounded corners.
                let (_, h) = cb.layout.pixel_size();
                cr.set_source_rgb(0.94, 0.94, 0.92);
                rounded_rect(cr, CONTENT_X - 8.0, cb.y_top - 4.0, CONTENT_W + 16.0, h as f64 + 8.0, 4.0);
                cr.fill().ok();
            }
            _ => {}
        }

        // Choose text colour.
        match &block.kind {
            folio_core::BlockKind::Caption => cr.set_source_rgb(0.4, 0.4, 0.4),
            folio_core::BlockKind::Quote   => cr.set_source_rgba(0.15, 0.15, 0.15, 0.85),
            _                              => cr.set_source_rgb(0.11, 0.11, 0.11),
        }

        cr.move_to(CONTENT_X, cb.y_top);
        pangocairo::functions::show_layout(cr, &cb.layout);

        // Cursor — hide when there is an active (non-collapsed) selection.
        let cursor_visible = state.cursor_visible
            && i == state.cursor.block_idx
            && state.selection.as_ref().map(|s| s.is_collapsed()).unwrap_or(true);
        if cursor_visible {
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

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.new_sub_path();
    cr.arc(x + w - r, y + r,     r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0,                           std::f64::consts::FRAC_PI_2);
    cr.arc(x + r,     y + h - r, r, std::f64::consts::FRAC_PI_2,   std::f64::consts::PI);
    cr.arc(x + r,     y + r,     r, std::f64::consts::PI,          3.0 * std::f64::consts::FRAC_PI_2);
    cr.close_path();
}
