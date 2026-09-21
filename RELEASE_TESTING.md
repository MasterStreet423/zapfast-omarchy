# Release testing

Record the candidate commit, operating system, installation type and result for
each check. Use synthetic messages with a test account for connected checks.
Screenshots and recordings for release notes must use the offline demo.

## Check first

- **Upgrade and restart, all platforms.** Upgrade an existing 0.14 installation
  without unlinking. Confirm the account, history and settings survive. Quit
  completely and start again. Test keyring recovery only with a disposable
  profile and mock credentials, never by deleting your real archive key.
- **Locked chats, phone plus desktop.** Lock a test chat on the phone and confirm
  it disappears from the normal list, search, unread totals, notifications and
  forwarding destinations. Repeat while the chat is open, then restart ZapFast.
  Open Locked beside the filters and set a local code: a wrong code must reveal
  nothing; the correct code should open the folder, including archived locked
  chats. Search within it, then choose All: messages must become hidden again.
  With no locked chats, the Locked chip should disappear. Filters stay on one
  row, scrolling horizontally if needed. Close the window while viewing a locked
  chat, then reopen from the tray: the code must be required again. Change or
  remove the local code and check that the old code stops working. Unlock on
  the phone and confirm the chat returns. The local code is separate from the
  phone's code. Revealed locked chats are currently read-only, including for
  forwarding; the composer must explain this without referring to group admins.
- **macOS editing.** In the composer and search, try Select All, Copy, Cut,
  Paste, Undo and Redo both from the Edit menu and with Command shortcuts.
  Copy a selection spanning several messages, including emoji. Repeat after
  closing to the tray and reopening. In a native file dialog, edit its filename
  or search field with the same shortcuts. Test with VoiceOver both off and on.
- **Image paste, all platforms.** Copy an image from a browser that also offers
  its URL or HTML, then paste into a composer with an existing caption. Exactly
  one picture should be staged and the caption should stay unchanged. Hold the
  paste keys briefly before releasing them to check for duplication. Repeat
  with a screenshot, plain text, and the macOS Paste menu. Text paste in search
  and dialogs must still work. Escape should discard the staged picture.
- **Idle and background behaviour.** In an optimized build, leave an ordinary
  text conversation untouched for 30 seconds. Compare CPU usage with the window
  focused, unfocused, on another workspace, and closed to the tray. Confirm no
  sustained busy core, frozen window or lost notifications. Reopen from the
  tray, a notification and a second launch. Muted and locked chats must remain
  silent. Moving between workspaces must not mark hidden messages as read.

## New features and platform coverage

- **Keyboard focus.** Use Tab and Shift+Tab through chat rows, filters, Settings
  and a dialog. The focused control must be visible, outlined and scrolled into
  view. Enter or Space should activate it; mouse input should hide the outline.
  The message input keeps an outline while active, and the voice button gets a
  circular ring. Continue past it into message controls: no invisible targets,
  and the conversation must reveal focused messages instead of snapping down.
  `?` opens help outside text fields but types normally in the composer and search.
- **New chat and group names.** Use the pencil or Ctrl+N (Command+N on macOS),
  Message yourself, contact search and Add contact. Confirm the self-chat is not
  duplicated. An unnamed group with repeated first names should use the same
  counted summary in its title and subtitle, including after restart. Check a
  group deliberately named "Group" still keeps its real name after metadata sync.
- **Reactions.** Open a message menu, then +. The menu must stay visible beside
  the emoji picker, with the target highlighted and previewed. Scroll over the
  conversation: it must stay still; scrolling the emoji grid still works. Escape,
  outside click and choosing a reaction close both. Repeated choices should rise
  in quick reactions and survive restart; removing a reaction does not raise usage.
- **Messaging.** Send text and a captioned picture in a direct chat and a group.
  Reply with a double-click, react with an emoji outside the quick list, edit a
  message, and verify the results on the phone. Check sent/delivered/read states
  without assuming a group is read when only one participant has read it.
- **Voice.** Record and play a short test message. Cycle 1x, 1.5x and 2x while
  playing, seek, and confirm intelligible speech without a pitch change.
- **History and polls.** Load older history, reconnect, create a test poll and
  change or withdraw a vote on the phone. Confirm later history does not undo
  the latest vote. Earlier totals still depend on what the phone supplies.
- **Text and themes.** Check Arabic and Hebrew mixed with Latin text, numbers
  and emoji, then copy back to a plain text editor. Compare against the phone;
  custom fonts do not establish correct text ordering. Check the included
  themes and desktop-following colours at normal and increased zoom.
- **Nix.** On a NixOS desktop, run `nix flake check --no-build`, `nix build
  .#default`, and enter `nix develop`. Launch the package and check fonts, emoji,
  file picking, notifications, tray behaviour, keyring access and audio. A
  successful package build alone does not verify desktop integration. CI builds
  x86_64-linux and aarch64-linux.
- **Accessibility.** Check message text, control names, selected chats and
  switch states using NVDA on Windows or VoiceOver on macOS. Record platform
  gaps explicitly.

## Automated and release gates

Run on the final candidate:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-targets --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps
cargo audit
```

Require completed Linux, macOS and Windows CI tests, the Windows arm64 compile
check, and both Nix package builds. Report runtime testing separately from
compilation. Check the offline demo with `--demo-shot` for changed UI surfaces.

Before publishing, check direct-install update/restart and rollback using test
installations; package-managed copies should still direct users to their package
manager. Follow the release sequence in `AGENTS.md`, including all platform
assets, checksums, written notes, website downloads and native packages. Updater
publisher signatures remain separate work tracked in issue #103.
