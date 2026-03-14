use folio_core::{Document, Block, BlockKind, PageSettings, InlineRun};
use folio_core::search::{SearchQuery, find_all, find_next, find_prev, replace_all};
use folio_core::DocPosition;

fn doc_with_text() -> Document {
    let mut d = Document::new("Test", PageSettings::default());
    d.blocks.clear();
    let mut b0 = Block::new(BlockKind::Paragraph);
    b0.content = vec![InlineRun::plain("Hello world hello")];
    let mut b1 = Block::new(BlockKind::Paragraph);
    b1.content = vec![InlineRun::plain("HELLO again")];
    d.blocks.push(b0);
    d.blocks.push(b1);
    d
}

#[test]
fn literal_case_insensitive_finds_all() {
    let doc = doc_with_text();
    let q = SearchQuery { pattern: "hello".into(), case_sensitive: false, use_regex: false };
    let all = find_all(&doc, &q).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn literal_case_sensitive_finds_fewer() {
    let doc = doc_with_text();
    let q = SearchQuery { pattern: "hello".into(), case_sensitive: true, use_regex: false };
    let all = find_all(&doc, &q).unwrap();
    // Only lowercase "hello" matches — "Hello" and "HELLO" do not.
    assert_eq!(all.len(), 1);
}

#[test]
fn regex_find() {
    let doc = doc_with_text();
    let q = SearchQuery { pattern: r"hel+o".into(), case_sensitive: false, use_regex: true };
    let all = find_all(&doc, &q).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn find_next_wraps() {
    let doc = doc_with_text();
    let q = SearchQuery { pattern: "hello".into(), case_sensitive: false, use_regex: false };
    // Start after last match — should wrap to first.
    let after = DocPosition::new(1, 100);
    let (m, wrapped) = find_next(&doc, &q, after).unwrap().unwrap();
    assert!(wrapped);
    assert_eq!(m.block_idx, 0);
    assert_eq!(m.byte_start, 0);
}

#[test]
fn replace_all_count() {
    let mut doc = doc_with_text();
    let q = SearchQuery { pattern: "hello".into(), case_sensitive: false, use_regex: false };
    let n = replace_all(&mut doc, &q, "hi").unwrap();
    assert_eq!(n, 3);
}
