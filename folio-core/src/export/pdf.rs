use anyhow::Result;
use std::path::Path;
use crate::document::Document;

/// Export document as PDF using Cairo's PDF surface.
/// Full implementation comes in Phase 8 — this is a correct stub that compiles.
pub fn export_pdf(_doc: &Document, _output: &Path) -> Result<()> {
    // TODO Phase 8: create cairo::PdfSurface, lay out blocks with Pango,
    // paint via Cairo, and surface.finish().
    anyhow::bail!("PDF export not yet implemented — coming in Phase 8")
}
