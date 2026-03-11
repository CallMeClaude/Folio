/// Hit-testing — converting between screen coordinates and DocPositions.
///
/// Phase 2: position_to_xy is handled inside render.rs via pango::Layout::cursor_pos.
/// Phase 4 will add xy_to_position (click-to-place cursor) using
/// pango::Layout::xy_to_index for each block's layout rect.

use folio_core::DocPosition;

/// Convert a DocPosition to (x, y) in content-area coordinates.
/// Returns None if the block index is out of range.
/// Full implementation in Phase 4 — requires layout cache from render pass.
pub fn position_to_xy(_pos: DocPosition) -> Option<(f64, f64)> {
    None // TODO Phase 4
}
