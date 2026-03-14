use folio_core::{
    Document, Block, BlockKind, PageSettings,
    InlineRun, InlineAttr, CrdtEngine,
};
use folio_core::format::{save_folio, load_folio};
use folio_core::format::json::{to_json, from_json};

fn full_doc() -> Document {
    let mut d = Document::new("Round-trip Test", PageSettings::default());
    d.subtitle = "A subtitle".to_string();
    d.blocks.clear();

    d.blocks.push(Block::paragraph("Normal paragraph."));

    let mut h1 = Block::new(BlockKind::Heading1);
    h1.content = vec![InlineRun::plain("Section One")];
    d.blocks.push(h1);

    d.blocks.push(Block::new(BlockKind::Divider));

    let mut quote = Block::new(BlockKind::Quote);
    quote.content = vec![
        InlineRun { text: "Bold".to_string(), attrs: vec![InlineAttr::Bold] },
        InlineRun::plain(" normal"),
    ];
    d.blocks.push(quote);

    let mut check = Block::new(BlockKind::CheckItem { checked: true });
    check.content = vec![InlineRun::plain("Done item")];
    d.blocks.push(check);

    d
}

#[test]
fn json_roundtrip() {
    let original = full_doc();
    let json     = to_json(&original).unwrap();
    let restored = from_json(&json).unwrap();

    assert_eq!(original.id,       restored.id);
    assert_eq!(original.title,    restored.title);
    assert_eq!(original.subtitle, restored.subtitle);
    assert_eq!(original.blocks.len(), restored.blocks.len());

    for (a, b) in original.blocks.iter().zip(restored.blocks.iter()) {
        assert_eq!(a.id,      b.id);
        assert_eq!(a.kind,    b.kind);
        assert_eq!(a.content, b.content);
    }
}

#[test]
fn folio_file_roundtrip() {
    let original = full_doc();
    let dir      = tempfile::tempdir().unwrap();
    let path     = dir.path().join("test.folio");

    // Build engine + initial checkpoint so content.loro is valid.
    let mut engine = CrdtEngine::new();
    engine.checkpoint(&original).unwrap();

    save_folio(&path, &original, &engine, &[]).unwrap();
    assert!(path.exists());

    let (mut restored_engine, restored, assets) = load_folio(&path).unwrap();
    assert!(assets.is_empty());
    assert_eq!(original.id,    restored.id);
    assert_eq!(original.title, restored.title);
    assert_eq!(original.blocks.len(), restored.blocks.len());

    for (a, b) in original.blocks.iter().zip(restored.blocks.iter()) {
        assert_eq!(a.id,      b.id);
        assert_eq!(a.kind,    b.kind);
        assert_eq!(a.content, b.content);
    }

    // Verify undo/redo history was cleared on load.
    assert!(!restored_engine.can_undo());
}
