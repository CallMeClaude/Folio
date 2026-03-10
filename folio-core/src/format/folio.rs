/// .folio bundle format — a ZIP archive containing:
///   document.json   — the serialized Document
///   assets/         — embedded images (UUID filename, original extension)
///
/// This is structurally identical to how .pages works.
use std::io::{Read, Write, Seek};
use std::path::Path;
use anyhow::{Context, Result};
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
use crate::document::Document;
use crate::format::json;

const DOCUMENT_JSON: &str = "document.json";
const ASSETS_DIR:    &str = "assets/";

/// Write a Document to a .folio file at `path`.
/// Any asset blobs should be passed as `assets`: Vec<(Uuid, extension, bytes)>.
pub fn save_folio(
    path: &Path,
    doc:  &Document,
    assets: &[(Uuid, String, Vec<u8>)],
) -> Result<()> {
    let file = std::fs::File::create(path)
        .with_context(|| format!("Cannot create {}", path.display()))?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Write document.json
    let json_str = json::to_json(doc)?;
    zip.start_file(DOCUMENT_JSON, opts)?;
    zip.write_all(json_str.as_bytes())?;

    // Write each asset
    for (id, ext, bytes) in assets {
        let name = format!("{}{}.{}", ASSETS_DIR, id, ext);
        zip.start_file(&name, opts)?;
        zip.write_all(bytes)?;
    }

    zip.finish()?;
    Ok(())
}

/// Read a Document from a .folio file at `path`.
/// Returns (Document, asset_map) where asset_map maps Uuid → raw bytes.
pub fn load_folio(
    path: &Path,
) -> Result<(Document, std::collections::HashMap<Uuid, Vec<u8>>)> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Cannot open {}", path.display()))?;
    let mut zip = ZipArchive::new(file)?;
    // Read document.json
    let mut json_file = zip.by_name(DOCUMENT_JSON)
        .context("document.json missing from .folio bundle")?;
    let mut json_str = String::new();
    json_file.read_to_string(&mut json_str)?;
    drop(json_file);
    let doc = json::from_json(&json_str)?;

    // Read all assets
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

    Ok((doc, assets))
}
