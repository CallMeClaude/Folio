//! Page layout cache.
//!
//! Every time the document changes the cache is invalidated (set to None in
//! EditorState). The render pass recomputes it and stores it back. The
//! hittest pass reads it to map screen coordinates → DocPosition.
//!
//! All coordinates are in *canvas* space (pixels from the top-left of the
//! DrawingArea widget).

use folio_core::{Document, BlockKind};
use pango;

// ── Page geometry (pixels at 96 DPI) ─────────────────────────────────────────

/// A4 page width in pixels at 96 DPI.
pub const PAGE_W:    f64 = 793.7;
/// A4 page height in pixels at 96 DPI.
pub const PAGE_H:    f64 = 1122.5;
/// Gap between the DrawingArea edge and the page rect.
pub const PAGE_PAD:  f64 = 40.0;
/// Left/right/top/bottom content margin in pixels (≈25.4 mm at 96 DPI).
pub const PAGE_MARGIN: f64 = 96.0;

/// X-coordinate of the content area's left edge (canvas space).
pub const CONTENT_X: f64 = PAGE_PAD + PAGE_MARGIN;
/// Y-coordinate of the content area's top edge (canvas space).
pub const CONTENT_Y: f64 = PAGE_PAD + PAGE_MARGIN;
/// Width of the content area in pixels.
pub const CONTENT_W: f64 = PAGE_W - PAGE_MARGIN * 2.0;

// ── Cache types ───────────────────────────────────────────────────────────────

/// Pango layout + position for one block.
pub struct CachedBlock {
    /// The computed pango layout (text + font attributes + wrap width).
    pub layout: pango::Layout,
    /// Top of this block in canvas coordinates.
    pub y_top:  f64,
    /// Bottom of this block in canvas coordinates (y_top + pixel_height).
    pub y_bot:  f64,
}

/// Cached layout for the entire document. Stored in EditorState and rebuilt
/// whenever the document changes.
pub struct LayoutCache {
    pub blocks: Vec<CachedBlock>,
}

impl LayoutCache {
    /// Build the layout cache from the current document state.
    /// `pctx` must be a Pango context configured for the target surface
    /// (produced by `pangocairo::functions::create_context(cr)` or
    ///  `pangocairo::FontMap::new().create_context()`).
    pub fn build(doc: &Document, pctx: &pango::Context) -> Self {
        let size  = doc.typography.font_size_pt;
        let font  = pango::FontDescription::from_string(
            &format!("{} {:.1}", doc.typography.font_family, size)
        );
        let width_pu = (CONTENT_W * pango::SCALE as f64) as i32;

        let mut blocks = Vec::with_capacity(doc.blocks.len());
        let mut y = CONTENT_Y;

        for block in &doc.blocks {
            let layout = make_block_layout(pctx, &font, block, width_pu, doc);
            let (_, h) = layout.pixel_size();
            let height  = h as f64;
            let gap     = block_gap(&block.kind);

            blocks.push(CachedBlock {
                layout,
                y_top: y,
                y_bot: y + height,
            });

            y += height + gap;
        }

        LayoutCache { blocks }
    }
}

// ── Block layout helper ───────────────────────────────────────────────────────

pub fn make_block_layout(
    pctx:     &pango::Context,
    base_font: &pango::FontDescription,
    block:    &folio_core::Block,
    width_pu: i32,
    doc:      &Document,
) -> pango::Layout {
    let layout = pango::Layout::new(pctx);

    // Override font for heading styles.
    let mut font = base_font.clone();
    match &block.kind {
        BlockKind::Title    => { font.set_size((doc.typography.font_size_pt * 2.2 * pango::SCALE as f64) as i32); font.set_weight(pango::Weight::Bold); }
        BlockKind::Heading1 => { font.set_size((doc.typography.font_size_pt * 1.6 * pango::SCALE as f64) as i32); font.set_weight(pango::Weight::Semibold); }
        BlockKind::Heading2 => { font.set_size((doc.typography.font_size_pt * 1.25 * pango::SCALE as f64) as i32); font.set_weight(pango::Weight::Semibold); }
        BlockKind::Caption  => { font.set_size((doc.typography.font_size_pt * 0.85 * pango::SCALE as f64) as i32); }
        BlockKind::Code     => { font = pango::FontDescription::from_string(&format!("IBM Plex Mono {:.1}", doc.typography.font_size_pt)); }
        _ => {}
    }

    layout.set_font_description(Some(&font));
    layout.set_width(width_pu);
    layout.set_wrap(pango::WrapMode::WordChar);

    // Build attributed text from inline runs.
    let plain: String = block.content.iter().map(|r| r.text.as_str()).collect();
    layout.set_text(&plain);

    let attrs = build_inline_attrs(block);
    if !attrs.is_empty() {
        let attr_list = pango::AttrList::new();
        for a in attrs { attr_list.insert(a); }
        layout.set_attributes(Some(&attr_list));
    }

    layout
}

/// Build a Vec of pango::Attribute from a block's inline runs.
fn build_inline_attrs(block: &folio_core::Block) -> Vec<pango::Attribute> {
    use folio_core::InlineAttr;
    let mut attrs = Vec::new();
    let mut byte_pos: u32 = 0;

    for run in &block.content {
        let start = byte_pos;
        let end   = byte_pos + run.text.len() as u32;

        for attr in &run.attrs {
            let mut a: pango::Attribute = match attr {
                InlineAttr::Bold          => pango::AttrInt::new_weight(pango::Weight::Bold).into(),
                InlineAttr::Italic        => pango::AttrInt::new_style(pango::Style::Italic).into(),
                InlineAttr::Underline     => pango::AttrInt::new_underline(pango::Underline::Single).into(),
                InlineAttr::Strikethrough => pango::AttrInt::new_strikethrough(true).into(),
                InlineAttr::Superscript   => pango::AttrInt::new_rise(6000).into(),
                InlineAttr::Subscript     => pango::AttrInt::new_rise(-3000).into(),
                InlineAttr::TextColor(rgb) => {
                    let r = ((*rgb >> 16) & 0xFF) as u16 * 257;
                    let g = ((*rgb >>  8) & 0xFF) as u16 * 257;
                    let b = ( *rgb        & 0xFF) as u16 * 257;
                    pango::AttrColor::new_foreground(r, g, b).into()
                }
                InlineAttr::Highlight(rgb) => {
                    let r = ((*rgb >> 16) & 0xFF) as u16 * 257;
                    let g = ((*rgb >>  8) & 0xFF) as u16 * 257;
                    let b = ( *rgb        & 0xFF) as u16 * 257;
                    pango::AttrColor::new_background(r, g, b).into()
                }
                InlineAttr::Link(_) => pango::AttrInt::new_underline(pango::Underline::Single).into(),
            };
            a.set_start_index(start);
            a.set_end_index(end);
            attrs.push(a);
        }
        byte_pos = end;
    }
    attrs
}

/// Vertical gap below a block (pixels).
pub fn block_gap(kind: &BlockKind) -> f64 {
    match kind {
        BlockKind::Title    | BlockKind::Heading1 => 14.0,
        BlockKind::Heading2                       => 10.0,
        _                                         =>  6.0,
    }
}
