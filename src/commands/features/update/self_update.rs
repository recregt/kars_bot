use std::cmp::Ordering;

use axoupdater::{AxoUpdater, ReleaseSource, ReleaseSourceType, UpdateRequest, Version};
use serde::Deserialize;

const GITHUB_API_LATEST_RELEASE: &str =
    "https://api.github.com/repos/recregt/kars_bot/releases/latest";
const APP_NAME: &str = "kars_bot";
const REPO_OWNER: &str = "recregt";
const REPO_NAME: &str = "kars_bot";
const DIST_MANIFEST_ASSET_NAME: &str = "dist-manifest.json";
const TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";

#[derive(Debug, Clone, Deserialize)]
pub(super) struct DistManifest {
    pub latest: DistManifestLatest,
    pub targets: Vec<DistManifestTarget>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct DistManifestLatest {
    pub version: String,
    pub tag: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct DistManifestTarget {
    pub target: String,
    pub archive_url: String,
    pub installer_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub(super) async fn fetch_latest_dist_manifest() -> Result<DistManifest, String> {
    let client = reqwest::Client::builder()
        .user_agent("kars_bot-update/1.0")
        .build()
        .map_err(|error| format!("http client build failed: {error}"))?;

    let release = client
        .get(GITHUB_API_LATEST_RELEASE)
        .send()
        .await
        .map_err(|error| format!("release query failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("release query status failed: {error}"))?
        .json::<GithubRelease>()
        .await
        .map_err(|error| format!("release payload decode failed: {error}"))?;

    let manifest_url = release
        .assets
        .iter()
        .find(|asset| asset.name == DIST_MANIFEST_ASSET_NAME)
        .map(|asset| asset.browser_download_url.clone())
        .ok_or_else(|| "dist-manifest.json asset missing in latest release".to_string())?;

    let manifest = client
        .get(manifest_url)
        .send()
        .await
        .map_err(|error| format!("dist manifest download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("dist manifest status failed: {error}"))?
        .json::<DistManifest>()
        .await
        .map_err(|error| format!("dist manifest decode failed: {error}"))?;

    if manifest.latest.tag != release.tag_name {
        return Err(format!(
            "dist manifest tag mismatch manifest={} release={}",
            manifest.latest.tag, release.tag_name
        ));
    }

    Ok(manifest)
}

pub(super) fn update_available(current_version: &str, latest_version: &str) -> bool {
    compare_semver(current_version, latest_version) == Ordering::Less
}

pub(super) fn summarize_manifest_readiness(manifest: &DistManifest) -> (bool, String) {
    let target_entry = manifest
        .targets
        .iter()
        .find(|target| target.target == TARGET_TRIPLE);
    let Some(target) = target_entry else {
        return (
            false,
            format!("Target {} not present in dist manifest", TARGET_TRIPLE),
        );
    };

    (
        true,
        format!(
            "Target {} is available.\narchive={}\ninstaller={}",
            TARGET_TRIPLE, target.archive_url, target.installer_url
        ),
    )
}

pub(super) async fn run_self_update(current_version: &str) -> Result<Option<String>, String> {
    let manifest = fetch_latest_dist_manifest().await?;

    if !update_available(current_version, &manifest.latest.version) {
        return Ok(None);
    }

    let (ready, detail) = summarize_manifest_readiness(&manifest);
    if !ready {
        return Err(format!("dist manifest readiness failed: {detail}"));
    }

    let current_version = Version::parse(current_version)
        .map_err(|error| format!("current version parse failed: {error}"))?;

    let mut updater = AxoUpdater::new_for(APP_NAME);
    updater
        .set_release_source(ReleaseSource {
            release_type: ReleaseSourceType::GitHub,
            owner: REPO_OWNER.to_string(),
            name: REPO_NAME.to_string(),
            app_name: APP_NAME.to_string(),
        })
        .set_install_dir(
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
                .ok_or_else(|| "cannot resolve current executable directory".to_string())?
                .to_string_lossy()
                .to_string(),
        )
        .set_current_version(current_version)
        .map_err(|error| format!("set current version failed: {error}"))?
        .configure_version_specifier(UpdateRequest::SpecificTag(manifest.latest.tag.clone()))
        .set_install_args(vec!["--no-modify-path".to_string()])
        .disable_installer_stdout()
        .disable_installer_stderr();

    let result = updater
        .run()
        .await
        .map_err(|error| format!("axoupdater run failed: {error}"))?;

    Ok(result.map(|updated| {
        format!(
            "Updated via axoupdater: v{} -> v{} ({})",
            updated
                .old_version
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            updated.new_version,
            updated.new_version_tag
        )
    }))
}

fn compare_semver(current: &str, latest: &str) -> Ordering {
    match (parse_semver(current), parse_semver(latest)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => Ordering::Equal,
    }
}

fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let mut it = version.split('.');
    let major = it.next()?.parse::<u64>().ok()?;
    let minor = it.next()?.parse::<u64>().ok()?;
    let patch = it.next()?.parse::<u64>().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}
