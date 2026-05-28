use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WritingStats {
    /// Consecutive days with writing activity ending today (0 if today and
    /// yesterday both lack activity).
    pub current_streak: u32,
    pub longest_streak: u32,
    /// ISO date of the most recent activity (YYYY-MM-DD).
    pub last_writing_date: Option<String>,
}

/// Per-day counter of words written. Powers the 30-day sparkline in
/// Settings. Only positive deltas are accumulated — deletions don't
/// subtract — because the chart is about forward progress, not net
/// content size.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyWriting {
    /// ISO date `YYYY-MM-DD` (local time).
    pub date: String,
    pub words: u32,
    pub sessions: u32,
}

/// Strip HTML tags and return a rough word count. Used by the writing
/// stats pipeline to compute per-save deltas. Empty/whitespace input
/// returns 0.
pub fn count_words_in_html(html: &str) -> u32 {
    if html.trim().is_empty() {
        return 0;
    }
    let mut buf = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                buf.push(' ');
            }
            _ if !in_tag => buf.push(ch),
            _ => {}
        }
    }
    buf.split_whitespace().count() as u32
}

#[cfg(test)]
mod tests {
    use super::count_words_in_html;

    #[test]
    fn counts_words_ignoring_tags() {
        assert_eq!(
            count_words_in_html("<p>Hola <strong>mundo</strong> entero.</p>"),
            3
        );
    }

    #[test]
    fn empty_and_whitespace_are_zero() {
        assert_eq!(count_words_in_html(""), 0);
        assert_eq!(count_words_in_html("   \n\t "), 0);
        assert_eq!(count_words_in_html("<p></p>"), 0);
    }

    #[test]
    fn punctuation_does_not_split_words() {
        assert_eq!(count_words_in_html("<p>hola, mundo. ¿qué tal?</p>"), 4);
    }
}
