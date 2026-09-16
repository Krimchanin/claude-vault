use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    id: String,
    cli_session_id: Option<String>,
    title: String,
    source: String,
    turns: Option<u64>,
    modified_at: u64,
    archived: bool,
    relative_path: String,
    size: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResult {
    sessions: Vec<Session>,
    source_path: String,
    archive_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncResult {
    copied: usize,
    updated: usize,
    unchanged: usize,
    total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RestoreResult {
    restored: usize,
    skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    claude_root: String,
    transcript_root: String,
    language: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptMessage {
    role: String,
    content: String,
    timestamp: Option<String>,
}

#[derive(Default)]
struct CopyStats {
    copied: usize,
    updated: usize,
    unchanged: usize,
}

impl CopyStats {
    fn merge(&mut self, other: Self) {
        self.copied += other.copied;
        self.updated += other.updated;
        self.unchanged += other.unchanged;
    }

    fn total(&self) -> usize {
        self.copied + self.updated + self.unchanged
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|p| p.join("settings.json"))
        .map_err(|e| e.to_string())
}

fn detect_claude_root() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        let candidate = PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Claude");
        if candidate.join("claude-code-sessions").is_dir() {
            return Ok(candidate);
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(home) = std::env::var_os("HOME") {
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(home).join(".config"));
        for name in ["Claude", "claude"] {
            let candidate = config_home.join(name);
            if candidate.join("claude-code-sessions").is_dir() {
                return Ok(candidate);
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let packages = PathBuf::from(&local).join("Packages");
        if let Ok(entries) = fs::read_dir(packages) {
            for entry in entries.flatten() {
                let candidate = entry
                    .path()
                    .join("LocalCache")
                    .join("Roaming")
                    .join("Claude");
                if candidate.join("claude-code-sessions").is_dir() {
                    return Ok(candidate);
                }
            }
        }
        let candidate = PathBuf::from(local).join("Claude");
        if candidate.join("claude-code-sessions").is_dir() {
            return Ok(candidate);
        }
    }
    #[cfg(target_os = "windows")]
    if let Some(roaming) = std::env::var_os("APPDATA") {
        let candidate = PathBuf::from(roaming).join("Claude");
        if candidate.join("claude-code-sessions").is_dir() {
            return Ok(candidate);
        }
    }
    Err("Claude App не найден. Укажите папку в настройках".to_string())
}

fn default_transcript_root() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or("Не найдена домашняя папка")?;
    Ok(PathBuf::from(home).join(".claude").join("projects"))
}

fn read_settings(app: &AppHandle) -> Settings {
    if let Ok(raw) = fs::read_to_string(settings_path(app).unwrap_or_default()) {
        if let Ok(value) = serde_json::from_str(&raw) {
            return value;
        }
    }
    Settings {
        claude_root: detect_claude_root()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        transcript_root: default_transcript_root()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        language: "ru".to_string(),
    }
}

fn claude_root(app: &AppHandle) -> Result<PathBuf, String> {
    let configured = read_settings(app).claude_root;
    if configured.is_empty() {
        detect_claude_root()
    } else {
        Ok(PathBuf::from(configured))
    }
}

fn archive_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|p| p.join("vault"))
        .map_err(|e| e.to_string())
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.extension().and_then(|v| v.to_str()) == Some("json")
            && path
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("local_"))
        {
            out.push(path);
        }
    }
}

fn collect_all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn files_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let left_meta = fs::metadata(left).map_err(|e| e.to_string())?;
    let right_meta = fs::metadata(right).map_err(|e| e.to_string())?;
    if left_meta.len() != right_meta.len() {
        return Ok(false);
    }

    let mut left = BufReader::new(fs::File::open(left).map_err(|e| e.to_string())?);
    let mut right = BufReader::new(fs::File::open(right).map_err(|e| e.to_string())?);
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    loop {
        let left_read = left.read(&mut left_buffer).map_err(|e| e.to_string())?;
        let right_read = right.read(&mut right_buffer).map_err(|e| e.to_string())?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn copy_tree(source: &Path, destination: &Path, overwrite: bool) -> Result<CopyStats, String> {
    let mut files = Vec::new();
    collect_all_files(source, &mut files);
    let mut stats = CopyStats::default();
    for file in files {
        let relative = file.strip_prefix(source).map_err(|e| e.to_string())?;
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if !target.exists() {
            fs::copy(&file, &target).map_err(|e| e.to_string())?;
            stats.copied += 1;
        } else if overwrite && !files_equal(&file, &target)? {
            fs::copy(&file, &target).map_err(|e| e.to_string())?;
            stats.updated += 1;
        } else {
            stats.unchanged += 1;
        }
    }
    Ok(stats)
}

fn session_from_file(path: &Path, base: &Path, source: &str, archived: bool) -> Option<Session> {
    let raw = fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&raw).ok()?;
    let relative = path
        .strip_prefix(base)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    let modified_at = fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    let filename = path.file_stem()?.to_string_lossy();
    Some(Session {
        id: json
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or(&filename)
            .to_string(),
        cli_session_id: json
            .get("cliSessionId")
            .and_then(Value::as_str)
            .map(str::to_string),
        title: json
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Без названия")
            .to_string(),
        source: source.to_string(),
        turns: json.get("completedTurns").and_then(Value::as_u64),
        modified_at,
        archived,
        relative_path: relative,
        size: raw.len() as u64,
    })
}

fn transcript_root(app: &AppHandle) -> Result<PathBuf, String> {
    let configured = read_settings(app).transcript_root;
    if configured.is_empty() {
        default_transcript_root()
    } else {
        Ok(PathBuf::from(configured))
    }
}

fn collect_jsonl(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, out);
        } else if path.extension().and_then(|v| v.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
}

fn find_transcript(session_id: &str, app: &AppHandle) -> Option<PathBuf> {
    let mut files = Vec::new();
    collect_jsonl(&transcript_root(app).ok()?, &mut files);
    if let Some(path) = files
        .into_iter()
        .find(|p| p.file_stem().and_then(|v| v.to_str()) == Some(session_id))
    {
        return Some(path);
    }
    let archived = archive_root(app).ok()?.join("transcripts");
    let mut files = Vec::new();
    collect_jsonl(&archived, &mut files);
    files
        .into_iter()
        .find(|p| p.file_stem().and_then(|v| v.to_str()) == Some(session_id))
}

fn text_content(content: &Value) -> String {
    if let Some(text) = content.as_str() {
        return text.to_string();
    }
    content
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    (item.get("type").and_then(Value::as_str) == Some("text"))
                        .then(|| item.get("text").and_then(Value::as_str))
                        .flatten()
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .unwrap_or_default()
}

fn strip_system_blocks(mut text: String) -> String {
    while let Some(start) = text.find("<system-reminder>") {
        let Some(end) = text[start..].find("</system-reminder>") else {
            break;
        };
        text.replace_range(start..start + end + "</system-reminder>".len(), "");
    }
    text.trim().to_string()
}

#[tauri::command]
fn load_transcript(
    app: AppHandle,
    cli_session_id: String,
) -> Result<Vec<TranscriptMessage>, String> {
    let path = find_transcript(&cli_session_id, &app).ok_or("Транскрипт этой сессии не найден")?;
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut messages = Vec::new();
    for line in raw.lines() {
        let Ok(item) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let kind = item.get("type").and_then(Value::as_str).unwrap_or("");
        if kind != "user" && kind != "assistant" {
            continue;
        }
        let Some(message) = item.get("message") else {
            continue;
        };
        let content =
            strip_system_blocks(text_content(message.get("content").unwrap_or(&Value::Null)));
        if content.is_empty() {
            continue;
        }
        messages.push(TranscriptMessage {
            role: message
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or(kind)
                .to_string(),
            content,
            timestamp: item
                .get("timestamp")
                .and_then(Value::as_str)
                .map(str::to_string),
        });
    }
    Ok(messages)
}

fn read_live_sessions(app: &AppHandle) -> Result<(Vec<Session>, PathBuf), String> {
    let base = claude_root(app)?.join("claude-code-sessions");
    let mut files = Vec::new();
    collect_files(&base, &mut files);
    let mut sessions: Vec<_> = files
        .iter()
        .filter_map(|p| session_from_file(p, &base, "Claude App", false))
        .collect();
    sessions.sort_by_key(|s| std::cmp::Reverse(s.modified_at));
    Ok((sessions, base))
}

fn read_archived_sessions(app: &AppHandle) -> Result<Vec<Session>, String> {
    let vault = archive_root(app)?;
    let metadata = vault.join("metadata");
    let base = if metadata.is_dir() { metadata } else { vault };
    let mut files = Vec::new();
    collect_files(&base, &mut files);
    Ok(files
        .iter()
        .filter_map(|p| session_from_file(p, &base, "Хранилище", true))
        .collect())
}

#[tauri::command]
fn scan_sessions(app: AppHandle) -> Result<ScanResult, String> {
    let (live, source) = read_live_sessions(&app)?;
    let archived = read_archived_sessions(&app)?;
    let archived_ids: HashSet<String> = archived.iter().map(|s| s.id.clone()).collect();
    let mut merged: HashMap<String, Session> =
        archived.into_iter().map(|s| (s.id.clone(), s)).collect();
    for mut item in live {
        item.archived = archived_ids.contains(item.id.as_str());
        merged.insert(item.id.clone(), item);
    }
    let mut sessions: Vec<_> = merged.into_values().collect();
    sessions.sort_by_key(|s| std::cmp::Reverse(s.modified_at));
    Ok(ScanResult {
        sessions,
        source_path: source.to_string_lossy().to_string(),
        archive_path: archive_root(&app)?.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn sync_cache(app: AppHandle) -> Result<SyncResult, String> {
    let root = claude_root(&app)?.join("claude-code-sessions");
    let destination = archive_root(&app)?;
    fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
    let mut stats = copy_tree(&root, &destination.join("metadata"), true)?;
    let transcripts = transcript_root(&app)?;
    stats.merge(copy_tree(
        &transcripts,
        &destination.join("transcripts"),
        true,
    )?);
    let workspaces = claude_root(&app)?.join("scratch-workspaces");
    stats.merge(copy_tree(
        &workspaces,
        &destination.join("workspaces"),
        true,
    )?);
    Ok(SyncResult {
        copied: stats.copied,
        updated: stats.updated,
        unchanged: stats.unchanged,
        total: stats.total(),
    })
}

#[tauri::command]
fn restore_missing(app: AppHandle) -> Result<RestoreResult, String> {
    let source = archive_root(&app)?;
    let target_root = claude_root(&app)?.join("claude-code-sessions");
    let metadata = copy_tree(&source.join("metadata"), &target_root, false)?;
    let (mut restored, mut skipped) = (metadata.copied, metadata.unchanged);
    let transcript_source = source.join("transcripts");
    let transcript_target = transcript_root(&app)?;
    let counts = copy_tree(&transcript_source, &transcript_target, false)?;
    restored += counts.copied;
    skipped += counts.unchanged;
    let workspace_target = claude_root(&app)?.join("scratch-workspaces");
    let counts = copy_tree(&source.join("workspaces"), &workspace_target, false)?;
    restored += counts.copied;
    skipped += counts.unchanged;
    Ok(RestoreResult { restored, skipped })
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    read_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan_sessions,
            sync_cache,
            restore_missing,
            load_transcript,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_system_reminders_without_touching_chat_text() {
        let input = "Before\n<system-reminder>private context</system-reminder>\nAfter";
        assert_eq!(strip_system_blocks(input.to_string()), "Before\n\nAfter");
    }

    #[test]
    fn extracts_text_blocks_only() {
        let content = serde_json::json!([
            {"type": "text", "text": "First"},
            {"type": "tool_use", "name": "Read"},
            {"type": "text", "text": "Second"}
        ]);
        assert_eq!(text_content(&content), "First\n\nSecond");
    }

    #[test]
    fn compares_files_by_content() {
        let root = std::env::temp_dir().join(format!("claude-vault-test-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create test directory");
        let left = root.join("left");
        let right = root.join("right");
        fs::write(&left, b"same content").expect("write left file");
        fs::write(&right, b"same content").expect("write right file");
        assert!(files_equal(&left, &right).expect("compare equal files"));
        fs::write(&right, b"different").expect("rewrite right file");
        assert!(!files_equal(&left, &right).expect("compare different files"));
        let _ = fs::remove_dir_all(root);
    }
}
