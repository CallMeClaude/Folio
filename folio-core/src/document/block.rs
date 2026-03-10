use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Inline attributes ────────────────────────────────────────────────────────

/// A single inline formatting attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineAttr {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Superscript,
    Subscript,
    /// Foreground colour as 0xRRGGBB.
    TextColor(u32),
    /// Background highlight colour as 0xRRGGBB.
    Highlight(u32),
    /// Hyperlink URL.
    Link(String),
}

// ─── Inline run ───────────────────────────────────────────────────────────────

/// A run of text sharing the same set of inline attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineRun {
    pub text:  String,
    pub attrs: Vec<InlineAttr>,
}

impl InlineRun {
    pub fn plain(text: impl Into<String>) -> Self {
        InlineRun { text: text.into(), attrs: vec![] }
    }

    pub fn has_attr(&self, attr: &InlineAttr) -> bool {
        self.attrs.contains(attr)
    }
}

// ─── Block kinds ─────────────────────────────────────────────────────────────

/// Every structural element type the document can contain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    /// Normal body paragraph.
    Paragraph,
    /// Document title (outermost heading).
    Title,
    /// Heading level 1 (h2 in HTML equivalent).
    Heading1,
    /// Heading level 2 (h3 in HTML equivalent).
    Heading2,
    /// Small caption text.
    Caption,
    /// Block quote.
    Quote,
    /// Preformatted code block.
    Code,
    /// Unordered bullet list item.
    BulletItem,
    /// Ordered list item. `index` is the display number.
    OrderedItem { index: u32 },
    /// Checklist item.
    CheckItem { checked: bool },
    /// Horizontal rule / divider.
    Divider,
    /// Embedded image asset.
    Image { asset_id: Uuid, alt: String },
}

// ─── Block ───────────────────────────────────────────────────────────────────

/// A single logical block in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// Stable unique ID — used by the CRDT and for outline navigation.
    pub id:      Uuid,
    pub kind:    BlockKind,
    /// Inline content. Empty for Divider and Image blocks.
    pub content: Vec<InlineRun>,
    /// Per-block layout overrides (indent level, space before/after, etc.)
    pub layout:  BlockLayout,
}

impl Block {
    pub fn new(kind: BlockKind) -> Self {
        Block {
            id:      Uuid::new_v4(),
            kind,
            content: vec![],
            layout:  BlockLayout::default(),
        }
    }

    pub fn paragraph(text: impl Into<String>) -> Self {
        let mut b = Block::new(BlockKind::Paragraph);
        b.content = vec![InlineRun::plain(text)];
        b
    }

    /// Flat text content, ignoring inline attributes.
    pub fn plain_text(&self) -> String {
        self.content.iter().map(|r| r.text.as_str()).collect()
    }
}

// ─── Block layout overrides ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockLayout {
    /// Left indent in mm (0 = no indent).
    pub indent_mm:      f64,
    /// Extra space before block in mm.
    pub space_before_mm: f64,
    /// Extra space after block in mm.
    pub space_after_mm:  f64,
    /// Text alignment.
    pub alignment: Alignment,
}

impl Default for BlockLayout {
    fn default() -> Self {
        BlockLayout {
            indent_mm:       0.0,
            space_before_mm: 0.0,
            space_after_mm:  0.0,
            alignment:       Alignment::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Alignment { Left, Center, Right, Justified }
