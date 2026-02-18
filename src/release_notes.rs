use std::fs;

pub fn release_notes_for_version(changelog_path: &str, version: &str) -> Option<String> {
    let content = fs::read_to_string(changelog_path).ok()?;
    extract_release_section(&content, version)
}

fn extract_release_section(content: &str, version: &str) -> Option<String> {
    let with_v = format!("## v{}", version);
    let without_v = format!("## {}", version);

    let mut in_target = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            if line.starts_with(&with_v) || line.starts_with(&without_v) {
                in_target = true;
                continue;
            }

            if in_target {
                break;
            }
        }

        if in_target {
            lines.push(line);
        }
    }

    let notes = lines.join("\n").trim().to_string();
    if notes.is_empty() { None } else { Some(notes) }
}

#[cfg(test)]
mod tests {
    use super::extract_release_section;

    #[test]
    fn extracts_version_section_with_v_prefix() {
        let changelog =
            "# Changelog\n\n## v0.8.0 - 2026-02-17\n\n### Features\n- Added X\n\n## v0.7.0\n- Old";
        let notes = extract_release_section(changelog, "0.8.0").expect("notes");
        assert!(notes.contains("### Features"));
        assert!(notes.contains("Added X"));
        assert!(!notes.contains("v0.7.0"));
    }

    #[test]
    fn returns_none_if_version_missing() {
        let changelog = "# Changelog\n\n## v0.7.0\n- Old";
        assert!(extract_release_section(changelog, "0.8.0").is_none());
    }
}
