//! Fork additions to `i18n`: the interface locale for views that are not
//! handed the app, and translation of summaries stored in English.

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::i18n::{Locale, gettext};

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// Records the interface locale; the app calls this whenever it changes.
pub fn set_locale(locale: Locale) {
    let index = Locale::ALL.iter().position(|l| *l == locale).unwrap_or(0);
    CURRENT.store(index, Ordering::Relaxed);
}

/// The interface locale, for code that has no `App` at hand.
pub fn locale() -> Locale {
    Locale::ALL[CURRENT.load(Ordering::Relaxed).min(Locale::ALL.len() - 1)]
}

/// Translates English text that upstream keeps in constants and tables,
/// where a `gettext` call cannot go. Unknown text comes back unchanged.
pub fn tr(english: &str) -> Cow<'static, str> {
    let locale = locale();
    match english {
        // Emoji picker
        "Frequently Used" => gettext(locale, "Frequently Used"),
        "Recent" => gettext(locale, "Recent"),
        "Nothing matches" => gettext(locale, "Nothing matches"),
        "Smileys & Emotion" => gettext(locale, "Smileys & Emotion"),
        "People & Body" => gettext(locale, "People & Body"),
        "Animals & Nature" => gettext(locale, "Animals & Nature"),
        "Food & Drink" => gettext(locale, "Food & Drink"),
        "Travel & Places" => gettext(locale, "Travel & Places"),
        "Activities" => gettext(locale, "Activities"),
        "Objects" => gettext(locale, "Objects"),
        "Symbols" => gettext(locale, "Symbols"),
        "Flags" => gettext(locale, "Flags"),
        "Emoji" => gettext(locale, "Emoji"),
        "Stickers" => gettext(locale, "Stickers"),
        // Keyboard shortcuts
        "Search chats" => gettext(locale, "Search chats"),
        "Search the open chat (Enter for the next match)" => {
            gettext(locale, "Search the open chat (Enter for the next match)")
        }
        "Focus the message input" => gettext(locale, "Focus the message input"),
        "Previous / next chat" => gettext(locale, "Previous / next chat"),
        "Edit the previous message (when the input is empty)" => gettext(
            locale,
            "Edit the previous message (when the input is empty)",
        ),
        "Send (Shift+Enter for a new line)" => gettext(locale, "Send (Shift+Enter for a new line)"),
        "Dismiss the current action, return from search, or close the chat" => gettext(
            locale,
            "Dismiss the current action, return from search, or close the chat",
        ),
        "New chat or message yourself" => gettext(locale, "New chat or message yourself"),
        "Paste text, or stage a picture from the clipboard" => {
            gettext(locale, "Paste text, or stage a picture from the clipboard")
        }
        "Show or hide the chat list" => gettext(locale, "Show or hide the chat list"),
        "Jump to the newest message" => gettext(locale, "Jump to the newest message"),
        "Settings" => gettext(locale, "Settings"),
        "Zoom in / out" => gettext(locale, "Zoom in / out"),
        "Reset zoom" => gettext(locale, "Reset zoom"),
        "Keyboard shortcuts (? when not typing)" => {
            gettext(locale, "Keyboard shortcuts (? when not typing)")
        }
        "Close the window (ZapFast remains in the tray)" => {
            gettext(locale, "Close the window (ZapFast remains in the tray)")
        }
        "Quit" => gettext(locale, "Quit"),
        // Poll validation
        "Enter a question of up to 255 characters." => {
            gettext(locale, "Enter a question of up to 255 characters.")
        }
        "Add 2–12 answers, each with 1–100 characters." => {
            gettext(locale, "Add 2–12 answers, each with 1–100 characters.")
        }
        "Each answer must be different." => gettext(locale, "Each answer must be different."),
        // Constants in views
        "Not sent" => gettext(locale, "Not sent"),
        "This message could not be sent, and ZapFast will not retry it. Send it again yourself." => {
            gettext(
                locale,
                "This message could not be sent, and ZapFast will not retry it. Send it again yourself.",
            )
        }
        "Enter the whole number, starting with the country code" => gettext(
            locale,
            "Enter the whole number, starting with the country code",
        ),
        // Download failures kept on the attachment
        "No longer available on WhatsApp's servers" => {
            gettext(locale, "No longer available on WhatsApp's servers")
        }
        "This attachment is larger than the 64 MiB download limit" => gettext(
            locale,
            "This attachment is larger than the 64 MiB download limit",
        ),
        other => Cow::Owned(other.to_owned()),
    }
}

/// Labels that `Content::summary` writes in English, alone or as
/// `Label: detail` and `Label (detail)`.
fn summary_label(locale: Locale, english: &str) -> Option<Cow<'static, str>> {
    Some(match english {
        "Photo" => gettext(locale, "Photo"),
        "GIF" => gettext(locale, "GIF"),
        "Video" => gettext(locale, "Video"),
        "Video message" => gettext(locale, "Video message"),
        "Voice message" => gettext(locale, "Voice message"),
        "Audio" => gettext(locale, "Audio"),
        "Document" => gettext(locale, "Document"),
        "Sticker" => gettext(locale, "Sticker"),
        "Sticker pack" => gettext(locale, "Sticker pack"),
        "Location" => gettext(locale, "Location"),
        "Live location" => gettext(locale, "Live location"),
        "Live location ended" => gettext(locale, "Live location ended"),
        "Contact" => gettext(locale, "Contact"),
        "Poll" => gettext(locale, "Poll"),
        "This message was deleted" => gettext(locale, "This message was deleted"),
        "Unsupported message" => gettext(locale, "Unsupported message"),
        "View once message" => gettext(locale, "View once message"),
        "Message on your phone" => gettext(locale, "Message on your phone"),
        _ => return None,
    })
}

/// Shows a stored English message summary in the interface language. Only
/// the label is translated; what follows it is the sender's own text.
pub fn summary(text: &str) -> Cow<'_, str> {
    summary_in(locale(), text)
}

fn summary_in(locale: Locale, text: &str) -> Cow<'_, str> {
    if locale == Locale::English {
        return Cow::Borrowed(text);
    }
    if let Some(label) = summary_label(locale, text) {
        return label;
    }
    if let Some((label, rest)) = text.split_once(": ")
        && let Some(local) = summary_label(locale, label)
    {
        return Cow::Owned(format!("{local}: {rest}"));
    }
    if text.ends_with(')')
        && let Some((label, rest)) = text.split_once(" (")
        && let Some(local) = summary_label(locale, label)
    {
        return Cow::Owned(format!("{local} ({rest}"));
    }
    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_translate_the_label_and_keep_the_sender_text() {
        let es = Locale::Spanish;
        assert_eq!(summary_in(es, "Photo"), "Foto");
        assert_eq!(summary_in(es, "Photo: Photo"), "Foto: Photo");
        assert_eq!(
            summary_in(es, "Document: informe.pdf"),
            "Documento: informe.pdf"
        );
        assert_eq!(
            summary_in(es, "Voice message (0:12)"),
            "Mensaje de voz (0:12)"
        );
        assert_eq!(summary_in(es, "hola: qué tal"), "hola: qué tal");
        assert_eq!(summary_in(Locale::English, "Photo"), "Photo");
    }

    #[test]
    fn every_spanish_message_is_translated() {
        let po = include_str!("../assets/i18n/es.po");
        assert!(!po.contains("#, fuzzy\n"), "fuzzy entries in es.po");
        let empty = po
            .split("\n\n")
            .skip(1)
            .filter(|entry| {
                !entry.starts_with("#~")
                    && entry.contains("msgstr \"\"")
                    && !entry.contains("msgstr \"\"\n\"")
            })
            .count();
        assert_eq!(empty, 0, "untranslated entries in es.po");
    }
}
