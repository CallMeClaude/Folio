//! Hit-testing: map screen (x, y) → DocPosition.
//!
//! Uses the LayoutCache computed by render.rs. If the cache is not yet
//! populated (before any draw), we build a temporary one.

use pango::prelude::*;
use folio_core::DocPosition;
use crate::canvas::EditorState;
use crate::canvas::layout::{LayoutCache, CONTENT_X, CONTENT_Y};

/// Convert canvas coordinates (x, y) to a DocPosition.
/// Returns None only if the document has no blocks.
pub fn xy_to_position(state: &EditorState, x: f64, y: f64) -> Option<DocPosition> {
    {
        let cache_ref = state.layout_cache.borrow();
        if let Some(cache) = cache_ref.as_ref() {
            return Some(hit_in_cache(cache, x, y));
        }
    }
    // Cache empty — build a throwaway one without a Cairo context.
    let font_map: pango::FontMap = pangocairo::FontMap::new();
    let pctx = font_map.create_context();
    let cache = LayoutCache::build(&state.doc, &pctx);
    Some(hit_in_cache(&cache, x, y))
}

fn hit_in_cache(cache: &LayoutCache, x: f64, y: f64) -> DocPosition {
    if cache.blocks.is_empty() {
        return DocPosition::block_start(0);
    }

    let block_idx = find_block_idx(cache, y);
    let cb        = &cache.blocks[block_idx];

    let local_x = ((x - CONTENT_X).max(0.0) * pango::SCALE as f64) as i32;
    let local_y = ((y - cb.y_top ).max(0.0) * pango::SCALE as f64) as i32;

    let (_inside, index, trailing) = cb.layout.xy_to_index(local_x, local_y);

    let text = cb.layout.text();
    let byte_offset = if trailing > 0 {
        next_char_boundary(text.as_str(), index as usize)
    } else {
        index as usize
    };

    DocPosition::new(block_idx, byte_offset.min(text.len()))
}

fn find_block_idx(cache: &LayoutCache, y: f64) -> usize {
    let n = cache.blocks.len();
    for i in 0..n {
        let cb = &cache.blocks[i];
        if y < cb.y_top { return i; }
        if y <= cb.y_bot { return i; }
    }
    n - 1
}

fn next_char_boundary(s: &str, from: usize) -> usize {
    if from >= s.len() { return s.len(); }
    (from + 1..=s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len())
}
