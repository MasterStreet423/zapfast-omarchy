//! The floating window that answers a notification's message: the chat's
//! name, the message with a few around it, and the field. See
//! [`crate::app::quick_reply`] for when it opens and where it goes.

use egui::{Align, CornerRadius, Frame, Key, Layout, Margin, Modifiers, Sense, Stroke, Ui};

use super::widgets;
use crate::app::App;
use crate::app::quick_reply::Outcome;
use crate::theme;

/// How many messages show, the notification's among them.
const CONTEXT: usize = 5;
/// The longest a message runs before an ellipsis; the full window has the rest.
const MAX_ROWS: usize = 6;

/// One message as the window shows it.
struct Row {
    /// The sender, in a group, or "You".
    sender: Option<String>,
    text: String,
    from_me: bool,
    /// The message the notification showed.
    target: bool,
}

/// Draws the window into `ui` and says what the reader asked for.
pub fn show(app: &mut App, ui: &mut Ui) -> Option<Outcome> {
    let reply = app.quick_reply()?;
    let (chat_id, target) = (reply.chat.clone(), reply.message.clone());
    let Some(chat) = app.chat(&chat_id) else {
        app.drop_quick_reply();
        return None;
    };
    let title = app.chat_title(chat);
    let is_group = chat.is_group();
    let rows = rows(app, &chat_id, &target, is_group);
    let picture = app.avatar(&chat_id);
    let palette = app.palette;
    let locale = app.locale;

    // The keys act before the field sees them: Enter would otherwise be a
    // new line and Esc would only take the focus away.
    let mut outcome = None;
    ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::Escape) {
            outcome = Some(Outcome::Close);
        } else if input.consume_key(Modifiers::COMMAND, Key::O) {
            outcome = Some(Outcome::OpenChat);
        } else if !input.modifiers.shift && input.consume_key(Modifiers::NONE, Key::Enter) {
            // Shift+Enter stays a new line, as in the full composer.
            outcome = Some(Outcome::Send);
        }
    });

    let back = Frame::new()
        .fill(palette.window)
        .inner_margin(Margin::same(12));
    back.show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        // Header: who, and the way to the full chat.
        ui.horizontal(|ui| {
            widgets::avatar(ui, &palette, &title, &chat_id, 32.0, picture.as_deref());
            ui.add_space(8.0);
            let open = crate::i18n::gettext(locale, "Open chat (Ctrl+O)");
            let open = egui::RichText::new(open.as_ref())
                .color(palette.accent)
                .font(theme::medium(13.0));
            // The name gives way to the button, with an ellipsis.
            let width = (ui.available_width() - 150.0).max(40.0);
            let name = widgets::line(ui, &title, theme::semibold(15.0), palette.text, width, 1);
            let (rect, _) = ui.allocate_exact_size(name.size(), Sense::hover());
            name.paint(ui, rect.min, palette.text);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(egui::Button::new(open).frame(false)).clicked() {
                    outcome = Some(Outcome::OpenChat);
                }
            });
        });
        ui.add_space(8.0);

        let field_height = 92.0;
        let list_height = (ui.available_height() - field_height).max(40.0);
        egui::ScrollArea::vertical()
            .max_height(list_height)
            .min_scrolled_height(list_height)
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if rows.is_empty() {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().color(palette.dim));
                        ui.label(
                            egui::RichText::new(crate::i18n::gettext(locale, "Loading…").as_ref())
                                .color(palette.dim),
                        );
                    });
                }
                for row in &rows {
                    message(ui, &palette, row);
                    ui.add_space(6.0);
                }
            });

        ui.add_space(8.0);
        let Some(reply) = app.quick_reply_mut() else {
            return;
        };
        if let Some(error) = &reply.error {
            ui.label(
                egui::RichText::new(error)
                    .color(palette.danger)
                    .font(theme::regular(12.5)),
            );
            ui.add_space(4.0);
        }
        let hint = if is_group {
            crate::i18n::gettext(locale, "Reply to this message")
        } else {
            crate::i18n::gettext(locale, "Type a message")
        };
        let field = Frame::new()
            .fill(palette.surface)
            .corner_radius(CornerRadius::same(theme::RADIUS))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut reply.text)
                        .id(egui::Id::new("quick-reply-text"))
                        .hint_text(egui::RichText::new(hint.as_ref()).color(palette.dim))
                        .font(theme::regular(15.0))
                        .text_color(palette.text)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY)
                        .frame(egui::Frame::NONE),
                )
            })
            .inner;
        if std::mem::take(&mut reply.focus) {
            field.request_focus();
        }
        if field.changed() {
            reply.error = None;
        }
    });
    outcome
}

/// The notification's message with the ones around it, oldest first: the
/// newest few, or, when more came since, the message with those just before.
fn rows(app: &App, chat: &str, target: &str, is_group: bool) -> Vec<Row> {
    let Some(conversation) = app.conversations.get(chat) else {
        return Vec::new();
    };
    let messages = &conversation.messages;
    let Some(at) = messages.iter().position(|message| message.id == target) else {
        return Vec::new();
    };
    let end = messages.len().min(at + CONTEXT).max(at + 1);
    let start = end.saturating_sub(CONTEXT);
    messages[start..end]
        .iter()
        .map(|message| Row {
            sender: if message.from_me {
                Some(crate::i18n::gettext(app.locale, "You").into_owned())
            } else if is_group {
                Some(app.display_name_or(&message.sender, message.sender_name.as_deref()))
            } else {
                None
            },
            text: app.message_text(message),
            from_me: message.from_me,
            target: message.id == target,
        })
        .collect()
}

fn message(ui: &mut Ui, palette: &theme::Palette, row: &Row) {
    let fill = if row.target {
        palette.surface_active
    } else if row.from_me {
        palette.bubble_out
    } else {
        palette.surface
    };
    let response = Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(theme::RADIUS))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(sender) = &row.sender {
                let color = if row.from_me {
                    palette.secondary
                } else {
                    palette.accent
                };
                widgets::rich_text(ui, sender, theme::semibold(12.5), color);
            }
            let width = ui.available_width();
            let line = widgets::line(
                ui,
                &row.text,
                theme::regular(14.0),
                palette.text,
                width,
                MAX_ROWS,
            );
            let (rect, _) = ui.allocate_exact_size(line.size(), Sense::hover());
            line.paint(ui, rect.min, palette.text);
        })
        .response;
    if row.target {
        // A bar down the left edge marks the message being answered.
        let rect = response.rect;
        let bar = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.left() + 3.0, rect.bottom()),
        );
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), palette.accent);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(theme::RADIUS),
            Stroke::new(1.0, palette.accent.gamma_multiply(0.5)),
            egui::StrokeKind::Inside,
        );
    }
}
