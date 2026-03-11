use folio_core::{Document, Block, BlockKind, PageSettings, InlineRun};
use folio_core::stats::compute;
use folio_core::export::{export_txt, export_md, export_html};

fn doc_with_prose() -> Document {
    let mut d = Document::new("Stats Test", PageSettings::default());
    d.blocks.clear();

    let mut h = Block::new(BlockKind::Heading1);
    h.content = vec![InlineRun::plain("Introduction")];
    d.blocks.push(h);

    d.blocks.push(Block::paragraph(
        "The quick brown fox jumps over the lazy dog. \
         Pack my box with five dozen liquor jugs. \
         How vexingly quick daft zebras jump.",
    ));
    d.blocks.push(Block::paragraph(
        "A second paragraph with more words. \
         This one has a few extra sentences to pad the count out.",
    ));
    d.blocks.push(Block::new(BlockKind::Divider));
    d
}

#[test]
fn word_count_is_nonzero() {
    let d     = doc_with_prose();
    let stats = compute(&d);
    assert!(stats.words > 0, "expected nonzero word count");
}

#[test]
fn word_count_reasonable() {
    let d     = doc_with_prose();
    let stats = compute(&d);
    // The two paragraphs have about 46 words combined.
    assert!(stats.words >= 40 && stats.words <= 60,
        "word count {} out of expected range 40-60", stats.words);
}

#[test]
fn sentence_count_nonzero() {
    let d     = doc_with_prose();
    let stats = compute(&d);
    assert!(stats.sentences >= 3);
}

#[test]
fn paragraph_count_correct() {
    let d     = doc_with_prose();
    let stats = compute(&d);
    assert_eq!(stats.paragraphs, 2);
}

// ─── export smoke tests ───────────────────────────────────────────────────────

#[test]
fn export_txt_contains_text() {
    let d   = doc_with_prose();
    let txt = export_txt(&d).unwrap();
    assert!(txt.contains("quick brown fox"));
    assert!(txt.contains("second paragraph"));
}

#[test]
fn export_md_has_heading_prefix() {
    let d  = doc_with_prose();
    let md = export_md(&d).unwrap();
    assert!(md.contains("## Introduction"));
}

#[test]
fn export_html_is_valid_shell() {
    let d    = doc_with_prose();
    let html = export_html(&d).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<h2>Introduction</h2>"));
    assert!(html.contains("quick brown fox"));
}
