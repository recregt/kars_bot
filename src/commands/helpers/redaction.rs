pub(crate) fn maybe_redact_sensitive_output(body: &str, enabled: bool) -> String {
    if !enabled {
        return body.to_string();
    }

    let redacted_ip = redact_ipv4_tokens(body);
    redact_port_suffixes(&redacted_ip)
}

fn redact_ipv4_tokens(input: &str) -> String {
    input
        .split_whitespace()
        .map(redact_ipv4_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_ipv4_token(token: &str) -> String {
    let trimmed = token.trim_matches(|ch: char| ",;()[]{}".contains(ch));
    let ipv4_candidate = trimmed.split(':').next().unwrap_or(trimmed);
    let dot_count = ipv4_candidate.chars().filter(|ch| *ch == '.').count();
    if dot_count != 3 {
        return token.to_string();
    }

    let is_ipv4 = ipv4_candidate
        .split('.')
        .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()));
    if !is_ipv4 {
        return token.to_string();
    }

    token.replacen(ipv4_candidate, "[REDACTED_IP]", 1)
}

fn redact_port_suffixes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if ch == ':' {
            let mut lookahead = index + 1;
            let mut digits = 0usize;
            while lookahead < chars.len() && chars[lookahead].is_ascii_digit() && digits < 5 {
                lookahead += 1;
                digits += 1;
            }

            if digits > 0 {
                out.push_str(":[REDACTED_PORT]");
                index = lookahead;
                continue;
            }
        }

        out.push(ch);
        index += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::maybe_redact_sensitive_output;

    #[test]
    fn keeps_original_when_redaction_disabled() {
        let input = "inet 192.168.1.20:8080";
        assert_eq!(maybe_redact_sensitive_output(input, false), input);
    }

    #[test]
    fn redacts_ipv4_and_port_tokens() {
        let input = "src 192.168.1.20:8080 dst 10.0.0.5:443";
        let out = maybe_redact_sensitive_output(input, true);
        assert!(out.contains("[REDACTED_IP]:[REDACTED_PORT]"));
        assert!(!out.contains("192.168.1.20"));
        assert!(!out.contains(":8080"));
    }
}
