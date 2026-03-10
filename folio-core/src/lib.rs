// folio-core — document model, CRDT, serialization, export
// No GTK dependency. All logic testable in isolation.

pub mod document;
pub mod crdt;
pub mod format;
pub mod export;
pub mod stats;

pub use document::{Document, Block, BlockKind, InlineRun, InlineAttr};
pub use document::page::{PaperSize, Margins, PageSettings};
