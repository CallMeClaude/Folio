use folio_core::{Document, Block, BlockKind, PageSettings, DocPosition, DocRange};

fn doc_with(text: &str) -> Document {
    let mut d = Document::new("Test", PageSettings::default());
    d.blocks.clear();
    d.blocks.push(Block::paragraph(text));
    d
}

#[test]
fn insert_into_empty_block() {
    let mut d = doc_with("");
    d.insert_text(DocPosition::new(0, 0), "hello").unwrap();
    assert_eq!(d.blocks[0].plain_text(), "hello");
}

#[test]
fn insert_at_start() {
    let mut d = doc_with("world");
    d.insert_text(DocPosition::new(0, 0), "hello ").unwrap();
    assert_eq!(d.blocks[0].plain_text(), "hello world");
}

#[test]
fn insert_at_end() {
    let mut d = doc_with("hello");
    let end = d.blocks[0].plain_text().len();
    d.insert_text(DocPosition::new(0, end), " world").unwrap();
    assert_eq!(d.blocks[0].plain_text(), "hello world");
}

#[test]
fn insert_in_middle() {
    let mut d = doc_with("helloworld");
    d.insert_text(DocPosition::new(0, 5), " ").unwrap();
    assert_eq!(d.blocks[0].plain_text(), "hello world");
}

#[test]
fn insert_bad_block_idx_errors() {
    let mut d = doc_with("hi");
    assert!(d.insert_text(DocPosition::new(99, 0), "x").is_err());
}

#[test]
fn delete_whole_word() {
    let mut d = doc_with("hello world");
    d.delete_range(DocRange::new(DocPosition::new(0, 5), DocPosition::new(0, 11))).unwrap();
    assert_eq!(d.blocks[0].plain_text(), "hello");
}

#[test]
fn delete_collapsed_is_noop() {
    let mut d = doc_with("hello");
    let pos = DocPosition::new(0, 2);
    d.delete_range(DocRange::new(pos, pos)).unwrap();
    assert_eq!(d.blocks[0].plain_text(), "hello");
}

#[test]
fn delete_across_blocks_merges() {
    let mut d = Document::new("T", PageSettings::default());
    d.blocks.clear();
    d.blocks.push(Block::paragraph("hello "));
    d.blocks.push(Block::paragraph("world"));
    d.delete_range(DocRange::new(DocPosition::new(0, 3), DocPosition::new(1, 2))).unwrap();
    assert_eq!(d.blocks.len(), 1);
    assert_eq!(d.blocks[0].plain_text(), "helrld");
}

// ─── split_block ──────────────────────────────────────────────────────────────

#[test]
fn split_at_middle_creates_two_blocks() {
    let mut d = doc_with("hello world");
    d.split_block(DocPosition::new(0, 5)).unwrap();
    assert_eq!(d.blocks.len(), 2);
    assert_eq!(d.blocks[0].plain_text(), "hello");
    assert_eq!(d.blocks[1].plain_text(), " world");
}

#[test]
fn split_at_end_creates_empty_second_block() {
    let mut d = doc_with("hello");
    let end = d.blocks[0].plain_text().len();
    d.split_block(DocPosition::new(0, end)).unwrap();
    assert_eq!(d.blocks.len(), 2);
    assert_eq!(d.blocks[0].plain_text(), "hello");
    assert_eq!(d.blocks[1].plain_text(), "");
}

#[test]
fn split_heading_produces_paragraph() {
    let mut d = Document::new("T", PageSettings::default());
    d.blocks.clear();
    d.blocks.push({
        let mut b = Block::new(BlockKind::Heading1);
        b.content = vec![folio_core::InlineRun::plain("My Heading")];
        b
    });
    d.split_block(DocPosition::new(0, 2)).unwrap();
    assert!(matches!(d.blocks[1].kind, BlockKind::Paragraph));
}

// ─── merge_blocks ─────────────────────────────────────────────────────────────

#[test]
fn merge_joins_content() {
    let mut d = Document::new("T", PageSettings::default());
    d.blocks.clear();
    d.blocks.push(Block::paragraph("hello"));
    d.blocks.push(Block::paragraph(" world"));
    d.merge_blocks(0).unwrap();
    assert_eq!(d.blocks.len(), 1);
    assert_eq!(d.blocks[0].plain_text(), "hello world");
}

#[test]
fn merge_at_last_block_errors() {
    let mut d = doc_with("only");
    assert!(d.merge_blocks(0).is_err());
}

// ─── inline attrs ─────────────────────────────────────────────────────────────

#[test]
fn apply_bold_to_range() {
    let mut d = doc_with("hello world");
    let range = DocRange::new(DocPosition::new(0, 0), DocPosition::new(0, 5));
    d.apply_inline_attr(range, folio_core::InlineAttr::Bold).unwrap();
    // First run should be bold
    assert!(d.blocks[0].content[0].attrs.contains(&folio_core::InlineAttr::Bold));
    // Second run (after split) should not be bold
    assert!(!d.blocks[0].content[1].attrs.contains(&folio_core::InlineAttr::Bold));
}

#[test]
fn remove_bold_from_range() {
    let mut d = doc_with("hello");
    let range = DocRange::new(DocPosition::new(0, 0), DocPosition::new(0, 5));
    d.apply_inline_attr(range, folio_core::InlineAttr::Bold).unwrap();
    d.remove_inline_attr(range, &folio_core::InlineAttr::Bold).unwrap();
    for run in &d.blocks[0].content {
        assert!(!run.attrs.contains(&folio_core::InlineAttr::Bold));
    }
}

#[test]
fn runs_normalise_after_remove() {
    let mut d = doc_with("hello");
    let range = DocRange::new(DocPosition::new(0, 0), DocPosition::new(0, 5));
    d.apply_inline_attr(range, folio_core::InlineAttr::Bold).unwrap();
    d.remove_inline_attr(range, &folio_core::InlineAttr::Bold).unwrap();
    // After apply+remove, adjacent plain runs should be merged back to one.
    assert_eq!(d.blocks[0].content.len(), 1);
    assert_eq!(d.blocks[0].content[0].text, "hello");
}
