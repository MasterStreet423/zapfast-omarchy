//! Answering a message from its notification without opening ZapFast.
//!
//! A click on a message's notification opens a small floating window over
//! whatever is on screen: the message, a few around it for context, and a
//! field to answer. Enter sends and closes it, Esc closes it, and Ctrl+O
//! opens the chat in the full window instead.
//!
//! The window is one of two things. With ZapFast hidden in the tray, the shell
//! makes it the only window, under its own app id (`zapfast-reply`) so the
//! compositor can float it; it closes back to the tray. With the full window
//! up, it is a second viewport beside it, and the full window stays where it
//! is. Both draw [`crate::ui::quick_reply`].
//!
//! It never opens behind the app lock or for a locked chat: those clicks open
//! the full window, as they always did.

use super::App;
use crate::backend::{Command, LinkStatus};
use crate::model::{Action, ChatId, ReplyTarget};

/// The app id the compositor matches to float the window.
pub const APP_ID: &str = "zapfast-reply";

/// The answer being written to one message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickReply {
    pub chat: ChatId,
    /// The message the notification showed.
    pub message: String,
    pub text: String,
    /// Why the last Enter sent nothing, shown above the field.
    pub error: Option<String>,
    /// Whether the field still has to take the keyboard.
    pub focus: bool,
}

/// What the reader asked of the window this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Send,
    Close,
    OpenChat,
}

impl App {
    /// Starts answering `message` in a floating window. Returns `false` when
    /// the click should open the full window instead: the setting is off, the
    /// chat is gone or locked.
    pub(super) fn open_quick_reply(&mut self, chat: &str, message: &str) -> bool {
        if !self.settings.quick_reply {
            return false;
        }
        if self.chat(chat).is_none_or(|chat| chat.locked) {
            return false;
        }
        // A second click on the same message keeps what was written.
        let text = match &self.quick_reply {
            Some(open) if open.chat == chat && open.message == message => open.text.clone(),
            _ => String::new(),
        };
        self.quick_reply = Some(QuickReply {
            chat: chat.to_owned(),
            message: message.to_owned(),
            text,
            error: None,
            focus: true,
        });
        self.ensure_loaded(chat);
        if self.window_hidden {
            // The shell makes the floating window instead of the full one.
            self.quick_pending = true;
            self.wants_show = true;
        }
        true
    }

    pub fn quick_reply(&self) -> Option<&QuickReply> {
        self.quick_reply.as_ref()
    }

    pub fn quick_reply_mut(&mut self) -> Option<&mut QuickReply> {
        self.quick_reply.as_mut()
    }

    /// Lets the answer go without a word, as when its chat is gone.
    pub fn drop_quick_reply(&mut self) {
        self.quick_reply = None;
    }

    /// Whether the next window the shell makes is the floating one.
    pub fn quick_window_pending(&self) -> bool {
        self.quick_pending && self.quick_reply.is_some()
    }

    /// Whether the window on screen is the floating one alone.
    pub fn is_quick_window(&self) -> bool {
        self.quick_window
    }

    /// One frame of the floating window when it is the only window: the
    /// work every frame does, without the full interface.
    pub fn quick_frame_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.window_focused = ctx.input(|input| input.viewport().focused.unwrap_or(true));
        self.report_presence();
        let outcome = crate::ui::quick_reply::show(self, ui);
        if self.quick_reply.is_none() {
            // Nothing to answer any more (the chat went away): back to the tray.
            self.close_quick_reply(&ctx);
        } else if let Some(outcome) = outcome {
            self.quick_outcome(outcome, &ctx);
        }
        self.apply_actions(&ctx);
    }

    /// Shows the floating window beside the full one, while there is an
    /// answer under way. Called from the full window's frame.
    pub(super) fn show_quick_viewport(&mut self, ctx: &egui::Context) {
        if self.quick_window || self.quick_reply.is_none() {
            return;
        }
        let title = crate::i18n::gettext(self.locale, "Reply").into_owned();
        let builder = egui::ViewportBuilder::default()
            .with_title(title)
            .with_app_id(APP_ID)
            .with_inner_size(SIZE)
            .with_min_inner_size(MIN_SIZE)
            .with_decorations(false);
        let id = egui::ViewportId::from_hash_of(APP_ID);
        let outcome = ctx.show_viewport_immediate(id, builder, |ctx, _class| {
            let mut outcome = None;
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| outcome = crate::ui::quick_reply::show(self, ui));
            if ctx.input(|input| input.viewport().close_requested()) {
                outcome = Some(Outcome::Close);
            }
            outcome
        });
        if let Some(outcome) = outcome {
            self.quick_outcome(outcome, ctx);
        }
    }

    fn quick_outcome(&mut self, outcome: Outcome, ctx: &egui::Context) {
        match outcome {
            Outcome::Send => {
                if self.send_quick_reply() {
                    self.close_quick_reply(ctx);
                }
            }
            Outcome::Close => {
                self.keep_quick_draft();
                self.close_quick_reply(ctx);
            }
            Outcome::OpenChat => {
                // What was written goes on in the full window's composer.
                self.keep_quick_draft();
                let Some(reply) = self.quick_reply.take() else {
                    return;
                };
                if self.open_chat.as_deref() == Some(&reply.chat) && self.composer.trim().is_empty()
                {
                    self.composer = reply.text.trim().to_owned();
                }
                self.actions.push(Action::OpenMessage {
                    chat: reply.chat,
                    message: reply.message,
                });
                if self.quick_window {
                    // The floating window closes and the full one opens.
                    self.reopen = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    self.actions.push(Action::ShowWindow);
                }
            }
        }
    }

    /// Sends the answer, quoting the message in a group, where a bare answer
    /// may read as meant for someone else. Returns whether it went: offline,
    /// the window stays with the text and says why.
    fn send_quick_reply(&mut self) -> bool {
        let connected = self.link == LinkStatus::Connected;
        let locale = self.locale;
        let Some(reply) = self.quick_reply.as_mut() else {
            return true;
        };
        let text = reply.text.trim().to_owned();
        if text.is_empty() {
            return false;
        }
        if !connected {
            reply.error = Some(
                crate::i18n::gettext(locale, "Not sent: ZapFast is not connected to WhatsApp.")
                    .into_owned(),
            );
            return false;
        }
        let (chat, message) = (reply.chat.clone(), reply.message.clone());
        let quoting = self
            .chat(&chat)
            .is_some_and(|chat| chat.is_group())
            .then(|| ReplyTarget::from(message));
        self.backend.send(Command::SendText {
            chat: chat.clone(),
            text,
            quoting,
            mentions: Vec::new(),
            // A quick answer goes out at once, without waiting for a link's page.
            preview: None,
        });
        // Answering is reading: the phone and the list stop counting it.
        self.mark_read(&chat);
        self.follow_sent_chat();
        true
    }

    /// Keeps an unsent answer as the chat's draft, so Esc never loses what
    /// was written. A draft already there, or the chat open in the full
    /// window, wins: neither is overwritten.
    fn keep_quick_draft(&mut self) {
        let Some(reply) = &self.quick_reply else {
            return;
        };
        let text = reply.text.trim();
        if text.is_empty()
            || self.open_chat.as_deref() == Some(&reply.chat)
            || self
                .drafts
                .get(&reply.chat)
                .is_some_and(|draft| !draft.trim().is_empty())
        {
            return;
        }
        let (chat, text) = (reply.chat.clone(), text.to_owned());
        self.store_draft(&chat, &text);
        self.drafts.insert(chat, text);
    }

    fn close_quick_reply(&mut self, ctx: &egui::Context) {
        self.quick_reply = None;
        if self.quick_window {
            // Back to the tray, whatever closing the full window would do.
            self.hide_intent = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

/// The floating window's size, in points.
pub const SIZE: [f32; 2] = [440.0, 420.0];
const MIN_SIZE: [f32; 2] = [320.0, 240.0];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Backend;
    use crate::model::{AccountId, Chat, Content, Delivery, Message};
    use crate::notify::NotificationTarget;
    use crate::paths::AppDirs;
    use crate::settings::Settings;

    const ADA: &str = "15550001111@s.whatsapp.net";
    const GROUP: &str = "1-2@g.us";

    /// A hidden window with a direct chat and a group, each holding the
    /// message a notification announced, and a backend a test can read.
    fn app(directory: &std::path::Path) -> (App, tokio::sync::mpsc::UnboundedReceiver<Command>) {
        let mut app = App::headless(AppDirs::under(directory), Settings::default()).0;
        let (backend, commands) = Backend::recording();
        app.backend = backend;
        app.link = LinkStatus::Connected;
        app.window_hidden = true;
        for chat in [ADA, GROUP] {
            app.chats.push(Chat::new(chat.into(), "Fixture".into()));
            let messages = (1..=3)
                .map(|n| message(chat, &format!("m{n}"), n))
                .collect();
            app.conversations
                .entry(chat.into())
                .or_default()
                .merge(messages, false);
        }
        (app, commands)
    }

    fn message(chat: &str, id: &str, timestamp: i64) -> Message {
        Message {
            id: id.into(),
            chat: chat.into(),
            sender: chat.into(),
            sender_name: None,
            from_me: false,
            timestamp,
            content: Content::text(id),
            status: Delivery::None,
            delivered_at: None,
            read_at: None,
            quoted: None,
            reactions: Vec::new(),
            history_order: None,
            edited: false,
            mentions: Vec::new(),
            forwarded: false,
            thumbnail: None,
        }
    }

    fn click(app: &mut App, chat: &str, message: &str) {
        app.notification_opens
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(NotificationTarget {
                account: AccountId::first(),
                chat: chat.into(),
                message: Some(message.into()),
            });
        app.handle_notification_opens();
    }

    fn sent(commands: &mut tokio::sync::mpsc::UnboundedReceiver<Command>) -> Vec<Command> {
        std::iter::from_fn(|| commands.try_recv().ok()).collect()
    }

    /// Hidden in the tray, a click makes the floating window, not the full
    /// one, and leaves the full window's chat alone.
    #[test]
    fn a_click_while_hidden_asks_for_the_floating_window() {
        let directory = tempfile::tempdir().unwrap();
        let (mut app, _) = app(directory.path());
        click(&mut app, ADA, "m2");
        assert!(app.wants_show);
        assert!(app.quick_window_pending());
        assert!(
            !app.actions
                .iter()
                .any(|action| matches!(action, Action::OpenMessage { .. }))
        );
        assert_eq!(app.open_chat, None);
    }

    /// With the full window up, the answer opens beside it: nothing asks the
    /// shell for a window, and the full one stays as it is.
    #[test]
    fn a_click_with_the_window_up_answers_beside_it() {
        let directory = tempfile::tempdir().unwrap();
        let (mut app, _) = app(directory.path());
        app.window_hidden = false;
        click(&mut app, ADA, "m2");
        assert!(app.quick_reply.is_some());
        assert!(!app.wants_show);
        assert!(!app.quick_window_pending());
        assert!(
            !app.actions
                .iter()
                .any(|action| matches!(action, Action::ShowWindow))
        );
    }

    /// The app lock, a locked chat and the setting turned off all keep the
    /// old click: the full window, at the message.
    #[test]
    fn locks_and_the_setting_keep_the_full_window() {
        let directory = tempfile::tempdir().unwrap();
        let (mut off, _) = app(directory.path());
        off.settings.quick_reply = false;
        click(&mut off, ADA, "m2");
        assert!(off.quick_reply.is_none());
        assert!(
            off.actions
                .iter()
                .any(|action| matches!(action, Action::OpenMessage { .. }))
        );

        let (mut locked_chat, _) = app(directory.path());
        locked_chat.chats[0].locked = true;
        click(&mut locked_chat, ADA, "m2");
        assert!(locked_chat.quick_reply.is_none());
    }

    /// In a group the answer quotes the message; in a direct chat it goes
    /// bare. Either way the chat counts as read and the window closes.
    #[test]
    fn a_group_answer_quotes_and_a_direct_one_does_not() {
        let ctx = egui::Context::default();
        for (chat, quotes) in [(GROUP, true), (ADA, false)] {
            let directory = tempfile::tempdir().unwrap();
            let (mut app, mut commands) = app(directory.path());
            click(&mut app, chat, "m2");
            app.quick_reply.as_mut().unwrap().text = "  ok  ".into();
            app.quick_outcome(Outcome::Send, &ctx);
            let sent = sent(&mut commands);
            assert!(sent.iter().any(|command| matches!(command,
                Command::SendText { chat: to, text, quoting, .. }
                    if to == chat && text == "ok"
                        && quoting.as_ref().map(|quote| quote.id.as_str())
                            == quotes.then_some("m2"))));
            assert!(
                sent.iter()
                    .any(|command| matches!(command, Command::MarkRead { .. }))
            );
            assert!(app.quick_reply.is_none());
        }
    }

    /// Offline, Enter sends nothing and the window stays with the text and
    /// the reason, instead of closing on a message that never left.
    #[test]
    fn an_offline_answer_stays_in_the_window() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let (mut app, mut commands) = app(directory.path());
        app.link = LinkStatus::Connecting;
        click(&mut app, ADA, "m2");
        app.quick_reply.as_mut().unwrap().text = "ok".into();
        app.quick_outcome(Outcome::Send, &ctx);
        assert!(
            !sent(&mut commands)
                .iter()
                .any(|command| matches!(command, Command::SendText { .. }))
        );
        let reply = app.quick_reply.as_ref().unwrap();
        assert_eq!(reply.text, "ok");
        assert!(reply.error.is_some());
    }

    /// Esc keeps what was written as the chat's draft, unless the chat
    /// already had one.
    #[test]
    fn closing_keeps_the_answer_as_a_draft() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let (mut app, _) = app(directory.path());
        click(&mut app, ADA, "m2");
        app.quick_reply.as_mut().unwrap().text = "half".into();
        app.quick_outcome(Outcome::Close, &ctx);
        assert!(app.quick_reply.is_none());
        assert_eq!(app.drafts.get(ADA).map(String::as_str), Some("half"));

        app.drafts.insert(GROUP.into(), "mine".into());
        click(&mut app, GROUP, "m2");
        app.quick_reply.as_mut().unwrap().text = "other".into();
        app.quick_outcome(Outcome::Close, &ctx);
        assert_eq!(app.drafts.get(GROUP).map(String::as_str), Some("mine"));
    }

    /// Ctrl+O in the floating window closes it into the full one, at the
    /// message, and the window made next is the full one.
    #[test]
    fn opening_the_chat_trades_the_floating_window_for_the_full_one() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let (mut app, _) = app(directory.path());
        click(&mut app, ADA, "m2");
        // What `attach` does as the shell makes the window.
        app.quick_window = std::mem::take(&mut app.quick_pending);
        assert!(app.is_quick_window());
        app.quick_outcome(Outcome::OpenChat, &ctx);
        assert!(app.reopen);
        assert!(app.quick_reply.is_none());
        assert!(!app.quick_window_pending());
        assert!(app.actions.iter().any(|action| matches!(action,
            Action::OpenMessage { chat, message } if chat == ADA && message == "m2")));
    }
}
