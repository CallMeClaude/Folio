//! Font management — Phase 9.
//!
//! Bundled fonts are shipped in `data/fonts/` and copied to the app's
//! font cache directory (`~/.local/share/folio/fonts/`) on first run.
//! User-installed fonts (drag .ttf or CDN URL) also land there.
//! Fontconfig picks them up automatically from that directory.

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

// ── Font cache directory ──────────────────────────────────────────────────────

/// `~/.local/share/folio/fonts/`
pub fn font_cache_dir() -> PathBuf {
    glib::user_data_dir().join("folio").join("fonts")
}

/// Ensure the font cache dir exists and return it.
pub fn ensure_font_dir() -> Result<PathBuf> {
    let dir = font_cache_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("cannot create font dir {}", dir.display()))?;
    Ok(dir)
}

// ── Bundled fonts ─────────────────────────────────────────────────────────────

/// Returns the list of bundled font file names expected in `data/fonts/`.
pub const BUNDLED_FONTS: &[&str] = &[
    "Lora-Regular.ttf",
    "Lora-Bold.ttf",
    "Lora-Italic.ttf",
    "Newsreader-Regular.ttf",
    "Newsreader-Italic.ttf",
    "Fraunces-Regular.ttf",
    "Inter-Regular.ttf",
    "Inter-Bold.ttf",
    "IBMPlexMono-Regular.ttf",
];

/// Copy bundled fonts from `data_dir/fonts/` to the user font cache.
/// Skips fonts that are already present. Safe to call on every launch.
pub fn install_bundled_fonts(data_dir: &Path) -> Result<()> {
    let src_dir  = data_dir.join("fonts");
    let dest_dir = ensure_font_dir()?;
    if !src_dir.exists() { return Ok(()); }
    for name in BUNDLED_FONTS {
        let src  = src_dir.join(name);
        let dest = dest_dir.join(name);
        if src.exists() && !dest.exists() {
            std::fs::copy(&src, &dest)
                .with_context(|| format!("failed to copy font {}", name))?;
        }
    }
    Ok(())
}

// ── User font install ─────────────────────────────────────────────────────────

/// Install a .ttf/.otf file from an arbitrary path into the font cache.
/// Returns the installed path.
pub fn install_font_file(src: &Path) -> Result<PathBuf> {
    let dest_dir = ensure_font_dir()?;
    let name     = src.file_name()
        .context("font path has no filename")?;
    let dest = dest_dir.join(name);
    std::fs::copy(src, &dest)
        .with_context(|| format!("failed to install font {}", src.display()))?;
    Ok(dest)
}

// ── CDN font download ─────────────────────────────────────────────────────────

/// Download a font from a URL and install it in the font cache.
/// Uses blocking HTTP via `reqwest` on a background thread.
/// Returns the installed path.
pub async fn download_font(url: &str) -> Result<PathBuf> {
    let dest_dir = ensure_font_dir()?;

    // Derive filename from the URL's last path segment.
    let raw_name = url.split('/').last().unwrap_or("font.ttf");
    let name     = raw_name.split('?').next().unwrap_or(raw_name);
    if !name.ends_with(".ttf") && !name.ends_with(".otf") && !name.ends_with(".woff2") {
        anyhow::bail!("URL does not appear to be a font file: {}", url);
    }
    let dest = dest_dir.join(name);

    let bytes = reqwest::get(url).await
        .with_context(|| format!("HTTP GET failed for {}", url))?
        .bytes().await
        .context("failed to read font bytes")?;

    std::fs::write(&dest, &bytes)
        .with_context(|| format!("failed to write font to {}", dest.display()))?;

    Ok(dest)
}

// ── List installed fonts ──────────────────────────────────────────────────────

/// Return the names (without extension) of all fonts in the cache dir.
pub fn list_installed_fonts() -> Vec<String> {
    let dir = font_cache_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else { return vec![]; };
    rd.flatten()
        .filter_map(|e| {
            let p = e.path();
            let ext = p.extension()?.to_str()?;
            if matches!(ext, "ttf" | "otf" | "woff2") {
                p.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}
