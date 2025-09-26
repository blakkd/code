use std::io::Write;

use codex_core::config_types::Notifications;
use tracing::debug;

const APP_NAME: &str = "Code";
const OSC_PREFIX: &str = "\u{001b}]9;1;";
const BEL: char = '\u{0007}';
const MAX_BODY_LEN: usize = 160;

pub(crate) fn turn_complete(
    last_assistant_message: Option<&str>,
    config: &Notifications,
    has_focus: bool,
) {
    if !should_emit(config, "agent-turn-complete", has_focus) {
        return;
    }

    let body = last_assistant_message.map(truncate);
    emit("Turn complete", body.as_deref());
}

pub(crate) fn approval_requested(
    summary: &str,
    config: &Notifications,
    has_focus: bool,
) {
    if !should_emit(config, "approval-requested", has_focus) {
        return;
    }

    emit("Approval requested", Some(&truncate(summary)));
}

fn should_emit(config: &Notifications, notif_type: &str, has_focus: bool) -> bool {
    if has_focus {
        return false;
    }

    match config {
        Notifications::Enabled(enabled) => *enabled,
        Notifications::Custom(types) => types.iter().any(|t| t == notif_type),
    }
}

fn emit(subtitle: &str, body: Option<&str>) {
    let mut payload = String::with_capacity(OSC_PREFIX.len() + APP_NAME.len() + subtitle.len() + 8);
    payload.push_str(OSC_PREFIX);
    payload.push_str(APP_NAME);
    payload.push(':');
    payload.push_str(subtitle);
    if let Some(body) = body {
        payload.push('\n');
        payload.push_str(body);
    }
    payload.push(BEL);

    debug!("tui.notifications -> {subtitle}");

    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(payload.as_bytes());
    let _ = stdout.flush();
}

fn truncate(text: &str) -> String {
    if text.len() <= MAX_BODY_LEN {
        return text.to_string();
    }

    let mut acc = String::with_capacity(MAX_BODY_LEN);
    for (idx, line) in text.lines().enumerate() {
        let sep = if idx == 0 { "" } else { "\n" };
        if acc.len() + sep.len() + line.len() > MAX_BODY_LEN {
            if idx == 0 {
                acc.extend(line.chars().take(MAX_BODY_LEN));
            }
            acc.push('…');
            return acc;
        }
        acc.push_str(sep);
        acc.push_str(line);
    }

    acc
}
