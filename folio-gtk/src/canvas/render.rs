use cairo::Context;
use folio_core::{Block, BlockKind};
use crate::canvas::EditorState;

// A4 at 96 DPI (pixels).
const PAGE_W: f64 = 793.7;
const PAGE_H: f64 = 1122.5;
const PAD:    f64 = 40.0;   // gap between window edge and page
const MARGIN: f64 = 96.0;   // 25.4 mm at 96 DPI

pub fn draw(cr: &Context, state: &EditorState) {
    // ── Canvas background ──────────────────────────────────────────────────
    cr.set_source_rgb(0.925, 0.922, 0.918);
    cr.paint().ok();

    // ── Page drop-shadow ───────────────────────────────────────────────────
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.10);
    cr.rectangle(PAD + 3.0, PAD + 3.0, PAGE_W, PAGE_H);
    cr.fill().ok();

    // ── White page ─────────────────────────────────────────────────────────
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.rectangle(PAD, PAD, PAGE_W, PAGE_H);
    cr.fill().ok();

    // ── Content ────────────────────────────────────────────────────────────
    cr.save().ok();
    cr.translate(PAD + MARGIN, PAD + MARGIN);
    draw_content(cr, state, PAGE_W - MARGIN * 2.0);
    cr.restore().ok();
}

fn draw_content(cr: &Context, state: &EditorState, content_w: f64) {
    let pctx = pangocairo::functions::create_context(cr);
    let size  = state.doc.typography.font_size_pt as i32;
    let font  = pango::FontDescription::from_string(&format!("Lora {}", size));

    let mut y = 0.0_f64;
    for (idx, block) in state.doc.blocks.iter().enumerate() {
        let layout = make_layout(&pctx, &font, block, content_w);

        cr.set_source_rgb(0.11, 0.11, 0.11);
        cr.move_to(0.0, y);
        pangocairo::functions::show_layout(cr, &layout);

        if idx == state.cursor.block_idx && state.cursor_visible {
            paint_cursor(cr, &layout, state.cursor.byte_offset, y);
        }

        let (_, h) = layout.pixel_size();
        y += h as f64 + block_gap(&block.kind);
    }
}

fn make_layout(
    ctx:   &pango::Context,
    font:  &pango::FontDescription,
    block: &Block,
    width: f64,
) -> pango::Layout {
    let l = pango::Layout::new(ctx);
    l.set_font_description(Some(font));
    l.set_width((width * pango::SCALE as f64) as i32);
    l.set_wrap(pango::WrapMode::WordChar);
    l.set_text(&block.plain_text());
    l
}

fn paint_cursor(cr: &Context, layout: &pango::Layout, byte_off: usize, block_y: f64) {
    let (strong, _) = layout.cursor_pos(byte_off as i32);
    let s = pango::SCALE as f64;
    let x = strong.x() as f64 / s;
    let y = block_y + strong.y() as f64 / s;
    let h = strong.height() as f64 / s;

    cr.set_source_rgb(0.106, 0.431, 0.929); // Adwaita accent blue
    cr.set_line_width(1.5);
    cr.move_to(x, y);
    cr.line_to(x, y + h);
    cr.stroke().ok();
}

fn block_gap(kind: &BlockKind) -> f64 {
    match kind {
        BlockKind::Title | BlockKind::Heading1 => 14.0,
        BlockKind::Heading2                    => 10.0,
        _                                      =>  6.0,
    }
}
