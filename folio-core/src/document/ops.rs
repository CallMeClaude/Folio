//! Editing operations on a `Document`.
//!
//! All mutations go through these methods. They validate positions, keep
//! `InlineRun` spans consistent, and update `modified` timestamp.
//!
//! The canvas calls these; the CRDT layer checkpoints after each call.

use chrono::Utc;
use thiserror::Error;
use crate::document::{Document, Block, BlockKind, InlineRun, InlineAttr};
use crate::document::position::{DocPosition, DocRange};

#[derive(Debug, Error)]
pub enum EditError {
    #[error("block index {0} out of range (document has {1} blocks)")]
    BlockOutOfRange(usize, usize),
    #[error("byte offset {0} is not on a char boundary in block {1}")]
    BadByteOffset(usize, usize),
    #[error("cannot merge: block index {0} is the last block")]
    MergeAtEnd(usize),
}

type Result<T> = std::result::Result<T, EditError>;

// ─── Validation helpers ───────────────────────────────────────────────────────

impl Document {
    fn check_block(&self, idx: usize) -> Result<()> {
        if idx >= self.blocks.len() {
            Err(EditError::BlockOutOfRange(idx, self.blocks.len()))
        } else {
            Ok(())
        }
    }

    fn check_pos(&self, pos: DocPosition) -> Result<()> {
        self.check_block(pos.block_idx)?;
        let text = self.blocks[pos.block_idx].plain_text();
        if !text.is_char_boundary(pos.byte_offset) {
            Err(EditError::BadByteOffset(pos.byte_offset, pos.block_idx))
        } else {
            Ok(())
        }
    }

    fn touch(&mut self) {
        self.modified = Utc::now();
    }
}

// ─── insert_text ─────────────────────────────────────────────────────────────

impl Document {
    /// Insert `text` at `pos`. The text is inserted into the run that owns
    /// that byte offset, inheriting its inline attributes.
    pub fn insert_text(&mut self, pos: DocPosition, text: &str) -> Result<()> {
        if text.is_empty() { return Ok(()); }
        self.check_pos(pos)?;
        let block = &mut self.blocks[pos.block_idx];
        insert_into_runs(&mut block.content, pos.byte_offset, text);
        self.touch();
        Ok(())
    }
}

/// Insert `text` at `byte_offset` into a run list.
/// Splits the owning run, inserts, then normalises adjacent runs with
/// identical attrs.
fn insert_into_runs(runs: &mut Vec<InlineRun>, byte_offset: usize, text: &str) {
    if runs.is_empty() {
        runs.push(InlineRun::plain(text));
        return;
    }

    // Find which run owns this byte offset.
    let mut cursor = 0usize;
    for i in 0..runs.len() {
        let run_len = runs[i].text.len();
        if byte_offset <= cursor + run_len {
            // Offset relative to this run's start.
            let local = byte_offset - cursor;
            let before = runs[i].text[..local].to_string();
            let after  = runs[i].text[local..].to_string();
            let attrs  = runs[i].attrs.clone();

            // Build replacement: before + new + after, same attrs.
            let mut new_text = before;
            new_text.push_str(text);
            new_text.push_str(&after);
            runs[i].text = new_text;
            // No split needed — we just expanded the run in place.
            return;
        }
        cursor += run_len;
    }

    // Past the end — append to last run.
    if let Some(last) = runs.last_mut() {
        last.text.push_str(text);
    }
}

// ─── delete_range ─────────────────────────────────────────────────────────────

impl Document {
    /// Delete text in `range`. If the range spans multiple blocks the
    /// intermediate blocks are removed and the edge blocks are merged.
    pub fn delete_range(&mut self, range: DocRange) -> Result<()> {
        if range.is_collapsed() { return Ok(()); }
        self.check_pos(range.start)?;
        self.check_pos(range.end)?;

        if range.is_single_block() {
            let block = &mut self.blocks[range.start.block_idx];
            delete_from_runs(
                &mut block.content,
                range.start.byte_offset,
                range.end.byte_offset,
            );
        } else {
            // Delete tail of start block.
            let start_len = self.blocks[range.start.block_idx].plain_text().len();
            delete_from_runs(
                &mut self.blocks[range.start.block_idx].content,
                range.start.byte_offset,
                start_len,
            );
            // Delete head of end block.
            delete_from_runs(
                &mut self.blocks[range.end.block_idx].content,
                0,
                range.end.byte_offset,
            );
            // Remove fully-covered intermediate blocks (back-to-front).
            let remove_start = range.start.block_idx + 1;
            let remove_end   = range.end.block_idx;   // exclusive after +1
            if remove_start < remove_end {
                self.blocks.drain(remove_start..remove_end);
            }
            // Now end block is at remove_start; merge it into start block.
            self.merge_blocks(range.start.block_idx)?;
        }
        self.touch();
        Ok(())
    }
}

/// Delete bytes [from, to) from a run list.
fn delete_from_runs(runs: &mut Vec<InlineRun>, from: usize, to: usize) {
    if from >= to { return; }
    let mut cursor = 0usize;
    let mut i = 0;
    while i < runs.len() {
        let run_len = runs[i].text.len();
        let run_start = cursor;
        let run_end   = cursor + run_len;
        if run_end <= from || run_start >= to {
            // Entirely outside deletion range.
        } else {
            let del_start = from.saturating_sub(run_start);
            let del_end   = (to - run_start).min(run_len);
            runs[i].text.drain(del_start..del_end);
        }
        cursor += run_len;
        i += 1;
    }
    runs.retain(|r| !r.text.is_empty());
}

// ─── split_block (Enter key) ──────────────────────────────────────────────────

impl Document {
    /// Split the block at `pos` into two blocks. The second block inherits
    /// the kind of the first (except Title/Heading → Paragraph).
    pub fn split_block(&mut self, pos: DocPosition) -> Result<()> {
        self.check_pos(pos)?;

        // Clone what we need before taking any mutable borrow.
        let full_text = self.blocks[pos.block_idx].plain_text();
        let tail_text = full_text[pos.byte_offset..].to_string();
        let new_kind  = match &self.blocks[pos.block_idx].kind {
            BlockKind::Title | BlockKind::Heading1 | BlockKind::Heading2
                => BlockKind::Paragraph,
            other => other.clone(),
        };

        // Truncate current block to the split point.
        delete_from_runs(
            &mut self.blocks[pos.block_idx].content,
            pos.byte_offset,
            full_text.len(),
        );

        let mut new_block = Block::new(new_kind);
        if !tail_text.is_empty() {
            new_block.content = vec![InlineRun::plain(tail_text)];
        }

        self.blocks.insert(pos.block_idx + 1, new_block);
        self.touch();
        Ok(())
    }
}

// ─── merge_blocks (Backspace at block start) ──────────────────────────────────

impl Document {
    /// Merge block `idx+1` into block `idx`. Keeps block `idx`'s kind.
    pub fn merge_blocks(&mut self, idx: usize) -> Result<()> {
        self.check_block(idx)?;
        if idx + 1 >= self.blocks.len() {
            return Err(EditError::MergeAtEnd(idx));
        }
        let next = self.blocks.remove(idx + 1);
        self.blocks[idx].content.extend(next.content);
        normalise_runs(&mut self.blocks[idx].content);
        self.touch();
        Ok(())
    }
}

/// Merge adjacent runs that share identical attrs.
fn normalise_runs(runs: &mut Vec<InlineRun>) {
    if runs.len() < 2 { return; }
    let mut i = 0;
    while i + 1 < runs.len() {
        if runs[i].attrs == runs[i + 1].attrs {
            let text = runs[i + 1].text.clone();
            runs[i].text.push_str(&text);
            runs.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

// ─── apply_inline_attr ────────────────────────────────────────────────────────

impl Document {
    /// Add `attr` to every run (or run fragment) covered by `range`.
    /// Range must be within a single block.
    pub fn apply_inline_attr(
        &mut self,
        range: DocRange,
        attr:  InlineAttr,
    ) -> Result<()> {
        if range.is_collapsed() { return Ok(()); }
        self.check_pos(range.start)?;
        self.check_pos(range.end)?;
        debug_assert!(range.is_single_block(),
            "apply_inline_attr: range must be within a single block");

        let block = &mut self.blocks[range.start.block_idx];
        split_runs_at(&mut block.content, range.start.byte_offset);
        split_runs_at(&mut block.content, range.end.byte_offset);

        let mut cursor = 0usize;
        for run in block.content.iter_mut() {
            let run_end = cursor + run.text.len();
            if cursor >= range.start.byte_offset && run_end <= range.end.byte_offset {
                if !run.attrs.contains(&attr) {
                    run.attrs.push(attr.clone());
                }
            }
            cursor = run_end;
        }
        normalise_runs(&mut block.content);
        self.touch();
        Ok(())
    }

    /// Remove `attr` from every run covered by `range`.
    pub fn remove_inline_attr(
        &mut self,
        range: DocRange,
        attr:  &InlineAttr,
    ) -> Result<()> {
        if range.is_collapsed() { return Ok(()); }
        self.check_pos(range.start)?;
        self.check_pos(range.end)?;

        let block = &mut self.blocks[range.start.block_idx];
        split_runs_at(&mut block.content, range.start.byte_offset);
        split_runs_at(&mut block.content, range.end.byte_offset);

        let mut cursor = 0usize;
        for run in block.content.iter_mut() {
            let run_end = cursor + run.text.len();
            if cursor >= range.start.byte_offset && run_end <= range.end.byte_offset {
                run.attrs.retain(|a| a != attr);
            }
            cursor = run_end;
        }
        normalise_runs(&mut block.content);
        self.touch();
        Ok(())
    }
}

/// Split the run list at `byte_offset` so that `byte_offset` falls exactly
/// at a run boundary. No-op if it already does.
fn split_runs_at(runs: &mut Vec<InlineRun>, byte_offset: usize) {
    let mut cursor = 0usize;
    for i in 0..runs.len() {
        let run_len = runs[i].text.len();
        if cursor < byte_offset && byte_offset < cursor + run_len {
            let local = byte_offset - cursor;
            let right_text  = runs[i].text[local..].to_string();
            let right_attrs = runs[i].attrs.clone();
            runs[i].text.truncate(local);
            runs.insert(i + 1, InlineRun { text: right_text, attrs: right_attrs });
            return;
        }
        cursor += run_len;
        if cursor == byte_offset { return; } // already on boundary
    }
}

// ─── set_block_kind ───────────────────────────────────────────────────────────

impl Document {
    /// Change the kind of block `idx`.
    pub fn set_block_kind(&mut self, idx: usize, kind: BlockKind) -> Result<()> {
        self.check_block(idx)?;
        self.blocks[idx].kind = kind;
        self.touch();
        Ok(())
    }

    /// Return the number of blocks.
    pub fn block_count(&self) -> usize { self.blocks.len() }

    /// Return byte length of block `idx`'s plain text.
    pub fn block_text_len(&mut self, idx: usize) -> usize {
        self.blocks[idx].plain_text().len()
    }
}
