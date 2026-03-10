use anyhow::Result;
use crate::document::{Document, BlockKind, InlineAttr};

/// Export document as Markdown.
pub fn export_md(doc: &Document) -> Result<String> {
    let mut out = String::new();

    if !doc.title.is_empty() {
        out.push_str(&format!("# {}\n", doc.title));
    }
    if !doc.subtitle.is_empty() {
        out.push_str(&format!("*{}*\n", doc.subtitle));
    }
    if !doc.title.is_empty() || !doc.subtitle.is_empty() {
        out.push('\n');
    }

    for block in &doc.blocks {
        let inline = render_inline_md(&block.content);
        let line = match &block.kind {
            BlockKind::Title      => format!("# {}", inline),
            BlockKind::Heading1   => format!("## {}", inline),
            BlockKind::Heading2   => format!("### {}", inline),
            BlockKind::Caption    => format!("#### {}", inline),
            BlockKind::Quote      => format!("> {}", inline),
            BlockKind::Code       => format!("```\n{}\n```", inline),
            BlockKind::BulletItem => format!("- {}", inline),
            BlockKind::OrderedItem { index } => format!("{}. {}", index, inline),
            BlockKind::CheckItem { checked } =>
                format!("- [{}] {}", if *checked { "x" } else { " " }, inline),
            BlockKind::Divider    => "---".to_string(),
            BlockKind::Image { alt, .. } => format!("![{}]()", alt),
            BlockKind::Paragraph  => inline,
        };
        if !line.is_empty() {
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

fn render_inline_md(runs: &[crate::document::InlineRun]) -> String {
    runs.iter().map(|r| {
        let mut s = r.text.clone();
        if r.attrs.contains(&InlineAttr::Bold)          { s = format!("**{}**", s); }
        if r.attrs.contains(&InlineAttr::Italic)        { s = format!("*{}*", s); }
        if r.attrs.contains(&InlineAttr::Strikethrough) { s = format!("~~{}~~", s); }
        if let Some(InlineAttr::Link(url)) = r.attrs.iter().find(|a| matches!(a, InlineAttr::Link(_))) {
            s = format!("[{}]({})", s, url);
        }
        s
    }).collect()
}
