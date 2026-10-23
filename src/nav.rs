//! Browser-like history for the mouse's back and forward buttons.

use crate::model::{ChatId, Page};

/// A place in the interface the back and forward buttons return to: the page
/// and the chat open on it. Overlays are not places; Escape closes those, and
/// the buttons stay inert while one is open. Chats belong to the account that
/// owns them, so the history is cleared when the window switches accounts.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    /// The page the window shows.
    pub page: Page,
    /// The chat open on that page, if any.
    pub chat: Option<ChatId>,
}

/// The places visited in this window and where the back and forward buttons
/// stand among them.
#[derive(Default)]
pub struct History {
    /// Places before the current one, oldest first.
    past: Vec<Location>,
    /// Places after the current one, the most recently left first.
    future: Vec<Location>,
    /// The place now showing, if one has been recorded.
    current: Option<Location>,
}

impl History {
    /// The most places kept; older ones fall off the back.
    const LIMIT: usize = 64;

    /// Whether the page and chat now showing are already the recorded place.
    pub fn is_current(&self, page: &Page, chat: Option<&str>) -> bool {
        self.current
            .as_ref()
            .is_some_and(|current| current.page == *page && current.chat.as_deref() == chat)
    }

    /// Records `location` as the place now showing. Visiting a new place drops
    /// the forward history, as a browser does.
    pub fn visit(&mut self, location: Location) {
        if self.current.as_ref() == Some(&location) {
            return;
        }
        let Some(previous) = self.current.replace(location) else {
            return;
        };
        self.past.push(previous);
        if self.past.len() > Self::LIMIT {
            self.past.remove(0);
        }
        self.future.clear();
    }

    /// Forgets every place that had `chat` open, for a chat that is gone.
    pub fn forget(&mut self, chat: &str) {
        self.past
            .retain(|location| location.chat.as_deref() != Some(chat));
        self.future
            .retain(|location| location.chat.as_deref() != Some(chat));
        if self
            .current
            .as_ref()
            .is_some_and(|location| location.chat.as_deref() == Some(chat))
        {
            // The next recorded place takes its place without pushing.
            self.current = None;
        }
    }

    /// Steps back, returning the place to show, or `None` at the oldest.
    pub fn back(&mut self) -> Option<Location> {
        let target = self.past.pop()?;
        if let Some(current) = self.current.replace(target.clone()) {
            self.future.push(current);
        }
        Some(target)
    }

    /// Steps forward again, returning the place to show, or `None` at the
    /// newest.
    pub fn forward(&mut self) -> Option<Location> {
        let target = self.future.pop()?;
        if let Some(current) = self.current.replace(target.clone()) {
            self.past.push(current);
        }
        Some(target)
    }

    /// Forgets every place, for another session or account.
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.current = None;
    }
}
