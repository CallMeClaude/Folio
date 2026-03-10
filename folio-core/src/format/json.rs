/// JSON schema helpers for document.json inside the .folio bundle.
/// The Document struct derives serde, so this is mostly thin wrappers.
use anyhow::Result;
use crate::document::Document;

/// Serialize a Document to a pretty-printed JSON string.
pub fn to_json(doc: &Document) -> Result<String> {
    Ok(serde_json::to_string_pretty(doc)?)
}

/// Deserialize a Document from a JSON string.
pub fn from_json(s: &str) -> Result<Document> {
    Ok(serde_json::from_str(s)?)
}
