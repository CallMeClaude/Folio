/// CRDT layer — wraps `loro` for undo/redo and document state tracking.
///
/// The loro `LoroDoc` is the source of truth. The `Document` struct in
/// `document/mod.rs` is a derived view that gets rebuilt after every
/// CRDT operation and is what the UI reads from.
use anyhow::Result;
use loro::{LoroDoc, UndoManager};
use crate::document::Document;

pub struct CrdtEngine {
    pub doc:  LoroDoc,
    pub undo: UndoManager,
}

impl CrdtEngine {
    /// Create a new engine from an empty document.
    pub fn new() -> Self {
        let doc  = LoroDoc::new();
        let undo = UndoManager::new(&doc);
        CrdtEngine { doc, undo }
    }

    /// Undo the last operation. Returns true if something was undone.
    pub fn undo(&mut self) -> Result<bool> {
        Ok(self.undo.undo().is_ok())
    }

    /// Redo the last undone operation. Returns true if something was redone.
    pub fn redo(&mut self) -> Result<bool> {
        Ok(self.undo.redo().is_ok())
    }

    /// Export a snapshot of the loro doc as bytes (for autosave).
    pub fn snapshot(&self) -> Vec<u8> {
        self.doc.export(loro::ExportMode::Snapshot).unwrap_or_default()
    }

    /// Import a snapshot produced by `snapshot()`.
    pub fn load_snapshot(&mut self, data: &[u8]) -> Result<()> {
        self.doc.import(data)?;
        Ok(())
    }
}

impl Default for CrdtEngine {
    fn default() -> Self { Self::new() }
}
