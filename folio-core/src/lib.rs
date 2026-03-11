// folio-core — document model, editing ops, CRDT, serialization, export.
// No GTK dependency. All logic testable in isolation.

pub mod document;
pub mod crdt;
pub mod format;
pub mod export;
pub mod stats;

pub use document::{
    Document, Block, BlockKind, BlockLayout, Alignment,
    InlineRun, InlineAttr,
    PaperSize, Margins, PageSettings, Orientation,
    TypographySettings,
    DocPosition, DocRange,
    EditError,
};
pub use crdt::CrdtEngine;
