//! 持久调试日志。行格式见前端 debugLog.ts 注释约定。

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 8 * 1024 * 1024;

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(dir: PathBuf) {
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("dsee-hx-debug.log");
    let shown = path.display().to_string();
    if let Ok(mut g) = LOG_PATH.lock() {
        *g = Some(path);
    }
    evt(
        "boot",
        &format!("pid={} file={shown}", std::process::id()),
    );
}

fn log_path() -> Option<PathBuf> {
    LOG_PATH.lock().ok().and_then(|g| g.clone())
}

fn stamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn rotate_if_needed(path: &Path) {
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    if meta.len() <= MAX_BYTES {
        return;
    }
    let Ok(mut f) = fs::File::open(path) else {
        return;
    };
    let mut buf = String::new();
    if f.read_to_string(&mut buf).is_err() {
        return;
    }
    let keep = buf.len() / 2;
    let trimmed = buf.split_at(keep).1;
    let cut = trimmed.find('\n').map(|i| i + 1).unwrap_or(0);
    let body = &trimmed[cut..];
    let _ = fs::write(path, format!("==== log rotated ====\n{body}"));
}

fn write_line(line: &str) {
    let Some(path) = log_path() else {
        return;
    };
    rotate_if_needed(&path);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{line}");
        let _ = f.flush();
    }
}

/// 原样追加（前端已带 ISO 时间戳）
pub fn append(line: &str) {
    let t = line.trim_end();
    if t.is_empty() {
        return;
    }
    write_line(t);
}

/// Rust 侧：`<ms>|rs|<event>|<detail>`
pub fn evt(event: &str, detail: &str) {
    if detail.is_empty() {
        write_line(&format!("{}|rs|{event}", stamp_ms()));
    } else {
        write_line(&format!("{}|rs|{event}|{detail}", stamp_ms()));
    }
}

pub fn read_all() -> Result<String, String> {
    let path = log_path().ok_or_else(|| "debug log not initialized".to_string())?;
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn path_display() -> Option<String> {
    log_path().map(|p| p.display().to_string())
}

pub fn export_to(dest: &Path) -> Result<(), String> {
    let content = read_all()?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(dest, content).map_err(|e| e.to_string())
}
