use serde::{Deserialize, Serialize};

/// A position within the document: which block, and how many bytes into
/// that block's flat text content.
///
/// All byte offsets must fall on a UTF-8 character boundary.
/// `Document` methods enforce this — callers never slice strings directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocPosition {
    pub block_idx:   usize,
    pub byte_offset: usize,
}

impl DocPosition {
    pub fn new(block_idx: usize, byte_offset: usize) -> Self {
        DocPosition { block_idx, byte_offset }
    }

    /// Position at the very start of a block.
    pub fn block_start(block_idx: usize) -> Self {
        DocPosition { block_idx, byte_offset: 0 }
    }
}

/// An inclusive byte range within a single block, or spanning multiple blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocRange {
    pub start: DocPosition,
    pub end:   DocPosition,
}

impl DocRange {
    pub fn new(start: DocPosition, end: DocPosition) -> Self {
        debug_assert!(start <= end, "DocRange start must be <= end");
        DocRange { start, end }
    }

    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    /// True if start and end are in the same block.
    pub fn is_single_block(&self) -> bool {
        self.start.block_idx == self.end.block_idx
    }
}
