//! JSON helpers for document.json inside the .folio bundle.
//!
//! document.json stores only document *metadata* — title, subtitle, paper
//! size, margins, orientation, timestamps, and typography settings.
//! The block tree lives in content.loro (the loro snapshot).
//!
//! Full round-trip serialisation of `Document` (including blocks) is still
//! available via `to_json` / `from_json` for tests and export pipelines.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::document::{Document, PageSettings, TypographySettings};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ── Full round-trip (tests / exports) ────────────────────────────────────────

/// Serialise a full Document (metadata + block tree) to pretty JSON.
pub fn to_json(doc: &Document) -> Result<String> {
    Ok(serde_json::to_string_pretty(doc)?)
}

/// Deserialise a full Document from JSON.
pub fn from_json(s: &str) -> Result<Document> {
    Ok(serde_json::from_str(s)?)
}

// ── Metadata-only (document.json inside .folio) ───────────────────────────────

/// The subset of Document fields stored in document.json.
/// Blocks are deliberately excluded — they live in content.loro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id:         Uuid,
    pub title:      String,
    pub subtitle:   String,
    pub created:    DateTime<Utc>,
    pub modified:   DateTime<Utc>,
    pub page:       PageSettings,
    pub typography: TypographySettings,
}

impl From<&Document> for DocumentMetadata {
    fn from(doc: &Document) -> Self {
        DocumentMetadata {
            id:         doc.id,
            title:      doc.title.clone(),
            subtitle:   doc.subtitle.clone(),
            created:    doc.created,
            modified:   doc.modified,
            page:       doc.page.clone(),
            typography: doc.typography.clone(),
        }
    }
}

/// Serialise metadata-only JSON for document.json inside the .folio bundle.
pub fn to_metadata_json(doc: &Document) -> Result<String> {
    let meta = DocumentMetadata::from(doc);
    Ok(serde_json::to_string_pretty(&meta)?)
}
