use super::super::super::helpers::as_html_block;

pub(super) fn unsupported_feature_message(feature: &str, command: &str) -> String {
    as_html_block(
        feature,
        &format!(
            "This feature is not supported on this system. Missing dependency: {}",
            command
        ),
    )
}
