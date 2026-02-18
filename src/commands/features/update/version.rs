use std::{cmp::Ordering, fs};

pub(super) fn latest_changelog_version(changelog_path: &str) -> Option<String> {
    let content = fs::read_to_string(changelog_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("## ") {
            continue;
        }

        let after_header = trimmed.trim_start_matches("## ").trim();
        let version_token = after_header
            .split_whitespace()
            .next()
            .map(|token| token.trim_start_matches('v'))?;

        if parse_version(version_token).is_some() {
            return Some(version_token.to_string());
        }
    }

    None
}

pub(super) fn compare_versions(current: &str, latest: &str) -> Ordering {
    match (parse_version(current), parse_version(latest)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => Ordering::Equal,
    }
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut it = version.split('.');
    let major = it.next()?.parse::<u64>().ok()?;
    let minor = it.next()?.parse::<u64>().ok()?;
    let patch = it.next()?.parse::<u64>().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::{compare_versions, parse_version};

    #[test]
    fn parses_semver_triplet() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("x.y.z"), None);
    }

    #[test]
    fn compares_versions_correctly() {
        assert_eq!(compare_versions("1.0.0", "1.1.0"), Ordering::Less);
        assert_eq!(compare_versions("1.1.0", "1.1.0"), Ordering::Equal);
        assert_eq!(compare_versions("1.2.0", "1.1.9"), Ordering::Greater);
    }
}
