use serde::{Deserialize, Serialize};

/// Physical paper dimensions in millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaperSize {
    A3,
    A4,
    A5,
    Letter,   // 215.9 × 279.4 mm
    Legal,    // 215.9 × 355.6 mm
    Tabloid,  // 279.4 × 431.8 mm
    Custom { width_mm: f64, height_mm: f64 },
}

impl PaperSize {
    /// Returns (width_mm, height_mm).
    pub fn dimensions(&self) -> (f64, f64) {
        match self {
            PaperSize::A3      => (297.0, 420.0),
            PaperSize::A4      => (210.0, 297.0),
            PaperSize::A5      => (148.0, 210.0),
            PaperSize::Letter  => (215.9, 279.4),
            PaperSize::Legal   => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::Custom { width_mm, height_mm } => (*width_mm, *height_mm),
        }
    }

    /// Width in points (1 pt = 1/72 inch = 0.3528 mm).
    pub fn width_pt(&self)  -> f64 { self.dimensions().0 / 0.3528 }
    /// Height in points.
    pub fn height_pt(&self) -> f64 { self.dimensions().1 / 0.3528 }
}

impl Default for PaperSize {
    fn default() -> Self { PaperSize::A4 }
}

/// Page margins in millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    pub top_mm:    f64,
    pub bottom_mm: f64,
    pub left_mm:   f64,
    pub right_mm:  f64,
}

impl Default for Margins {
    fn default() -> Self {
        Margins { top_mm: 25.4, bottom_mm: 25.4, left_mm: 25.4, right_mm: 25.4 }
    }
}
/// Page orientation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Per-document page settings stored inside the .folio bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSettings {
    pub paper_size:  PaperSize,
    pub margins:     Margins,
    pub orientation: Orientation,
}

impl Default for PageSettings {
    fn default() -> Self {
        PageSettings {
            paper_size:  PaperSize::default(),
            margins:     Margins::default(),
            orientation: Orientation::Portrait,
        }
    }
}
