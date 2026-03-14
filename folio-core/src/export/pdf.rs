//! PDF export using Cairo's native PDF surface + Pango for text layout.
//!
//! Each document page becomes one PDF page. Block layout mirrors the
//! on-screen pagination logic (PageGeometry), so the PDF matches
//! exactly what the user sees in the editor.

use anyhow::{Context, Result};
use std::path::Path;
use cairo::PdfSurface;
use pango::prelude::*;
use crate::document::{Document, BlockKind};

// ── Points conversion (Cairo PDF works in points: 1 pt = 1/72 inch) ──────────

/// Millimetres → points.
const MM_TO_PT: f64 = 72.0 / 25.4;

// ── Public entry point ────────────────────────────────────────────────────────

/// Export `doc` as a PDF file at `output`.
pub fn export_pdf(doc: &Document, output: &Path) -> Result<()> {
    let (pw_mm, ph_mm) = doc.page.paper_size.dimensions();
    let (pw_mm, ph_mm) = match doc.page.orientation {
        crate::document::page::Orientation::Portrait  => (pw_mm, ph_mm),
        crate::document::page::Orientation::Landscape => (ph_mm, pw_mm),
    };
    let pw_pt = pw_mm * MM_TO_PT;
    let ph_pt = ph_mm * MM_TO_PT;

    let surface = PdfSurface::new(pw_pt, ph_pt, output)
        .context("failed to create PDF surface")?;
    surface.set_metadata(cairo::PdfMetadata::Title, &doc.title).ok();

    let cr = cairo::Context::new(&surface)
        .context("failed to create Cairo context")?;

    let mt = doc.page.margins.top_mm    * MM_TO_PT;
    let mb = doc.page.margins.bottom_mm * MM_TO_PT;
    let ml = doc.page.margins.left_mm   * MM_TO_PT;
    let mr = doc.page.margins.right_mm  * MM_TO_PT;
    let content_w = pw_pt - ml - mr;
    let content_h = ph_pt - mt - mb;

    let font_map: pango::FontMap = pangocairo::FontMap::new();
    let pctx = font_map.create_context();
    pangocairo::functions::update_context(&cr, &pctx);

    let base_font = pango::FontDescription::from_string(
        &format!("{} {:.1}", doc.typography.font_family, doc.typography.font_size_pt)
    );
    let width_pu = (content_w * pango::SCALE as f64) as i32;

    let mut page_y = mt; // current y on the current PDF page

    for block in &doc.blocks {
        // Divider gets special treatment — just a line, no Pango layout.
        if matches!(block.kind, BlockKind::Divider) {
            ensure_page_space(&cr, &surface, &mut page_y, mt, mb, ph_pt, 8.0);
            cr.set_source_rgb(0.6, 0.6, 0.6);
            cr.set_line_width(0.5);
            cr.move_to(ml, page_y + 4.0);
            cr.line_to(ml + content_w, page_y + 4.0);
            cr.stroke().ok();
            page_y += 8.0 + block_gap_pt(&block.kind);
            continue;
        }

        let layout = make_layout(&pctx, &base_font, block, width_pu, doc);
        let (_, h_pu) = layout.size();
        let h_pt = h_pu as f64 / pango::SCALE as f64;
        let gap  = block_gap_pt(&block.kind);

        // Page break if block doesn't fit.
        if page_y + h_pt > ph_pt - mb {
            cr.show_page().ok();
            surface.set_size(pw_pt, ph_pt).ok();
            page_y = mt;
        }

        // Quote bar.
        if matches!(block.kind, BlockKind::Quote) {
            cr.set_source_rgba(0.212, 0.522, 0.894, 0.7);
            cr.rectangle(ml - 8.0, page_y, 2.0, h_pt);
            cr.fill().ok();
        }

        // Code background.
        if matches!(block.kind, BlockKind::Code) {
            cr.set_source_rgb(0.94, 0.94, 0.92);
            cr.rectangle(ml - 4.0, page_y - 2.0, content_w + 8.0, h_pt + 4.0);
            cr.fill().ok();
        }

        // Text colour.
        let (r, g, b) = match &block.kind {
            BlockKind::Caption                       => (0.45, 0.45, 0.45),
            BlockKind::Quote                         => (0.15, 0.15, 0.15),
            BlockKind::CheckItem { checked: true }   => (0.55, 0.55, 0.55),
            _                                        => (0.08, 0.08, 0.08),
        };
        cr.set_source_rgb(r, g, b);
        cr.move_to(ml, page_y);
        pangocairo::functions::show_layout(&cr, &layout);

        // Bullet / number / checkbox decorations.
        match &block.kind {
            BlockKind::BulletItem => {
                cr.set_source_rgb(0.08, 0.08, 0.08);
                cr.arc(ml - 10.0, page_y + 5.0, 2.0, 0.0, std::f64::consts::TAU);
                cr.fill().ok();
            }
            BlockKind::OrderedItem { index } => {
                let label = format!("{}.", index);
                let lay2 = pango::Layout::new(&pctx);
                lay2.set_font_description(Some(&base_font));
                lay2.set_text(&label);
                let (lw, _) = lay2.pixel_size();
                cr.set_source_rgb(0.08, 0.08, 0.08);
                cr.move_to(ml - lw as f64 - 4.0, page_y);
                pangocairo::functions::show_layout(&cr, &lay2);
            }
            BlockKind::CheckItem { checked } => {
                let bx = ml - 16.0;
                let by = page_y + 2.0;
                cr.set_source_rgb(0.6, 0.6, 0.6);
                cr.set_line_width(0.8);
                cr.rectangle(bx, by, 10.0, 10.0);
                cr.stroke().ok();
                if *checked {
                    cr.set_source_rgb(0.106, 0.431, 0.929);
                    cr.set_line_width(1.2);
                    cr.move_to(bx + 2.0, by + 5.0);
                    cr.line_to(bx + 4.5, by + 7.5);
                    cr.line_to(bx + 8.5, by + 2.5);
                    cr.stroke().ok();
                }
            }
            _ => {}
        }

        page_y += h_pt + gap;
    }

    surface.finish();
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn ensure_page_space(
    cr:       &cairo::Context,
    surface:  &PdfSurface,
    page_y:   &mut f64,
    mt: f64, mb: f64, ph_pt: f64,
    needed: f64,
) {
    if *page_y + needed > ph_pt - mb {
        cr.show_page().ok();
        surface.set_size(ph_pt, ph_pt).ok(); // keep same size
        *page_y = mt;
    }
}

fn block_gap_pt(kind: &BlockKind) -> f64 {
    match kind {
        BlockKind::Title    | BlockKind::Heading1 => 10.0,
        BlockKind::Heading2                       =>  7.0,
        _                                         =>  4.5,
    }
}

fn make_layout(
    pctx:      &pango::Context,
    base_font: &pango::FontDescription,
    block:     &crate::document::Block,
    width_pu:  i32,
    doc:       &Document,
) -> pango::Layout {
    let layout   = pango::Layout::new(pctx);
    let mut font = base_font.clone();
    let pts      = doc.typography.font_size_pt;
    let ps       = pango::SCALE as f64;

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
        crate::document::Alignment::Left      => pango::Alignment::Left,
        crate::document::Alignment::Center    => pango::Alignment::Center,
        crate::document::Alignment::Right     => pango::Alignment::Right,
        crate::document::Alignment::Justified => { layout.set_justify(true); pango::Alignment::Left }
    };
    layout.set_alignment(pango_align);

    let spacing = ((doc.typography.line_height - 1.0) * pts * ps) as i32;
    layout.set_spacing(spacing.max(0));

    let plain: String = block.content.iter().map(|r| r.text.as_str()).collect();
    layout.set_text(&plain);

    // Inline attrs
    let mut attrs_list = pango::AttrList::new();
    let mut byte_pos = 0u32;
    for run in &block.content {
        let start = byte_pos;
        let end   = byte_pos + run.text.len() as u32;
        for attr in &run.attrs {
            use crate::document::InlineAttr;
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
            attrs_list.insert(a);
        }
        byte_pos = end;
    }
    layout.set_attributes(Some(&attrs_list));
    layout
}
