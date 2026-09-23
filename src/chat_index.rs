//! Lets desktop launchers list and open chats: the running app keeps a small
//! index of recent chats in the runtime directory, and `zapfast open-chat <id>`
//! asks it to open one. Locked chats never appear in the index.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::app::App;

/// File in the runtime directory that launchers read.
const INDEX_FILE: &str = "chats.json";
/// Most recent chats listed.
const LIMIT: usize = 60;
/// How often the index is rebuilt at most.
const INTERVAL: Duration = Duration::from_secs(2);
/// Longest chat id accepted from another process.
const ID_LIMIT: usize = 128;

#[derive(Serialize)]
struct Entry {
    id: String,
    name: String,
    unread: u32,
    group: bool,
}

struct Written {
    at: Instant,
    json: String,
}

static LAST: Mutex<Option<Written>> = Mutex::new(None);

/// Rewrites the index when the recent chats changed, at most every `INTERVAL`.
pub fn sync(app: &App) {
    let mut last = LAST.lock().unwrap_or_else(|p| p.into_inner());
    if last.as_ref().is_some_and(|w| w.at.elapsed() < INTERVAL) {
        return;
    }
    let json = index(app);
    if last.as_ref().is_some_and(|w| w.json == json) {
        if let Some(w) = last.as_mut() {
            w.at = Instant::now();
        }
        return;
    }
    if let Err(error) = write(&app.dirs.runtime, &json) {
        log::debug!("could not write the chat index: {error}");
    }
    *last = Some(Written {
        at: Instant::now(),
        json,
    });
}

fn index(app: &App) -> String {
    let mut chats: Vec<_> = app
        .chats
        .iter()
        .filter(|chat| !chat.locked && !chat.archived)
        .collect();
    chats.sort_by_key(|chat| std::cmp::Reverse(chat.last_activity));
    let entries: Vec<Entry> = chats
        .into_iter()
        .take(LIMIT)
        .map(|chat| Entry {
            id: chat.id.clone(),
            name: app.chat_title(chat),
            unread: chat.unread,
            group: chat.is_group(),
        })
        .collect();
    serde_json::to_string(&entries).unwrap_or_default()
}

/// Written through a temporary file and renamed, readable only by the user.
fn write(dir: &Path, json: &str) -> std::io::Result<()> {
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::create_dir_all(dir)?;
    let temporary = dir.join(format!("{INDEX_FILE}.tmp"));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(&temporary)?.write_all(json.as_bytes())?;
    std::fs::rename(temporary, dir.join(INDEX_FILE))
}

/// The chat id in an `open-chat:<id>` request, if it looks like one.
pub fn parse_open(verb: &str) -> Option<String> {
    let id = verb.strip_prefix("open-chat:")?;
    let valid = !id.is_empty()
        && id.len() <= ID_LIMIT
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '-' | ':'));
    valid.then(|| id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_requests_accept_only_chat_ids() {
        assert_eq!(
            parse_open("open-chat:56912345678@s.whatsapp.net").as_deref(),
            Some("56912345678@s.whatsapp.net")
        );
        assert_eq!(parse_open("open-chat:"), None);
        assert_eq!(parse_open("open-chat:a b"), None);
        assert_eq!(parse_open("open-chat:x/../y"), None);
        assert_eq!(parse_open("show"), None);
        assert_eq!(parse_open(&format!("open-chat:{}", "1".repeat(200))), None);
    }
}
