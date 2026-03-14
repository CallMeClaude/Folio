//! CRDT layer — wraps `loro::LoroDoc` + `loro::UndoManager`.
//!
//! LoroDoc is the authoritative persistent store. `Vec<Block>` in `Document`
//! is a fast read-cache for rendering. After every undo/redo the block list is
//! rebuilt from the loro snapshot.
//!
//! Storage layout inside the LoroDoc:
//!   "doc_json" : LoroText  — full Document serialised as JSON
//!
//! On every checkpoint the latest JSON is written to "doc_json" and committed,
//! then `UndoManager::record_new_checkpoint` is called. Undo/redo reverts the
//! LoroText, then we deserialise back into a Document.
//!
//! .folio bundles store `doc.export(Snapshot)` bytes in `content.loro`.
//! document.json inside the bundle holds *only* metadata (title, paper size,
//! dates) — the block tree lives exclusively in the loro snapshot.

use anyhow::{Context, Result};
use loro::{ExportMode, LoroDoc, UndoManager};
use crate::document::Document;

const DOC_CONTAINER: &str = "doc_json";

pub struct CrdtEngine {
    doc:  LoroDoc,
    undo: UndoManager,
}

impl std::fmt::Debug for CrdtEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrdtEngine").finish_non_exhaustive()
    }
}

impl CrdtEngine {
    /// Create a fresh engine (empty document, empty history).
    pub fn new() -> Self {
        let doc  = LoroDoc::new();
        // Prime the container so it exists before UndoManager attaches.
        let _ = doc.get_text(DOC_CONTAINER);
        doc.commit();
        let undo = UndoManager::new(&doc);
        CrdtEngine { doc, undo }
    }

    // ── Checkpointing ────────────────────────────────────────────────────────

    /// Snapshot the current document state into LoroDoc and record an undo
    /// checkpoint. Call this *before* every user-visible edit.
    pub fn checkpoint(&mut self, document: &Document) -> Result<()> {
        let json = serde_json::to_string(document)
            .context("failed to serialise document for checkpoint")?;
        let text    = self.doc.get_text(DOC_CONTAINER);
        let old_len = text.len_unicode();
        if old_len > 0 {
            text.delete(0, old_len).context("loro: delete old snapshot")?;
        }
        text.insert(0, &json).context("loro: insert new snapshot")?;
        self.doc.commit();
        self.undo.record_new_checkpoint()
            .context("loro: record_new_checkpoint")?;
        Ok(())
    }

    // ── Undo / Redo ──────────────────────────────────────────────────────────

    /// Undo the last checkpoint. Returns the restored Document, or None if
    /// there is nothing to undo.
    pub fn undo(&mut self) -> Result<Option<Document>> {
        if !self.undo.can_undo() { return Ok(None); }
        self.undo.undo().context("loro: undo")?;
        Ok(Some(self.read_document()?))
    }

    /// Redo the last undone checkpoint. Returns the restored Document, or
    /// None if the redo stack is empty.
    pub fn redo(&mut self) -> Result<Option<Document>> {
        if !self.undo.can_redo() { return Ok(None); }
        self.undo.redo().context("loro: redo")?;
        Ok(Some(self.read_document()?))
    }

    pub fn can_undo(&self) -> bool { self.undo.can_undo() }
    pub fn can_redo(&self) -> bool { self.undo.can_redo() }

    // ── Persistence ──────────────────────────────────────────────────────────

    /// Export the full loro snapshot as raw bytes (stored as content.loro
    /// inside the .folio ZIP bundle).
    pub fn export_snapshot(&self) -> Result<Vec<u8>> {
        self.doc.export(ExportMode::Snapshot)
            .context("loro: export snapshot")
    }

    /// Restore engine + Document from raw snapshot bytes (content.loro).
    pub fn import_snapshot(bytes: &[u8]) -> Result<(Self, Document)> {
        let doc  = LoroDoc::from_snapshot(bytes)
            .context("loro: from_snapshot")?;
        let undo = UndoManager::new(&doc);
        let engine = CrdtEngine { doc, undo };
        let document = engine.read_document()?;
        Ok((engine, document))
    }

    /// Drop all undo/redo history. Call after loading a file so the user
    /// cannot undo back to before the load.
    pub fn clear_history(&mut self) {
        self.undo = UndoManager::new(&self.doc);
    }

    // ── Private ──────────────────────────────────────────────────────────────

    fn read_document(&self) -> Result<Document> {
        let text = self.doc.get_text(DOC_CONTAINER);
        let json = text.to_string();
        serde_json::from_str(&json)
            .context("loro: deserialise document from snapshot")
    }
}

impl Default for CrdtEngine {
    fn default() -> Self { Self::new() }
}
