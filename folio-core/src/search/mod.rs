//! Document search — Find & Replace engine.
//!
//! Operates on `Document` with no GTK dependency.
//! Supports literal text and regex search, with optional case sensitivity.

use regex::{Regex, RegexBuilder};
use anyhow::Result;
use crate::document::{Document, DocPosition, DocRange};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub pattern:        String,
    pub case_sensitive: bool,
    pub use_regex:      bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub block_idx:  usize,
    pub byte_start: usize,
    pub byte_end:   usize,
}

impl SearchMatch {
    pub fn start_pos(&self) -> DocPosition { DocPosition::new(self.block_idx, self.byte_start) }
    pub fn end_pos(&self)   -> DocPosition { DocPosition::new(self.block_idx, self.byte_end) }
    pub fn to_range(&self)  -> DocRange    { DocRange::new(self.start_pos(), self.end_pos()) }
}

// ── Compiler ──────────────────────────────────────────────────────────────────

fn compile(q: &SearchQuery) -> Result<Regex> {
    let pat = if q.use_regex {
        q.pattern.clone()
    } else {
        regex::escape(&q.pattern)
    };
    RegexBuilder::new(&pat)
        .case_insensitive(!q.case_sensitive)
        .build()
        .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))
}

// ── Search ────────────────────────────────────────────────────────────────────

pub fn find_all(doc: &Document, query: &SearchQuery) -> Result<Vec<SearchMatch>> {
    if query.pattern.is_empty() { return Ok(vec![]); }
    let re = compile(query)?;
    let mut results = Vec::new();
    for (block_idx, block) in doc.blocks.iter().enumerate() {
        let text = block.plain_text();
        for m in re.find_iter(&text) {
            results.push(SearchMatch { block_idx, byte_start: m.start(), byte_end: m.end() });
        }
    }
    Ok(results)
}

/// Find next match after `after` — wraps around. Returns (match, did_wrap).
pub fn find_next(doc: &Document, q: &SearchQuery, after: DocPosition) -> Result<Option<(SearchMatch, bool)>> {
    let all = find_all(doc, q)?;
    if all.is_empty() { return Ok(None); }
    let idx = all.iter().position(|m|
        m.block_idx > after.block_idx
        || (m.block_idx == after.block_idx && m.byte_start > after.byte_offset)
    );
    Ok(Some(match idx {
        Some(i) => (all[i].clone(), false),
        None    => (all[0].clone(), true),
    }))
}

/// Find previous match before `before` — wraps around.
pub fn find_prev(doc: &Document, q: &SearchQuery, before: DocPosition) -> Result<Option<(SearchMatch, bool)>> {
    let all = find_all(doc, q)?;
    if all.is_empty() { return Ok(None); }
    let idx = all.iter().rposition(|m|
        m.block_idx < before.block_idx
        || (m.block_idx == before.block_idx && m.byte_end < before.byte_offset)
    );
    Ok(Some(match idx {
        Some(i) => (all[i].clone(), false),
        None    => (all[all.len()-1].clone(), true),
    }))
}

// ── Replace ───────────────────────────────────────────────────────────────────

/// Replace the text covered by `m` with `replacement` in the document.
pub fn replace_match(doc: &mut Document, m: &SearchMatch, replacement: &str) -> Result<()> {
    doc.delete_range(m.to_range())?;
    if !replacement.is_empty() {
        doc.insert_text(m.start_pos(), replacement)?;
    }
    Ok(())
}

/// Replace all matches. Returns the number of replacements made.
pub fn replace_all(doc: &mut Document, q: &SearchQuery, replacement: &str) -> Result<usize> {
    // Collect matches first, then replace back-to-front so offsets stay valid.
    let mut all = find_all(doc, q)?;
    let count = all.len();
    all.reverse();
    for m in &all {
        // Recompute the match position using regex to handle offset changes.
        replace_match(doc, m, replacement)?;
    }
    Ok(count)
}
