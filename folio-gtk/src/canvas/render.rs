//! Cairo paint pass — multi-page rendering with all block kinds.

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

    // ── Resize DrawingArea ────────────────────────────────────────────────
    let needed_h = cache.total_canvas_h as i32;
    let needed_w = geo.canvas_w() as i32;
    if widget.content_width()  != needed_w { widget.set_content_width(needed_w); }
    if widget.content_height() != needed_h { widget.set_content_height(needed_h); }

    // ── Canvas background ─────────────────────────────────────────────────
    cr.set_source_rgb(0.925, 0.922, 0.918);
    cr.paint().ok();

    // ── Pages ─────────────────────────────────────────────────────────────
    for p in 0..cache.page_count {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.10);
        cr.rectangle(geo.page_x() + 3.0, geo.page_top(p) + 3.0, geo.page_w, geo.page_h);
        cr.fill().ok();
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(geo.page_x(), geo.page_top(p), geo.page_w, geo.page_h);
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
    let cx  = geo.content_x();
    let cur = state.cursor.block_idx;

    for (i, cb) in cache.blocks.iter().enumerate() {
        let block = &state.doc.blocks[i];

        // Focus-mode alpha: dim every block except the one holding the cursor.
        let alpha = if state.focus_mode && i != cur { 0.28 } else { 1.0 };

        // Block-kind decorations.
        match &block.kind {
            folio_core::BlockKind::Quote => {
                let (_, h) = cb.layout.pixel_size();
                cr.set_source_rgba(0.212, 0.522, 0.894, 0.7 * alpha);
                cr.rectangle(cx - 12.0, cb.y_top, 3.0, h as f64);
                cr.fill().ok();
                cr.set_source_rgba(0.212, 0.522, 0.894, 0.07 * alpha);
                cr.rectangle(cx - 12.0, cb.y_top, geo.content_w + 12.0, h as f64);
                cr.fill().ok();
            }
            folio_core::BlockKind::Code => {
                let (_, h) = cb.layout.pixel_size();
                cr.set_source_rgba(0.94, 0.94, 0.92, alpha);
                rounded_rect(cr, cx - 8.0, cb.y_top - 4.0, geo.content_w + 16.0, h as f64 + 8.0, 4.0);
                cr.fill().ok();
            }
            folio_core::BlockKind::BulletItem => {
                // Bullet dot
                let dot_x = cx - 14.0;
                let dot_y = cb.y_top + 7.0;
                cr.set_source_rgba(0.11, 0.11, 0.11, alpha);
                cr.arc(dot_x, dot_y, 2.5, 0.0, std::f64::consts::TAU);
                cr.fill().ok();
            }
            folio_core::BlockKind::OrderedItem { index } => {
                let label = format!("{}.", index);
                let ps    = pango::SCALE as f64;
                let font  = pango::FontDescription::from_string(
                    &format!("{} {:.1}", block_num_font(&state.doc.typography.font_family),
                             state.doc.typography.font_size_pt));
                let pctx  = pangocairo::functions::create_context(cr);
                let lay   = pango::Layout::new(&pctx);
                lay.set_font_description(Some(&font));
                lay.set_text(&label);
                let (tw, _) = lay.pixel_size();
                cr.set_source_rgba(0.11, 0.11, 0.11, alpha);
                cr.move_to(cx - 8.0 - tw as f64, cb.y_top);
                pangocairo::functions::show_layout(cr, &lay);
            }
            folio_core::BlockKind::CheckItem { checked } => {
                paint_checkbox(cr, cx - 20.0, cb.y_top + 2.0, *checked, alpha);
            }
            folio_core::BlockKind::Divider => {
                let (_, h) = cb.layout.pixel_size();
                let mid = cb.y_top + h as f64 / 2.0;
                cr.set_source_rgba(0.6, 0.6, 0.6, alpha);
                cr.set_line_width(1.0);
                cr.move_to(cx, mid);
                cr.line_to(cx + geo.content_w, mid);
                cr.stroke().ok();
            }
            _ => {}
        }

        // Text colour.
        let (r, g, b) = match &block.kind {
            folio_core::BlockKind::Caption                          => (0.4, 0.4, 0.4),
            folio_core::BlockKind::Quote                           => (0.15, 0.15, 0.15),
            folio_core::BlockKind::CheckItem { checked: true }     => (0.5, 0.5, 0.5),
            _                                                       => (0.11, 0.11, 0.11),
        };
        cr.set_source_rgba(r, g, b, alpha);
        cr.move_to(cx, cb.y_top);
        pangocairo::functions::show_layout(cr, &cb.layout);

        // Cursor
        if state.cursor_visible && i == cur
            && state.selection.as_ref().map(|s| s.is_collapsed()).unwrap_or(true)
        {
            paint_cursor(cr, &cb.layout, state.cursor.byte_offset, cb.y_top, cx,
                         state.cursor_style);
        }
    }
}

fn paint_cursor(cr: &Context, layout: &pango::Layout, byte_off: usize,
                block_y: f64, content_x: f64,
                style: crate::canvas::cursor::CursorStyle) {
    use crate::canvas::cursor::CursorStyle;
    let (strong, _) = layout.cursor_pos(byte_off as i32);
    let ps = pango::SCALE as f64;
    let x  = content_x + strong.x()      as f64 / ps;
    let y  = block_y   + strong.y()      as f64 / ps;
    let h  =             strong.height() as f64 / ps;

    cr.set_source_rgb(0.106, 0.431, 0.929);
    match style {
        CursorStyle::IBeam => {
            cr.set_line_width(1.5);
            cr.move_to(x, y);
            cr.line_to(x, y + h);
            cr.stroke().ok();
        }
        CursorStyle::Block => {
            cr.set_source_rgba(0.106, 0.431, 0.929, 0.35);
            cr.rectangle(x, y, 8.0, h);
            cr.fill().ok();
        }
        CursorStyle::Underscore => {
            cr.set_line_width(2.0);
            cr.move_to(x, y + h);
            cr.line_to(x + 8.0, y + h);
            cr.stroke().ok();
        }
    }
}

fn paint_checkbox(cr: &Context, x: f64, y: f64, checked: bool, alpha: f64) {
    let size = 13.0;
    // Box
    cr.set_source_rgba(0.7, 0.7, 0.7, alpha);
    rounded_rect(cr, x, y, size, size, 3.0);
    cr.set_line_width(1.2);
    cr.stroke().ok();
    if checked {
        // Checkmark
        cr.set_source_rgba(0.106, 0.431, 0.929, alpha);
        cr.set_line_width(1.8);
        cr.move_to(x + 2.5, y + 6.5);
        cr.line_to(x + 5.5, y + 9.5);
        cr.line_to(x + 10.5, y + 3.5);
        cr.stroke().ok();
    }
}

fn block_num_font(family: &str) -> &str {
    // Use the same family for list numbers
    family
}

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    cr.new_sub_path();
    cr.arc(x + w - r, y + r,     r, -FRAC_PI_2,        0.0);
    cr.arc(x + w - r, y + h - r, r,  0.0,               FRAC_PI_2);
    cr.arc(x + r,     y + h - r, r,  FRAC_PI_2,         PI);
    cr.arc(x + r,     y + r,     r,  PI,      3.0 * FRAC_PI_2);
    cr.close_path();
}
