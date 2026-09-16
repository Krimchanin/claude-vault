//! Local account snapshots, separate from the chat archive. Never parse credentials.
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
pub(crate) static PROFILE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountState {
    pub active: Option<String>,
    pub source: Option<PathBuf>,
}

fn read_state(base: &Path) -> Result<AccountState, String> {
    let file = base.join("state.json");
    if !file.exists() {
        return Ok(AccountState::default());
    }
    let state: AccountState = serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if state.active.as_deref().is_some_and(|id| !valid_id(id))
        || state
            .source
            .as_ref()
            .is_some_and(|path| !path.is_absolute() || path.parent().is_none())
    {
        return Err("Invalid account state".into());
    }
    Ok(state)
}

fn write_state(base: &Path, state: &AccountState) -> Result<(), String> {
    fs::write(
        base.join("state.next.json"),
        serde_json::to_vec(state).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let file = base.join("state.json");
    let previous = base.join(format!("state-{}.json", unique_id()?));
    if file.exists() {
        fs::rename(&file, &previous).map_err(|e| e.to_string())?;
    }
    if let Err(error) = fs::rename(base.join("state.next.json"), &file) {
        if previous.exists() {
            let _ = fs::rename(previous, file);
        }
        return Err(error.to_string());
    }
    Ok(())
}

fn unique_id() -> Result<String, String> {
    Ok(format!(
        "{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ))
}

fn require_closed() -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    return Err("Account switching is currently supported only on Windows. macOS Keychain switching is not yet verified.".into());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let output = Command::new("tasklist.exe")
            .args(["/FI", "IMAGENAME eq Claude.exe", "/FO", "CSV", "/NH"])
            .creation_flags(0x08000000)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("Unable to verify that Claude is closed".into());
        }
        if String::from_utf8_lossy(&output.stdout)
            .to_ascii_lowercase()
            .contains("\"claude.exe\"")
        {
            return Err("CLAUDE_RUNNING".into());
        }
        Ok(())
    }
}

fn copy_snapshot(source: &Path, target: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(source).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("Account profiles cannot contain symbolic links".into());
    }
    fs::create_dir(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err("Account profiles cannot contain symbolic links".into());
        }
        let destination = target.join(entry.file_name());
        if kind.is_dir() {
            copy_snapshot(&entry.path(), &destination)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination).map_err(|e| e.to_string())?;
        } else {
            return Err("Unsupported file in account profile".into());
        }
    }
    Ok(())
}

fn replace_snapshot(source: &Path, saved: &Path) -> Result<(), String> {
    let staging = saved.with_extension(format!("next-{}", unique_id()?));
    copy_snapshot(source, &staging)?;
    if saved.exists() {
        fs::rename(
            saved,
            saved.with_extension(format!("previous-{}", unique_id()?)),
        )
        .map_err(|e| e.to_string())?;
    }
    fs::rename(staging, saved).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_account_state(app: AppHandle) -> Result<serde_json::Value, String> {
    let base = root(&app)?;
    let state = read_state(&base)?;
    Ok(
        serde_json::json!({"active": state.active, "recovery_required": base.join("switch-recovery.json").exists()}),
    )
}

fn root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|p| p.join("account-profiles"))
        .map_err(|e| e.to_string())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 40 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

#[tauri::command]
pub fn list_profiles(app: AppHandle) -> Result<Vec<Profile>, String> {
    let base = root(&app)?;
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut profiles = Vec::new();
    for entry in fs::read_dir(base).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !valid_id(&id) {
            continue;
        }
        let metadata = entry.path().join("profile.json");
        if !metadata.exists() {
            continue;
        }
        let raw = fs::read_to_string(metadata).map_err(|e| e.to_string())?;
        let profile: Profile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        if profile.id == id {
            profiles.push(profile);
        }
    }
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

#[tauri::command]
pub fn create_profile(app: AppHandle, name: String, capture: bool) -> Result<Profile, String> {
    let _lock = PROFILE_LOCK.lock().map_err(|e| e.to_string())?;
    require_closed()?;
    if root(&app)?.join("switch-recovery.json").exists() {
        return Err("RECOVERY_REQUIRED".into());
    }
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 40 || name.chars().any(char::is_control) {
        return Err("Profile name must contain 1–40 printable characters".into());
    }
    if list_profiles(app.clone())?
        .iter()
        .any(|p| p.name.to_lowercase() == name.to_lowercase())
    {
        return Err("A profile with this name already exists".into());
    }
    let id = unique_id()?;
    let profile = Profile {
        id,
        name: name.to_owned(),
    };
    let directory = root(&app)?.join(&profile.id);
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let base = root(&app)?;
    let mut captured = None;
    if capture {
        if read_state(&base)?.active.is_some() {
            return Err("Current account is already saved".into());
        }
        let live = super::claude_root(&app)?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let profile_base = base.canonicalize().map_err(|e| e.to_string())?;
        if profile_base.starts_with(&live) || live.starts_with(&profile_base) {
            return Err("Account storage and Claude source must not overlap".into());
        }
        copy_snapshot(&live, &directory.join("data"))?;
        captured = Some(live);
    } else {
        if read_state(&base)?.active.is_none() {
            return Err("Save your current account first".into());
        }
        fs::create_dir(directory.join("data")).map_err(|e| e.to_string())?;
    }
    fs::write(
        directory.join("profile.json"),
        serde_json::to_vec_pretty(&profile).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if let Some(live) = captured {
        write_state(
            &base,
            &AccountState {
                active: Some(profile.id.clone()),
                source: Some(live),
            },
        )?;
    }
    Ok(profile)
}

fn find_executable(directory: &Path, depth: usize) -> Option<PathBuf> {
    let entries = fs::read_dir(directory).ok()?;
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let kind = entry.file_type().ok()?;
        if kind.is_symlink() {
            continue;
        }
        let path = entry.path();
        if kind.is_file()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("claude.exe")
        {
            return Some(path);
        }
        if kind.is_dir() && depth > 0 {
            if let Some(exe) = find_executable(&path, depth - 1) {
                candidates.push(exe);
            }
        }
    }
    candidates.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
    candidates.pop()
}

fn detect_executable() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // Constant, read-only query; user input is never interpolated into a shell.
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Get-AppxPackage -Name Claude | Select-Object -First 1 -ExpandProperty InstallLocation"])
            .creation_flags(0x08000000).output().map_err(|e| e.to_string())?;
        if output.status.success() {
            let install = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !install.is_empty() {
                if let Some(exe) = find_executable(Path::new(&install), 2) {
                    return Ok(exe);
                }
            }
        }
        for (variable, suffix) in [
            ("LOCALAPPDATA", "AnthropicClaude"),
            ("LOCALAPPDATA", "Programs/Claude"),
            ("ProgramFiles", "Claude"),
            ("ProgramFiles(x86)", "Claude"),
        ] {
            if let Some(base) = std::env::var_os(variable) {
                if let Some(exe) = find_executable(&PathBuf::from(base).join(suffix), 2) {
                    return Ok(exe);
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let mut candidates = vec![PathBuf::from(
            "/Applications/Claude.app/Contents/MacOS/Claude",
        )];
        if let Some(home) = std::env::var_os("HOME") {
            candidates
                .push(PathBuf::from(home).join("Applications/Claude.app/Contents/MacOS/Claude"));
        }
        if let Some(path) = candidates.into_iter().find(|p| p.is_file()) {
            return Ok(path);
        }
    }
    Err("Claude executable not found. Specify its full path in Accounts.".into())
}

#[tauri::command]
pub fn open_claude(app: AppHandle, executable: String) -> Result<(), String> {
    let _lock = PROFILE_LOCK.lock().map_err(|e| e.to_string())?;
    let base = root(&app)?;
    if base.join("switch-recovery.json").exists() {
        return Err("RECOVERY_REQUIRED".into());
    }
    let preference = base.join("executable.txt");
    let manual = if executable.trim().is_empty() {
        fs::read_to_string(&preference).unwrap_or_default()
    } else {
        executable.trim().to_owned()
    };
    let exe = if manual.is_empty() {
        detect_executable()?
    } else {
        PathBuf::from(&manual)
    };
    if !exe.is_file() {
        return Err("Claude executable does not exist. Update its path in Accounts.".into());
    }
    // Preserve MSIX package identity for default-profile launches.
    #[cfg(target_os = "windows")]
    if manual.is_empty() && exe.to_string_lossy().contains("WindowsApps") {
        use std::os::windows::process::CommandExt;
        let output = Command::new("powershell.exe").args(["-NoProfile", "-NonInteractive", "-Command", "Get-AppxPackage -Name Claude | Select-Object -First 1 -ExpandProperty PackageFamilyName"]).creation_flags(0x08000000).output().map_err(|e| e.to_string())?;
        let family = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !output.status.success() || family.is_empty() {
            return Err("Cannot resolve Claude package identity".into());
        }
        Command::new("explorer.exe")
            .arg(format!("shell:AppsFolder/{family}!Claude").replace("AppsFolder/", "AppsFolder\\"))
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    Command::new(&exe).spawn().map_err(|e| e.to_string())?;
    if !manual.is_empty() {
        fs::write(preference, manual).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn switch_profile(app: AppHandle, id: String) -> Result<(), String> {
    let _lock = PROFILE_LOCK.lock().map_err(|e| e.to_string())?;
    require_closed()?;
    let base = root(&app)?;
    if base.join("switch-recovery.json").exists() {
        return Err("RECOVERY_REQUIRED".into());
    }
    if !valid_id(&id) || !list_profiles(app.clone())?.iter().any(|p| p.id == id) {
        return Err("Unknown profile".into());
    }
    let mut state = read_state(&base)?;
    let active = state
        .active
        .as_deref()
        .ok_or("Save your current account first")?;
    if active == id {
        return Ok(());
    }
    let live = state.source.clone().ok_or("Missing original Claude path")?;
    let configured = super::read_settings(&app).claude_root;
    if !configured.is_empty()
        && live
            != PathBuf::from(configured)
                .canonicalize()
                .map_err(|e| e.to_string())?
    {
        return Err("Claude data path changed. Restore the original path in Settings.".into());
    }
    replace_snapshot(&live, &base.join(active).join("data"))?;
    let parent = live.parent().ok_or("Invalid Claude path")?;
    let stamp = unique_id()?;
    let staging = parent.join(format!("Claude-vault-staging-{stamp}"));
    let recovery = parent.join(format!("Claude-vault-recovery-{stamp}"));
    copy_snapshot(&base.join(&id).join("data"), &staging)?;
    require_closed()?;
    let journal = Recovery {
        live: live.clone(),
        backup: recovery.clone(),
        state: read_state(&base)?,
    };
    fs::write(
        base.join("switch-recovery.json"),
        serde_json::to_vec(&journal).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&live, &recovery).map_err(|e| e.to_string())?;
    if let Err(error) = fs::rename(&staging, &live) {
        let _ = fs::rename(&recovery, &live);
        return Err(format!("Switch failed; recovery available: {error}"));
    }
    state.active = Some(id);
    write_state(&base, &state)?;
    fs::rename(
        base.join("switch-recovery.json"),
        base.join(format!("completed-switch-{stamp}.json")),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Recovery {
    live: PathBuf,
    backup: PathBuf,
    state: AccountState,
}

#[tauri::command]
pub fn recover_account(app: AppHandle) -> Result<(), String> {
    let _lock = PROFILE_LOCK.lock().map_err(|e| e.to_string())?;
    require_closed()?;
    let base = root(&app)?;
    let file = base.join("switch-recovery.json");
    let journal: Recovery = serde_json::from_slice(&fs::read(&file).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if journal.state.source.as_ref() != Some(&journal.live)
        || !journal.live.is_absolute()
        || journal.live.parent().is_none()
        || !journal.state.active.as_deref().is_some_and(valid_id)
        || journal.backup.parent() != journal.live.parent()
        || !journal
            .backup
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("Claude-vault-recovery-"))
    {
        return Err("Invalid recovery journal".into());
    }
    if !journal.backup.exists() && !journal.live.exists() {
        return Err("Recovery data is missing; no files were changed".into());
    }
    if journal.backup.exists() {
        if journal.live.exists() {
            fs::rename(
                &journal.live,
                journal
                    .live
                    .with_extension(format!("retained-{}", unique_id()?)),
            )
            .map_err(|e| e.to_string())?;
        }
        fs::rename(journal.backup, journal.live).map_err(|e| e.to_string())?;
    }
    write_state(&base, &journal.state)?;
    fs::rename(
        file,
        base.join(format!("recovered-switch-{}.json", unique_id()?)),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_path_traversal_and_arbitrary_paths() {
        for id in ["../x", "", "C:/Claude", "abc/def", "default"] {
            assert!(!valid_id(id));
        }
        assert!(valid_id("a012bcdef"));
    }

    #[test]
    fn snapshots_preserve_binary_and_nested_data() {
        let directory =
            std::env::temp_dir().join(format!("vault-snapshot-{}", unique_id().unwrap()));
        let source = directory.join("source");
        let target = directory.join("saved");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/history"), [0, 255, 1, 42]).unwrap();
        copy_snapshot(&source, &target).unwrap();
        assert_eq!(
            fs::read(target.join("nested/history")).unwrap(),
            [0, 255, 1, 42]
        );
        fs::write(source.join("nested/history"), b"updated").unwrap();
        replace_snapshot(&source, &target).unwrap();
        assert_eq!(fs::read(target.join("nested/history")).unwrap(), b"updated");
        assert!(fs::read_dir(&directory).unwrap().flatten().any(|e| e
            .file_name()
            .to_string_lossy()
            .starts_with("saved.previous-")));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn account_state_updates_keep_previous_state() {
        let directory = std::env::temp_dir().join(format!("vault-state-{}", unique_id().unwrap()));
        fs::create_dir(&directory).unwrap();
        write_state(
            &directory,
            &AccountState {
                active: Some("abc".into()),
                source: None,
            },
        )
        .unwrap();
        write_state(
            &directory,
            &AccountState {
                active: Some("def".into()),
                source: None,
            },
        )
        .unwrap();
        assert_eq!(
            read_state(&directory).unwrap().active.as_deref(),
            Some("def")
        );
        assert!(fs::read_dir(&directory).unwrap().count() >= 2);
        fs::remove_dir_all(directory).unwrap();
    }
}
