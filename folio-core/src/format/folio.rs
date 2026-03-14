//! .folio bundle format — a ZIP archive containing:
//!
//!   document.json   — metadata only (title, subtitle, paper size, margins,
//!                     orientation, created/modified timestamps, typography).
//!                     The block tree lives in content.loro.
//!   content.loro    — loro binary snapshot (block tree + full edit history)
//!   assets/         — embedded images as original files, keyed by UUID
//!
//! This mirrors how .pages works: the ZIP *is* the document.

use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Context, Result};
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
use crate::crdt::CrdtEngine;
use crate::document::Document;
use crate::format::json;

const META_JSON:   &str = "document.json";
const LORO_BLOB:   &str = "content.loro";
const ASSETS_DIR:  &str = "assets/";

// ── Save ─────────────────────────────────────────────────────────────────────

/// Write a .folio bundle.
///
/// `engine` is used to export the loro snapshot (content.loro).
/// `assets` is a list of (uuid, extension, raw_bytes) for any embedded images.
pub fn save_folio(
    path:   &Path,
    doc:    &Document,
    engine: &CrdtEngine,
    assets: &[(Uuid, String, Vec<u8>)],
) -> Result<()> {
    let file = std::fs::File::create(path)
        .with_context(|| format!("cannot create {}", path.display()))?;
    let mut zip  = ZipWriter::new(file);
    let     opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. Metadata JSON (no block tree)
    let meta_json = json::to_metadata_json(doc)?;
    zip.start_file(META_JSON, opts)?;
    zip.write_all(meta_json.as_bytes())?;

    // 2. Loro snapshot (contains the full block tree + history)
    let loro_bytes = engine.export_snapshot()?;
    zip.start_file(LORO_BLOB, opts)?;
    zip.write_all(&loro_bytes)?;

    // 3. Assets
    for (id, ext, bytes) in assets {
        let name = format!("{}{}.{}", ASSETS_DIR, id, ext);
        zip.start_file(&name, opts)?;
        zip.write_all(bytes)?;
    }

    zip.finish()?;
    Ok(())
}

// ── Load ─────────────────────────────────────────────────────────────────────

/// Read a .folio bundle.
///
/// Returns `(engine, document, asset_map)` where:
/// - `engine`    is restored from the loro snapshot
/// - `document`  is the full Document rebuilt from the loro state
/// - `asset_map` maps Uuid → raw image bytes
pub fn load_folio(
    path: &Path,
) -> Result<(CrdtEngine, Document, std::collections::HashMap<Uuid, Vec<u8>>)> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let mut zip = ZipArchive::new(file)?;

    // 1. Read loro snapshot
    let loro_bytes = {
        let mut entry = zip.by_name(LORO_BLOB)
            .context("content.loro missing from .folio bundle")?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        buf
    };

    // 2. Restore engine + document from loro snapshot
    let (mut engine, document) = CrdtEngine::import_snapshot(&loro_bytes)?;
    engine.clear_history();

    // 3. Assets
    let mut assets = std::collections::HashMap::new();
    let names: Vec<String> = zip.file_names().map(|s| s.to_string()).collect();
    for name in names {
        if name.starts_with(ASSETS_DIR) && name != ASSETS_DIR {
            let stem = Path::new(&name)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            if let Some(id) = stem {
                let mut entry = zip.by_name(&name)?;
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                assets.insert(id, buf);
            }
        }
    }

    Ok((engine, document, assets))
}

// ── Quick metadata read (for launch screen listing) ──────────────────────────

/// Read only the document metadata from a .folio bundle — much cheaper than
/// loading the full loro snapshot. Used by the launch screen to list files.
pub fn read_folio_metadata(path: &Path) -> Result<crate::format::json::DocumentMetadata> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let mut zip = ZipArchive::new(file)?;
    let mut entry = zip.by_name(META_JSON)
        .context("document.json missing from .folio bundle")?;
    let mut s = String::new();
    entry.read_to_string(&mut s)?;
    Ok(serde_json::from_str(&s)?)
}
