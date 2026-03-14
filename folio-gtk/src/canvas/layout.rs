//! Page layout cache — Phase 5: real pagination.
//!
//! `PageGeometry` is derived from the document's `PageSettings` (paper size,
//! margins, orientation) and drives all coordinate calculations.
//!
//! `LayoutCache::build()` assigns each block to a page and computes absolute
//! canvas-space y coordinates. Blocks that don't fit on the remaining space of
//! the current page are moved to the next page (no mid-block splitting).

use folio_core::{Document, BlockKind};
use folio_core::document::page::{PageSettings, Orientation};
use pango;

// ── Conversion ────────────────────────────────────────────────────────────────

/// Millimetres → pixels at 96 DPI.
const MM_TO_PX: f64 = 96.0 / 25.4;

// ── Page geometry ─────────────────────────────────────────────────────────────

/// All spatial constants for one document, derived from `PageSettings`.
/// All values are in screen pixels at 96 DPI.
#[derive(Debug, Clone)]
pub struct PageGeometry {
    pub page_w:       f64,
    pub page_h:       f64,
    pub margin_top:   f64,
    pub margin_bot:   f64,
    pub margin_left:  f64,
    /// Right margin (used by PDF export, Phase 8).
    #[allow(dead_code)]
    pub margin_right: f64,
    /// Usable content width (page_w − margins).
    pub content_w:    f64,
    /// Usable content height per page (page_h − margins).
    pub content_h:    f64,
    /// Gap between the canvas edge and the page rect.
    pub canvas_pad:   f64,
    /// Vertical gap between consecutive pages.
    pub page_gap:     f64,
}

impl PageGeometry {
    pub fn from_settings(page: &PageSettings) -> Self {
        let (w_mm, h_mm) = page.paper_size.dimensions();
        let (w_mm, h_mm) = match page.orientation {
            Orientation::Portrait  => (w_mm, h_mm),
            Orientation::Landscape => (h_mm, w_mm),
        };
        let pw = w_mm * MM_TO_PX;
        let ph = h_mm * MM_TO_PX;
        let mt = page.margins.top_mm    * MM_TO_PX;
        let mb = page.margins.bottom_mm * MM_TO_PX;
        let ml = page.margins.left_mm   * MM_TO_PX;
        let mr = page.margins.right_mm  * MM_TO_PX;
        PageGeometry {
            page_w: pw, page_h: ph,
            margin_top: mt, margin_bot: mb,
            margin_left: ml, margin_right: mr,
            content_w: pw - ml - mr,
            content_h: ph - mt - mb,
            canvas_pad: 40.0,
            page_gap:   24.0,
        }
    }

    /// Canvas-space X of the page's left edge (same for all pages).
    pub fn page_x(&self) -> f64 { self.canvas_pad }

    /// Canvas-space Y of page `n`'s top edge.
    pub fn page_top(&self, n: usize) -> f64 {
        self.canvas_pad + n as f64 * (self.page_h + self.page_gap)
    }

    /// Canvas-space X of the content area's left edge.
    pub fn content_x(&self) -> f64 { self.canvas_pad + self.margin_left }

    /// Canvas-space Y of the content area's top on page `n`.
    pub fn content_top(&self, n: usize) -> f64 {
        self.page_top(n) + self.margin_top
    }

    /// Canvas-space Y of the content area's bottom on page `n`.
    pub fn content_bot(&self, n: usize) -> f64 {
        self.page_top(n) + self.page_h - self.margin_bot
    }

    /// Total DrawingArea height needed for `n_pages` pages.
    pub fn total_canvas_h(&self, n_pages: usize) -> f64 {
        if n_pages == 0 { return self.canvas_pad * 2.0; }
        self.page_top(n_pages - 1) + self.page_h + self.canvas_pad
    }

    /// Total DrawingArea width.
    pub fn canvas_w(&self) -> f64 { self.page_w + self.canvas_pad * 2.0 }
}

// ── Cache types ───────────────────────────────────────────────────────────────

pub struct CachedBlock {
    pub layout:   pango::Layout,
    /// Which page (0-indexed) this block lives on (used by PDF export, Phase 8).
    #[allow(dead_code)]
    pub page_idx: usize,
    /// Canvas-space Y of block top.
    pub y_top:    f64,
    /// Canvas-space Y of block bottom (y_top + pixel height).
    pub y_bot:    f64,
}

pub struct LayoutCache {
    pub blocks:         Vec<CachedBlock>,
    pub geo:            PageGeometry,
    /// Number of pages required to fit all blocks.
    pub page_count:     usize,
    /// Total DrawingArea height in pixels.
    pub total_canvas_h: f64,
}

impl LayoutCache {
    pub fn build(doc: &Document, pctx: &pango::Context) -> Self {
        let geo      = PageGeometry::from_settings(&doc.page);
        let base_font = pango::FontDescription::from_string(
            &format!("{} {:.1}", doc.typography.font_family, doc.typography.font_size_pt)
        );
        let width_pu = (geo.content_w * pango::SCALE as f64) as i32;

        let mut blocks   = Vec::with_capacity(doc.blocks.len());
        let mut page_idx = 0usize;
        let mut y        = geo.content_top(0);

        for block in &doc.blocks {
            let layout = make_block_layout(pctx, &base_font, block, width_pu, doc);
            let (_, h) = layout.pixel_size();
            let h      = h as f64;
            let gap    = block_gap(&block.kind);

            // If block doesn't fit on remaining space of this page AND we have
            // at least started filling it, push to the next page.
            if h <= geo.content_h && y + h > geo.content_bot(page_idx) {
                page_idx += 1;
                y = geo.content_top(page_idx);
            }

            blocks.push(CachedBlock {
                layout,
                page_idx,
                y_top: y,
                y_bot: y + h,
            });

            y += h + gap;

            // If we're now past the bottom of this page, next block starts fresh.
            if y > geo.content_bot(page_idx) {
                page_idx += 1;
                y = geo.content_top(page_idx);
            }
        }

        let page_count     = page_idx + 1;
        let total_canvas_h = geo.total_canvas_h(page_count);
        LayoutCache { blocks, geo, page_count, total_canvas_h }
    }
}

// ── Block layout ──────────────────────────────────────────────────────────────

pub fn make_block_layout(
    pctx:      &pango::Context,
    base_font: &pango::FontDescription,
    block:     &folio_core::Block,
    width_pu:  i32,
    doc:       &Document,
) -> pango::Layout {
    let layout = pango::Layout::new(pctx);
    let mut font = base_font.clone();
    let pts = doc.typography.font_size_pt;
    let ps  = pango::SCALE as f64;

    match &block.kind {
        BlockKind::Title    => { font.set_size((pts * 2.2  * ps) as i32); font.set_weight(pango::Weight::Bold); }
        BlockKind::Heading1 => { font.set_size((pts * 1.6  * ps) as i32); font.set_weight(pango::Weight::Semibold); }
        BlockKind::Heading2 => { font.set_size((pts * 1.25 * ps) as i32); font.set_weight(pango::Weight::Semibold); }
        BlockKind::Caption  => { font.set_size((pts * 0.85 * ps) as i32); font.set_style(pango::Style::Italic); }
        BlockKind::Quote    => { font.set_style(pango::Style::Italic); }
        BlockKind::Code     => { font = pango::FontDescription::from_string(&format!("IBM Plex Mono {:.1}", pts * 0.9)); }
        _ => {}
    }

    layout.set_font_description(Some(&font));
    layout.set_width(width_pu);
    layout.set_wrap(pango::WrapMode::WordChar);

    let pango_align = match block.layout.alignment {
        folio_core::Alignment::Left      => pango::Alignment::Left,
        folio_core::Alignment::Center    => pango::Alignment::Center,
        folio_core::Alignment::Right     => pango::Alignment::Right,
        folio_core::Alignment::Justified => { layout.set_justify(true); pango::Alignment::Left }
    };
    layout.set_alignment(pango_align);

    let spacing_pu = ((doc.typography.line_height - 1.0) * pts * ps) as i32;
    layout.set_spacing(spacing_pu.max(0));

    let plain: String = block.content.iter().map(|r| r.text.as_str()).collect();
    layout.set_text(&plain);

    let attr_vec = build_inline_attrs(block);
    if !attr_vec.is_empty() {
        let al = pango::AttrList::new();
        for a in attr_vec { al.insert(a); }
        layout.set_attributes(Some(&al));
    }
    layout
}

fn build_inline_attrs(block: &folio_core::Block) -> Vec<pango::Attribute> {
    use folio_core::InlineAttr;
    let mut attrs    = Vec::new();
    let mut byte_pos = 0u32;

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

pub fn block_gap(kind: &BlockKind) -> f64 {
    match kind {
        BlockKind::Title    | BlockKind::Heading1 => 14.0,
        BlockKind::Heading2                       => 10.0,
        _                                         =>  6.0,
    }
}
