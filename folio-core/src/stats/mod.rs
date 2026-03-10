/// Document statistics — word count, readability, session tracking.
use crate::document::Document;

#[derive(Debug, Clone, Default)]
pub struct DocStats {
    pub words:       usize,
    pub characters:  usize,
    pub sentences:   usize,
    pub paragraphs:  usize,
    pub read_minutes: usize,
    pub readability: ReadabilityLevel,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ReadabilityLevel {
    Easy,
    Fair,
    #[default]
    Hard,
    Dense,
}

impl std::fmt::Display for ReadabilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadabilityLevel::Easy  => write!(f, "Easy"),
            ReadabilityLevel::Fair  => write!(f, "Fair"),
            ReadabilityLevel::Hard  => write!(f, "Hard"),
            ReadabilityLevel::Dense => write!(f, "Dense"),
        }
    }
}

/// Compute stats from a Document.
pub fn compute(doc: &Document) -> DocStats {
    let full_text: String = doc.blocks.iter()
        .map(|b| b.plain_text())
        .collect::<Vec<_>>()
        .join(" ");

    let words      = count_words(&full_text);
    let characters = full_text.len();
    let sentences  = count_sentences(&full_text);
    let paragraphs = doc.blocks.iter()
        .filter(|b| matches!(b.kind, crate::document::BlockKind::Paragraph))
        .count();
    let read_minutes = (words / 238).max(1);

    let avg_wps = if sentences > 0 { words as f64 / sentences as f64 } else { 10.0 };
    let fk = (206.0 - 1.02 * avg_wps).clamp(0.0, 100.0) as u32;
    let readability = match fk {
        71..=100 => ReadabilityLevel::Easy,
        51..=70  => ReadabilityLevel::Fair,
        31..=50  => ReadabilityLevel::Hard,
        _        => ReadabilityLevel::Dense,
    };

    DocStats { words, characters, sentences, paragraphs, read_minutes, readability }
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().filter(|w| !w.is_empty()).count()
}

fn count_sentences(text: &str) -> usize {
    text.split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| s.trim().len() > 2)
        .count()
        .max(1)
}
