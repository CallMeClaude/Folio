use anyhow::Result;
use crate::document::{Document, BlockKind};

/// Export document as plain text.
pub fn export_txt(doc: &Document) -> Result<String> {
    let mut out = String::new();
    if !doc.title.is_empty() {
        out.push_str(&doc.title);
        out.push('\n');
    }
    if !doc.subtitle.is_empty() {
        out.push_str(&doc.subtitle);
        out.push('\n');
    }
    if !doc.title.is_empty() || !doc.subtitle.is_empty() {
        out.push('\n');
    }
    for block in &doc.blocks {
        let text = block.plain_text();
        match &block.kind {
            BlockKind::Divider => out.push_str("---\n"),
            BlockKind::Image { alt, .. } => {
                out.push_str(&format!("[Image: {}]\n", alt));
            }
            _ => {
                if !text.is_empty() {
                    out.push_str(&text);
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}
