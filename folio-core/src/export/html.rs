use anyhow::Result;
use crate::document::{Document, BlockKind, InlineAttr};

/// Export document as a self-contained HTML file.
pub fn export_html(doc: &Document) -> Result<String> {
    let mut body = String::new();

    if !doc.title.is_empty() {
        body.push_str(&format!("<h1>{}</h1>\n", esc(&doc.title)));
    }
    if !doc.subtitle.is_empty() {
        body.push_str(&format!("<p class=\"subtitle\">{}</p>\n", esc(&doc.subtitle)));
    }

    for block in &doc.blocks {
        let inline = render_inline_html(&block.content);
        let tag = match &block.kind {
            BlockKind::Title    => format!("<h1>{}</h1>", inline),
            BlockKind::Heading1 => format!("<h2>{}</h2>", inline),
            BlockKind::Heading2 => format!("<h3>{}</h3>", inline),
            BlockKind::Caption  => format!("<p class=\"caption\">{}</p>", inline),
            BlockKind::Quote    => format!("<blockquote>{}</blockquote>", inline),
            BlockKind::Code     => format!("<pre><code>{}</code></pre>", esc(&block.plain_text())),
            BlockKind::BulletItem => format!("<li>{}</li>", inline),
            BlockKind::OrderedItem { .. } => format!("<li>{}</li>", inline),
            BlockKind::CheckItem { checked } =>
                format!("<li><input type=\"checkbox\" {}disabled> {}</li>",
                    if *checked { "checked " } else { "" }, inline),
            BlockKind::Divider  => "<hr>".to_string(),
            BlockKind::Image { alt, .. } =>
                format!("<figure><img alt=\"{}\"></figure>", esc(alt)),
            BlockKind::Paragraph => format!("<p>{}</p>", inline),
        };
        body.push_str(&tag);
        body.push('\n');
    }

    Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>{title}</title>
<style>
body{{font-family:Georgia,serif;max-width:700px;margin:80px auto;
line-height:1.82;font-size:17px;color:#1c1c1e;padding:0 24px}}
h1,h2,h3{{letter-spacing:-.02em;margin:1.3em 0 .4em}}
blockquote{{border-left:2.5px solid #1b6ee4;padding-left:22px;
font-style:italic;color:#58585f}}
pre{{background:#f5f4f2;padding:14px 18px;border-radius:8px;
font-family:monospace;font-size:13px}}
hr{{border:none;border-top:1px solid #ddd;margin:1.8em 0}}
.subtitle{{font-style:italic;color:#888}}
.caption{{font-size:.80em;color:#888}}
</style>
</head>
<body>
{body}</body>
</html>"#,
        title = esc(&doc.title),
        body  = body))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn render_inline_html(runs: &[crate::document::InlineRun]) -> String {
    runs.iter().map(|r| {
        let mut s = esc(&r.text);
        if r.attrs.contains(&InlineAttr::Bold)          { s = format!("<strong>{}</strong>", s); }
        if r.attrs.contains(&InlineAttr::Italic)        { s = format!("<em>{}</em>", s); }
        if r.attrs.contains(&InlineAttr::Underline)     { s = format!("<u>{}</u>", s); }
        if r.attrs.contains(&InlineAttr::Strikethrough) { s = format!("<s>{}</s>", s); }
        if r.attrs.contains(&InlineAttr::Superscript)   { s = format!("<sup>{}</sup>", s); }
        if r.attrs.contains(&InlineAttr::Subscript)     { s = format!("<sub>{}</sub>", s); }
        if let Some(InlineAttr::TextColor(c)) = r.attrs.iter().find(|a| matches!(a, InlineAttr::TextColor(_))) {
            s = format!("<span style=\"color:#{:06X}\">{}</span>", c, s);
        }
        if let Some(InlineAttr::Highlight(c)) = r.attrs.iter().find(|a| matches!(a, InlineAttr::Highlight(_))) {
            s = format!("<span style=\"background:#{:06X}\">{}</span>", c, s);
        }
        if let Some(InlineAttr::Link(url)) = r.attrs.iter().find(|a| matches!(a, InlineAttr::Link(_))) {
            s = format!("<a href=\"{}\">{}</a>", url, s);
        }
        s
    }).collect()
}
