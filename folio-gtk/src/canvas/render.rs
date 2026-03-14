//! Cairo paint pass — Phase 5: real multi-page rendering.
//!
//! Each draw call:
//!  1. Rebuild LayoutCache if stale.
//!  2. Resize the DrawingArea to fit all pages.
//!  3. Paint canvas background.
//!  4. For each page: shadow + white rect.
//!  5. Selection highlights.
//!  6. Block decorations (Quote bar, Code bg) + text + cursor.

use cairo::Context;
use gtk4::prelude::*;
use gtk4::DrawingArea;
use crate::canvas::EditorState;
use crate::canvas::layout::LayoutCache;
use crate::canvas::selection::paint_selection_for_block;

pub fn draw(widget: &DrawingArea, cr: &Context, state: &EditorState) {
    // ── Rebuild cache if stale ────────────────────────────────────────────
    {
        let mut cache = state.layout_cache.borrow_mut();
        if cache.is_none() {
            let pctx = pangocairo::functions::create_context(cr);
            *cache = Some(LayoutCache::build(&state.doc, &pctx));
        }
    }

    let cache_ref = state.layout_cache.borrow();
    let cache = match cache_ref.as_ref() { Some(c) => c, None => return };
    let geo   = &cache.geo;

    // ── Resize DrawingArea to fit all pages ───────────────────────────────
    let needed_h = cache.total_canvas_h as i32;
    let needed_w = geo.canvas_w() as i32;
    if widget.content_width()  != needed_w { widget.set_content_width(needed_w); }
    if widget.content_height() != needed_h { widget.set_content_height(needed_h); }

    // ── Canvas background ─────────────────────────────────────────────────
    cr.set_source_rgb(0.925, 0.922, 0.918);
    cr.paint().ok();

    // ── Pages ─────────────────────────────────────────────────────────────
    for p in 0..cache.page_count {
        let px = geo.page_x();
        let py = geo.page_top(p);
        let pw = geo.page_w;
        let ph = geo.page_h;

        // Shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.10);
        cr.rectangle(px + 3.0, py + 3.0, pw, ph);
        cr.fill().ok();

        // White page
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(px, py, pw, ph);
        cr.fill().ok();
    }

    // ── Selection highlights ──────────────────────────────────────────────
    if let Some(sel) = &state.selection {
        if !sel.is_collapsed() {
            let range = sel.to_range();
            for (i, cb) in cache.blocks.iter().enumerate() {
                paint_selection_for_block(cr, i, &range, cb, geo.content_x());
            }
        }
    }

    // ── Block content ─────────────────────────────────────────────────────
    let cx = geo.content_x();
    for (i, cb) in cache.blocks.iter().enumerate() {
        let block = &state.doc.blocks[i];

        // Block-kind decorations.
        match &block.kind {
            folio_core::BlockKind::Quote => {
                let (_, h) = cb.layout.pixel_size();
                cr.set_source_rgba(0.212, 0.522, 0.894, 0.7);
                cr.rectangle(cx - 12.0, cb.y_top, 3.0, h as f64);
                cr.fill().ok();
                cr.set_source_rgba(0.212, 0.522, 0.894, 0.07);
                cr.rectangle(cx - 12.0, cb.y_top, geo.content_w + 12.0, h as f64);
                cr.fill().ok();
            }
            folio_core::BlockKind::Code => {
                let (_, h) = cb.layout.pixel_size();
                cr.set_source_rgb(0.94, 0.94, 0.92);
                rounded_rect(cr, cx - 8.0, cb.y_top - 4.0, geo.content_w + 16.0, h as f64 + 8.0, 4.0);
                cr.fill().ok();
            }
            _ => {}
        }

        // Text colour.
        match &block.kind {
            folio_core::BlockKind::Caption => cr.set_source_rgb(0.4, 0.4, 0.4),
            folio_core::BlockKind::Quote   => cr.set_source_rgba(0.15, 0.15, 0.15, 0.85),
            _                              => cr.set_source_rgb(0.11, 0.11, 0.11),
        }

        cr.move_to(cx, cb.y_top);
        pangocairo::functions::show_layout(cr, &cb.layout);

        // Cursor
        let show_cursor = state.cursor_visible
            && i == state.cursor.block_idx
            && state.selection.as_ref().map(|s| s.is_collapsed()).unwrap_or(true);
        if show_cursor {
            paint_cursor(cr, &cb.layout, state.cursor.byte_offset, cb.y_top, cx);
        }
    }
}

fn paint_cursor(cr: &Context, layout: &pango::Layout, byte_off: usize, block_y: f64, content_x: f64) {
    let (strong, _) = layout.cursor_pos(byte_off as i32);
    let ps = pango::SCALE as f64;
    let x  = content_x + strong.x()      as f64 / ps;
    let y  = block_y   + strong.y()      as f64 / ps;
    let h  =             strong.height() as f64 / ps;
    cr.set_source_rgb(0.106, 0.431, 0.929);
    cr.set_line_width(1.5);
    cr.move_to(x, y);
    cr.line_to(x, y + h);
    cr.stroke().ok();
}

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    cr.new_sub_path();
    cr.arc(x + w - r, y + r,     r, -FRAC_PI_2,    0.0);
    cr.arc(x + w - r, y + h - r, r,  0.0,           FRAC_PI_2);
    cr.arc(x + r,     y + h - r, r,  FRAC_PI_2,     PI);
    cr.arc(x + r,     y + r,     r,  PI,            3.0 * FRAC_PI_2);
    cr.close_path();
}
