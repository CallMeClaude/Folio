//! Selection state and painting helpers.
//!
//! A selection is (anchor, active):
//!   anchor — where mouse was pressed or shift-movement started
//!   active — where the cursor currently is
//!
//! The highlight covers [min(anchor,active), max(anchor,active)).

use folio_core::{DocPosition, DocRange};
use cairo::Context;
use crate::canvas::layout::{CachedBlock, CONTENT_X};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub anchor: DocPosition,
    pub active: DocPosition,
}

impl SelectionState {
    pub fn new(pos: DocPosition) -> Self {
        SelectionState { anchor: pos, active: pos }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.active
    }

    /// Return the selection as an ordered DocRange.
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

/// Paint the selection highlight for one block.
/// Uses layout lines + `index_to_x` — compatible with pango 0.20.
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

    // Byte range within this block that is selected.
    let sel_start = if block_idx == sel.start.block_idx {
        sel.start.byte_offset as i32
    } else {
        0
    };
    let sel_end = if block_idx == sel.end.block_idx {
        sel.end.byte_offset as i32
    } else {
        text_len
    };

    if sel_start >= sel_end { return; }

    let ps = pango::SCALE as f64;
    cr.set_source_rgba(0.212, 0.522, 0.894, 0.28);

    for line in cb.layout.lines_readonly() {
        let line_start = line.start_index();
        let line_end   = line_start + line.length();

        // Does the selection overlap this line?
        if sel_end <= line_start || sel_start >= line_end { continue; }

        // x coordinates for the selection endpoints on this line.
        let x0_pu = line.index_to_x(sel_start.max(line_start), false);
        let x1_pu = line.index_to_x(sel_end.min(line_end), false);

        // y position from line logical extents.
        let (_, logical) = line.extents();
        let line_y = cb.y_top + logical.y() as f64 / ps;
        let line_h = logical.height() as f64 / ps;

        let x0 = CONTENT_X + x0_pu as f64 / ps;
        let x1 = CONTENT_X + x1_pu as f64 / ps;
        let w  = (x1 - x0).abs().max(1.0);
        let x  = x0.min(x1);

        cr.rectangle(x, line_y, w, line_h);
    }
    cr.fill().ok();
}
