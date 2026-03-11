/// CRDT / undo-redo layer.
///
/// Phase 1 uses a shallow checkpoint model:
///   - `Vec<Block>` inside `Document` is the authoritative live state.
///   - Before each edit, the canvas calls `checkpoint(&doc)`.
///   - `undo()` restores the previous snapshot; `redo()` re-applies.
///
/// Public API:
///   engine.checkpoint(&doc)
///   engine.undo(&current_doc) -> Option<Document>
///   engine.redo(&current_doc) -> Option<Document>
///   engine.can_undo() -> bool
///   engine.can_redo() -> bool
///   engine.clear()

use crate::document::Document;

const MAX_HISTORY: usize = 200;

pub struct CrdtEngine {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
}

#[derive(Clone)]
struct Snapshot {
    json: String,
}

impl Snapshot {
    fn capture(doc: &Document) -> Self {
        Snapshot {
            json: serde_json::to_string(doc).unwrap_or_default(),
        }
    }

    fn restore(&self) -> Option<Document> {
        serde_json::from_str(&self.json).ok()
    }
}

impl CrdtEngine {
    pub fn new() -> Self {
        CrdtEngine {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Save state before an edit. Clears the redo stack.
    pub fn checkpoint(&mut self, doc: &Document) {
        self.redo_stack.clear();
        self.undo_stack.push(Snapshot::capture(doc));
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// Undo: restore previous state; push current onto redo stack.
    pub fn undo(&mut self, current: &Document) -> Option<Document> {
        let snap = self.undo_stack.pop()?;
        self.redo_stack.push(Snapshot::capture(current));
        snap.restore()
    }

    /// Redo: re-apply next state; push current onto undo stack.
    pub fn redo(&mut self, current: &Document) -> Option<Document> {
        let snap = self.redo_stack.pop()?;
        self.undo_stack.push(Snapshot::capture(current));
        snap.restore()
    }

    pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }

    /// Drop all history (call after loading a file).
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for CrdtEngine {
    fn default() -> Self { Self::new() }
}
