//! Localized emoji names and keywords from Unicode CLDR, for picker search
//! and `:shortcode` completion in languages other than English.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::i18n::Locale;

/// One `emoji<TAB>name|keyword|keyword...` per line. Regenerate with
/// `scripts/update-emoji-keywords.py`.
const EMOJI_ES: &str = include_str!("../assets/i18n/emoji-es.tsv");

/// Localized names and keywords for one emoji.
pub struct EmojiWords {
    /// Display name, as CLDR spells it (`cara llorando de risa`).
    pub name: String,
    /// Folded name followed by folded keywords, `|`-separated.
    pub folded: String,
}

impl EmojiWords {
    /// Folded search terms, name first.
    pub fn terms(&self) -> impl Iterator<Item = &str> + Clone {
        self.folded.split('|')
    }

    /// Ranks a folded query like the English name: exact name, then word
    /// prefix or exact keyword, then keyword prefix, then anywhere.
    pub fn match_score(&self, query: &str) -> Option<u8> {
        let mut terms = self.terms();
        let name = terms.next().unwrap_or_default();
        if name == query {
            Some(2)
        } else if name
            .split([' ', '-', ':'])
            .any(|word| word.starts_with(query))
            || terms.clone().any(|term| term == query)
        {
            Some(3)
        } else if terms.clone().any(|term| term.starts_with(query)) {
            Some(4)
        } else if self.terms().any(|term| term.contains(query)) {
            Some(5)
        } else {
            None
        }
    }

    /// `:cara_llorando_de_risa:` from the localized name.
    pub fn shortcode(&self) -> String {
        let slug: Vec<&str> = self
            .terms()
            .next()
            .unwrap_or_default()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|part| !part.is_empty())
            .collect();
        format!(":{}:", slug.join("_"))
    }
}

/// Names and keywords for `emoji` in `locale`. English has none: callers
/// already search the `emojis` crate names and shortcodes.
pub fn lookup(locale: Locale, emoji: &str) -> Option<&'static EmojiWords> {
    static ES: OnceLock<HashMap<String, EmojiWords>> = OnceLock::new();
    let table = match locale {
        Locale::Spanish => ES.get_or_init(|| parse(EMOJI_ES)),
        _ => return None,
    };
    table.get(&emoji.replace('\u{FE0F}', ""))
}

/// Lowercases and strips Latin accents so search matches `corazon` to
/// `corazón` and `nino` to `niño`.
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

fn parse(tsv: &str) -> HashMap<String, EmojiWords> {
    tsv.lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(emoji, words)| {
            let name = words.split('|').next().unwrap_or_default().to_owned();
            let words = EmojiWords {
                name,
                folded: fold(words),
            };
            (emoji.replace('\u{FE0F}', ""), words)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_keywords_ignore_accents_and_variation_selectors() {
        let heart = lookup(Locale::Spanish, "❤\u{FE0F}").expect("heart");
        assert!(heart.folded.contains("corazon"), "{}", heart.folded);
        let flag = lookup(Locale::Spanish, "🇨🇱").expect("flag");
        assert!(flag.folded.contains("chile"), "{}", flag.folded);
        assert!(lookup(Locale::English, "❤\u{FE0F}").is_none());
    }

    #[test]
    fn spanish_shortcodes_come_from_the_cldr_name() {
        let joy = lookup(Locale::Spanish, "😂").expect("joy");
        assert_eq!(joy.shortcode(), ":cara_llorando_de_risa:");
        assert_eq!(joy.name, "cara llorando de risa");
        let chile = lookup(Locale::Spanish, "🇨🇱").expect("flag");
        assert_eq!(chile.shortcode(), ":bandera_chile:");
        let crab = lookup(Locale::Spanish, "🦀").expect("crab");
        assert_eq!(crab.match_score("cangrejo"), Some(2));
    }
}
