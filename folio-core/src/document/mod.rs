pub mod block;
pub mod page;

pub use block::{Block, BlockKind, BlockLayout, Alignment, InlineRun, InlineAttr};
pub use page::{PaperSize, Margins, PageSettings, Orientation};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// The top-level document, as stored in document.json inside the .folio bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document ID (also used as the CRDT actor ID seed).
    pub id:       Uuid,
    pub title:    String,
    pub subtitle: String,
    pub created:  DateTime<Utc>,
    pub modified: DateTime<Utc>,
    /// Page layout settings (paper size, margins, orientation).
    pub page:     PageSettings,
    /// Ordered list of blocks — the document body.
    pub blocks:   Vec<Block>,
    /// Document-level typography settings.
    pub typography: TypographySettings,
}

impl Document {
    pub fn new(title: impl Into<String>, page: PageSettings) -> Self {
        let now = Utc::now();
        Document {
            id:         Uuid::new_v4(),
            title:      title.into(),
            subtitle:   String::new(),
            created:    now,
            modified:   now,
            page,
            blocks:     vec![Block::paragraph("")],
            typography: TypographySettings::default(),
        }
    }
}

/// Document-wide typography defaults (can be overridden per-block).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographySettings {
    /// Font family name as a CSS-like string, e.g. "Lora".
    pub font_family: String,
    /// Base font size in points.
    pub font_size_pt: f64,
    /// Line height multiplier.
    pub line_height:  f64,
    /// Letter spacing in em units.
    pub tracking_em:  f64,
}

impl Default for TypographySettings {
    fn default() -> Self {
        TypographySettings {
            font_family:  "Lora".to_string(),
            font_size_pt: 12.0,
            line_height:  1.82,
            tracking_em:  0.0,
        }
    }
}
