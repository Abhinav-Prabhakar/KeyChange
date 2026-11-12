//! Updates from GitHub Releases.
//!
//! KeyChange is ad-hoc signed, so Sparkle's signed-appcast model does not fit.
//! Instead we ask the public Releases API for the latest tag, require a
//! published SHA-256 next to the DMG, verify it before mounting, and hand the
//! result either to a small relaunch script or to Finder. Failure always ends
//! with the user still holding a working instrument.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

const API: &str = "https://api.github.com/repos/Abhinav-Prabhakar/KeyChange/releases/latest";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub dmg_url: String,
    pub checksum_url: String,
    pub page: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Staged {
    pub dmg: PathBuf,
    pub app: PathBuf,
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// `Some` when a newer release exists. `None` means this is the latest build.
pub fn check() -> Result<Option<Release>, String> {
    let json = run(
        "curl",
        &[
            "-fsSL",
            "--max-time",
            "15",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: KeyChange",
            API,
        ],
    )?;
    let api: ApiRelease =
        serde_json::from_str(&json).map_err(|err| format!("unreadable release feed: {err}"))?;
    let version = api.tag_name.trim_start_matches('v').to_string();
    if !is_newer(&version, current()) {
        return Ok(None);
    }
    let dmg = api
        .assets
        .iter()
        .find(|a| a.name.starts_with("KeyChange-") && a.name.ends_with(".dmg"))
        .ok_or_else(|| "the release has no DMG".to_string())?;
    let checksum = api
        .assets
        .iter()
        .find(|a| a.name.ends_with(".dmg.sha256") || a.name.ends_with(".sha256"))
        .ok_or_else(|| "the release has no checksum".to_string())?;
    Ok(Some(Release {
        version,
        dmg_url: dmg.browser_download_url.clone(),
        checksum_url: checksum.browser_download_url.clone(),
        page: api.html_url,
    }))
}

/// Compare dotted numeric versions. A suffix after `-` is ignored so
/// `0.5.0-beta.1` still sorts as `0.5.0`.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    fn parts(version: &str) -> Vec<u64> {
        version
            .trim_start_matches('v')
            .split('-')
            .next()
            .unwrap_or("0")
            .split('.')
            .map(|part| part.parse().unwrap_or(0))
            .collect()
    }
    let mut a = parts(candidate);
    let mut b = parts(current);
    let len = a.len().max(b.len());
    a.resize(len, 0);
    b.resize(len, 0);
    a > b
}

/// Download the DMG and its checksum, verify, and copy the app out of the
/// mounted image. The image is detached before returning.
pub fn download(release: &Release) -> Result<Staged, String> {
    let dir = cache_dir();
    fs::create_dir_all(&dir).map_err(|err| format!("could not create update folder: {err}"))?;
    let dmg = dir.join(format!("KeyChange-{}.dmg", release.version));
    run(
        "curl",
        &[
            "-fL",
            "--max-time",
            "300",
            "-o",
            &dmg.to_string_lossy(),
            &release.dmg_url,
        ],
    )?;

    let sums = run(
        "curl",
        &["-fsSL", "--max-time", "15", &release.checksum_url],
    )?;
    let expected = sums
        .split_whitespace()
        .next()
        .filter(|sum| sum.len() == 64)
        .ok_or_else(|| "the checksum asset is empty".to_string())?;
    let actual = run("shasum", &["-a", "256", &dmg.to_string_lossy()])?;
    let actual = actual.split_whitespace().next().unwrap_or("");
    if !actual.eq_ignore_ascii_case(expected) {
        let _ = fs::remove_file(&dmg);
        return Err("checksum mismatch — the download was discarded".to_string());
    }

    let mount = dir.join("mnt");
    let _ = fs::remove_dir_all(&mount);
    fs::create_dir_all(&mount).map_err(|err| format!("could not create mount point: {err}"))?;
    let mounted = run(
        "hdiutil",
        &[
            "attach",
            &dmg.to_string_lossy(),
            "-nobrowse",
            "-readonly",
            "-mountpoint",
            &mount.to_string_lossy(),
        ],
    );
    if let Err(err) = mounted {
        return Err(format!("could not open the DMG: {err}"));
    }
    let source = mount.join("KeyChange.app");
    let staged = dir.join("KeyChange.app");
    let _ = fs::remove_dir_all(&staged);
    let copied = run(
        "ditto",
        &[&source.to_string_lossy(), &staged.to_string_lossy()],
    );
    let _ = run("hdiutil", &["detach", &mount.to_string_lossy(), "-force"]);
    copied?;
    if !staged.exists() {
        return Err("the DMG did not contain KeyChange.app".to_string());
    }
    Ok(Staged { dmg, app: staged })
}

/// Replace the running bundle and relaunch. Returns `false` when the bundle is
/// not writable (the usual case for `/Applications`): the DMG is opened so the
/// user can drag it across, and nothing is deleted.
pub fn install(staged: &Staged) -> Result<bool, String> {
    let Some(bundle) = bundle_path() else {
        run("open", &[&staged.dmg.to_string_lossy()])?;
        return Ok(false);
    };
    if !can_replace(&bundle) {
        run("open", &[&staged.dmg.to_string_lossy()])?;
        return Ok(false);
    }

    let script = cache_dir().join("install.sh");
    let body = format!(
        "#!/bin/bash\n\
         sleep 0.6\n\
         while kill -0 {pid} 2>/dev/null; do sleep 0.2; done\n\
         ditto \"{staged}\" \"{bundle}\"\n\
         xattr -dr com.apple.quarantine \"{bundle}\" 2>/dev/null || true\n\
         open \"{bundle}\"\n",
        pid = std::process::id(),
        staged = staged.app.to_string_lossy(),
        bundle = bundle.to_string_lossy(),
    );
    fs::write(&script, body).map_err(|err| format!("could not write installer: {err}"))?;
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script)
            .map_err(|err| err.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).map_err(|err| err.to_string())?;
    }
    Command::new("/bin/sh")
        .arg(&script)
        .spawn()
        .map_err(|err| format!("could not start installer: {err}"))?;
    Ok(true)
}

fn bundle_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let bundle = exe
        .parent()? // Contents/MacOS
        .parent()? // Contents
        .parent()?; // KeyChange.app
    (bundle.extension()?.to_str()? == "app").then(|| bundle.to_path_buf())
}

fn can_replace(bundle: &Path) -> bool {
    let Some(parent) = bundle.parent() else {
        return false;
    };
    let probe = parent.join(".keychange-write-test");
    match fs::write(&probe, b"") {
        Ok(()) => {
            let _ = fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library/Caches/KeyChange/updates")
        .join(format!("v{}", current()))
}

fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("{program}: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(if stderr.is_empty() {
            format!("{program} failed")
        } else {
            stderr.to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert!(is_newer("0.5.0", "0.4.1"));
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(!is_newer("0.4.1", "0.4.1"));
        assert!(!is_newer("0.4.0", "0.4.1"));
        assert!(is_newer("1.0", "0.99.99"));
    }
}
