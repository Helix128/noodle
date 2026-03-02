use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use regex::Regex;

use crate::ndl::debug::log;
use crate::ndl::files::{LIVE_PATH, SOURCE_PATH};
use crate::ndl::utils::image;

pub type ProcessingLocks = Arc<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>>;

pub fn new_locks() -> ProcessingLocks {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn live_to_source_path(live: &Path) -> Option<PathBuf> {
    let live_base = Path::new(LIVE_PATH);
    let rel = live.strip_prefix(live_base).ok()?;
    let source_base = Path::new(SOURCE_PATH);

    match live.extension().and_then(|e| e.to_str()) {
        Some("avif") => {
            let candidate = source_base.join(rel).with_extension("png");
            if candidate.exists() { Some(candidate) } else { None }
        }
        Some("png") => None,
        _ => Some(source_base.join(rel)),
    }
}

fn is_stale(source: &Path, live: &Path) -> bool {
    if !live.exists() {
        return true;
    }
    let src_mtime = match std::fs::metadata(source).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let live_mtime = match std::fs::metadata(live).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    src_mtime > live_mtime
}

fn process_file(source: &Path, live: &Path) -> io::Result<()> {
    if let Some(parent) = live.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match live.extension().and_then(|e| e.to_str()) {
        Some("avif") => {
            let src = source
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid source path"))?;
            let dst = live
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid live path"))?;
            image::to_avif(src, dst)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        }
        Some("html") => {
            process_html(source, live)?;
        }
        _ => {
            std::fs::copy(source, live)?;
        }
    }

    Ok(())
}

fn process_html(source: &Path, live: &Path) -> io::Result<()> {
    let content = std::fs::read_to_string(source)?;
    let mut updated = content.clone();

    let live_base = Path::new(LIVE_PATH);
    let source_base = Path::new(SOURCE_PATH);
    let source_dir = source.parent().unwrap_or(Path::new(""));
    let live_dir = live_base.join(source_dir.strip_prefix(source_base).unwrap_or(source_dir));

    let patterns = [
        r#"src=["']([^"']+\.png)["']"#,
        r#"href=["']([^"']+\.png)["']"#,
        r#"url\(['"]*([^)'"]+\.png)['"]*\)"#,
    ];

    for pattern in patterns {
        let re = Regex::new(pattern)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let snapshot = updated.clone();
        for cap in re.captures_iter(&snapshot) {
            if let Some(img_match) = cap.get(1) {
                let img_path = img_match.as_str();
                if is_local_path(img_path) {
                    let source_img = if img_path.starts_with('/') {
                        source_base.join(&img_path[1..])
                    } else {
                        source_dir.join(img_path)
                    };
                    if source_img.exists() {
                        let live_avif = if img_path.starts_with('/') {
                            live_base.join(&img_path[1..]).with_extension("avif")
                        } else {
                            live_dir.join(img_path).with_extension("avif")
                        };
                        if let Err(e) = process_file(&source_img, &live_avif) {
                            log::error(&format!(
                                "Failed to convert '{}' while processing HTML: {}",
                                source_img.display(), e
                            ));
                        }
                        let new_ref = img_path.replace(".png", ".avif");
                        let full = cap.get(0).unwrap().as_str();
                        let new_full = full.replace(img_path, &new_ref);
                        updated = updated.replace(full, &new_full);
                    }
                }
            }
        }
    }

    std::fs::write(live, &updated)?;
    log::info(&format!("Processed HTML: '{}'", live.display()));
    Ok(())
}

fn is_local_path(path: &str) -> bool {
    if path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("//")
        || path.starts_with("data:")
    {
        return false;
    }
    let mut chars = path.chars();
    if let (Some(a), Some(b)) = (chars.next(), chars.next()) {
        if b == ':' && a.is_ascii_alphabetic() {
            return false;
        }
    }
    true
}

pub fn ensure_up_to_date(live_path: &Path, locks: &ProcessingLocks) -> io::Result<()> {
    let source_path = match live_to_source_path(live_path) {
        Some(p) => p,
        None => return Ok(()),
    };

    if !source_path.exists() {
        return Ok(());
    }

    if !is_stale(&source_path, live_path) {
        return Ok(());
    }

    let file_lock = {
        let mut map = locks
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "lock poisoned"))?;
        map.entry(live_path.to_path_buf())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };

    let _guard = file_lock
        .lock()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "file lock poisoned"))?;

    if !is_stale(&source_path, live_path) {
        return Ok(());
    }

    process_file(&source_path, live_path)
}
