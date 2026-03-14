//! Selection state and painting helpers.

use folio_core::{DocPosition, DocRange};
use cairo::Context;
use crate::canvas::layout::{CachedBlock, CONTENT_X};

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub anchor: DocPosition,
    pub active: DocPosition,
}

impl SelectionState {
    pub fn new(pos: DocPosition) -> Self {
        SelectionState { anchor: pos, active: pos }
    }

    pub fn is_collapsed(&self) -> bool { self.anchor == self.active }

    pub fn to_range(&self) -> DocRange {
        let (start, end) = if self.anchor <= self.active {
            (self.anchor, self.active)
        } else {
            (self.active, self.anchor)
        };
        DocRange::new(start, end)
    }
}

// ── Painting ──────────────────────────────────────────────────────────────────

/// Paint selection highlight for one block.
///
/// Uses `LayoutIter::line_yrange()` for correct absolute y coords within the
/// layout (relative to layout top, not line baseline) then adds `cb.y_top`.
pub fn paint_selection_for_block(
    cr:        &Context,
    block_idx: usize,
    sel:       &DocRange,
    cb:        &CachedBlock,
) {
    if block_idx < sel.start.block_idx || block_idx > sel.end.block_idx {
        return;
    }

    let text_len = cb.layout.text().len() as i32;
    let sel_start = if block_idx == sel.start.block_idx {
        sel.start.byte_offset as i32
    } else { 0 };
    let sel_end = if block_idx == sel.end.block_idx {
        sel.end.byte_offset as i32
    } else { text_len };

    if sel_start >= sel_end { return; }

    let ps = pango::SCALE as f64;
    cr.set_source_rgba(0.212, 0.522, 0.894, 0.28);

    // LayoutIter gives us absolute y positions within the layout.
    let mut iter = cb.layout.iter();
    loop {
        let line = match iter.line_readonly() {
            Some(l) => l,
            None    => break,
        };
        let line_start = line.start_index();
        let line_end   = line_start + line.length();

        if sel_end > line_start && sel_start < line_end {
            // x: from index_to_x on the line
            let x0_pu = line.index_to_x(sel_start.max(line_start), false);
            let x1_pu = line.index_to_x(sel_end.min(line_end),     false);

            // y: from line_yrange — gives absolute pango-unit y within layout
            let (y0_pu, y1_pu) = iter.line_yrange();
            let line_y = cb.y_top + y0_pu as f64 / ps;
            let line_h = (y1_pu - y0_pu) as f64 / ps;

            let x0 = CONTENT_X + x0_pu as f64 / ps;
            let x1 = CONTENT_X + x1_pu as f64 / ps;
            let w  = (x1 - x0).abs().max(1.0);
            cr.rectangle(x0.min(x1), line_y, w, line_h);
        }

        if !iter.next_line() { break; }
    }
    cr.fill().ok();
}
