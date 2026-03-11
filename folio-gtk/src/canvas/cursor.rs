/// Cursor style — Phase 9 will add user preference (I-beam / block / underscore).
/// For now the cursor is always a blinking 1.5 px I-beam painted by render.rs.
/// This module will hold cursor-style config and per-style paint helpers.

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CursorStyle {
    #[default]
    IBeam,
    Block,
    Underscore,
}
