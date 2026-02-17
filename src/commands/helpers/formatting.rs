use crate::system::{CommandError, CommandOutput};

const TELEGRAM_TEXT_HARD_LIMIT: usize = 4096;
const TELEGRAM_TEXT_SAFE_LIMIT: usize = 3900;
const TRUNCATE_NOTICE: &str = "\n\n⚠️ (Output was truncated...)";
const OUTPUT_HEAD_LINES: usize = 50;
const OUTPUT_TAIL_LINES: usize = 10;

pub(crate) fn command_body(output: &CommandOutput) -> String {
    let mut content = String::new();
    let stdout = limit_output_lines(output.stdout.trim());
    let stderr = limit_output_lines(output.stderr.trim());

    if !stdout.is_empty() {
        content.push_str(&stdout);
    }

    if !stderr.is_empty() {
        if !content.is_empty() {
            content.push_str("\n\n--- stderr ---\n");
        }
        content.push_str(&stderr);
    }

    if content.is_empty() {
        content.push_str("No output.");
    }

    if output.status != 0 {
        content.push_str(&format!("\n\n(exit status: {})", output.status));
    }

    content
}

pub(crate) fn as_html_block(title: &str, body: &str) -> String {
    let escaped_title = html_escape::encode_text(title);
    let body_budget = TELEGRAM_TEXT_SAFE_LIMIT.saturating_sub(TRUNCATE_NOTICE.len());
    let mut escaped_body = sanitize_and_truncate(body, body_budget);
    let was_truncated = html_escape::encode_text(body).len() > escaped_body.len();

    if was_truncated {
        escaped_body.push_str(TRUNCATE_NOTICE);
    }

    let message = format!("<b>{}</b>\n<pre>{}</pre>", escaped_title, escaped_body);
    if message.len() > TELEGRAM_TEXT_HARD_LIMIT {
        log::warn!("formatted Telegram message is close to hard limit");
    }
    message
}

pub(crate) fn command_error_html(error: &CommandError) -> String {
    format!(
        "<b>Command execution failed</b>\n<pre>{}</pre>",
        sanitize_and_truncate(&error.to_string(), TELEGRAM_TEXT_SAFE_LIMIT)
    )
}

fn limit_output_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= OUTPUT_HEAD_LINES + OUTPUT_TAIL_LINES {
        return text.to_string();
    }

    let head = lines
        .iter()
        .take(OUTPUT_HEAD_LINES)
        .copied()
        .collect::<Vec<_>>();
    let tail = lines
        .iter()
        .skip(lines.len() - OUTPUT_TAIL_LINES)
        .copied()
        .collect::<Vec<_>>();

    let omitted = lines.len() - (OUTPUT_HEAD_LINES + OUTPUT_TAIL_LINES);
    format!(
        "{}\n... ({} lines omitted) ...\n{}",
        head.join("\n"),
        omitted,
        tail.join("\n")
    )
}

fn sanitize_and_truncate(input: &str, max_escaped_len: usize) -> String {
    let escaped_full = html_escape::encode_text(input);
    if escaped_full.len() <= max_escaped_len {
        return escaped_full.into_owned();
    }

    let mut low = 0usize;
    let mut high = input.len();
    let mut best = "";

    while low <= high {
        let mid = (low + high) / 2;
        let candidate = truncate_to_char_boundary(input, mid);
        let escaped = html_escape::encode_text(candidate);

        if escaped.len() <= max_escaped_len {
            best = candidate;
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }

    html_escape::encode_text(best).into_owned()
}

fn truncate_to_char_boundary(input: &str, max_bytes: usize) -> &str {
    if input.len() <= max_bytes {
        return input;
    }

    let mut end = max_bytes;
    while !input.is_char_boundary(end) {
        end -= 1;
    }

    &input[..end]
}
