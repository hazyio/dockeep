use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, time::SystemTime};

use crate::config::{AppConfig, ImageFormat};
use anyhow::{Error, Result};
use img_parts::{
    Bytes,
    jpeg::{Jpeg, JpegSegment, markers},
    png::{Png, PngChunk},
    webp::{WebP, WebPChunk},
};
use regex::Regex;
use walkdir::WalkDir;
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
pub struct SaveScreenshotResult {
    pub saved: bool,
    pub is_replaced: bool,
    pub save_path: PathBuf,
}
pub fn last_modified(path: &PathBuf) -> std::io::Result<SystemTime> {
    fs::metadata(path)?.modified()
}

pub fn embed_capture_url(
    encoded: Vec<u8>,
    format: ImageFormat,
    value: &str,
) -> Result<Vec<u8>, img_parts::Error> {
    let mut out = Vec::new();
    let key = "dockeep_capture_url";
    match format {
        ImageFormat::Png => {
            let mut png = Png::from_bytes(Bytes::from(encoded))?;
            // tEXt = keyword, NUL, text
            let mut payload = key.as_bytes().to_vec();
            payload.push(0);
            payload.extend_from_slice(value.as_bytes());

            let chunk = PngChunk::new(*b"tEXt", Bytes::from(payload));
            let idx = png.chunks().len().saturating_sub(1); // before IEND
            png.chunks_mut().insert(idx, chunk);
            png.encoder().write_to(&mut out)?;
        }
        ImageFormat::Jpeg => {
            let mut jpeg = Jpeg::from_bytes(Bytes::from(encoded))?;
            let comment = format!("{key}={value}");
            let seg = JpegSegment::new_with_contents(markers::COM, Bytes::from(comment));
            let idx = jpeg
                .segments()
                .iter()
                .position(|s| s.marker() == markers::SOS)
                .unwrap_or(0);
            jpeg.segments_mut().insert(idx, seg);
            jpeg.encoder().write_to(&mut out)?;
        }
        ImageFormat::Webp => {
            let mut webp = WebP::from_bytes(Bytes::from(encoded))?;
            let xmp = format!(
                r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><{key}>{value}</{key}></x:xmpmeta>"#
            );
            webp.chunks_mut()
                .push(WebPChunk::new(*b"XMP ", Bytes::from(xmp)));
            webp.encoder().write_to(&mut out)?;
        }
    }
    Ok(out)
}
pub fn save_screenshot(
    data: Result<Vec<u8>, Error>,
    replace_path: Option<PathBuf>,
    working_dir: PathBuf,
) -> SaveScreenshotResult {
    let save_format = AppConfig::load().capture_setting.image_format;

    let (save_path, is_replace) = match replace_path {
        Some(rp) => (rp, true),
        None => {
            let working_dir = working_dir.clone();

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("Screenshot_{}.{}", timestamp, save_format.to_value());
            (working_dir.join(&filename), false)
        }
    };
    match data {
        Ok(data) => {
            if let Err(e) = std::fs::write(&save_path, &data) {
                tracing::error!("Failed to save screenshot to {:?}: {}", save_path, e);
                if is_replace {
                    // remove the created file
                    std::fs::remove_file(&save_path).ok();
                }
                return SaveScreenshotResult {
                    saved: false,
                    is_replaced: is_replace,
                    save_path,
                };
            }

            SaveScreenshotResult {
                saved: true,
                is_replaced: is_replace,
                save_path,
            }
        }
        Err(e) => {
            tracing::error!("Failed to capture screenshot: {}", e);
            SaveScreenshotResult {
                saved: false,
                is_replaced: is_replace,
                save_path,
            }
        }
    }
}
pub fn read_images(path: &PathBuf) -> Vec<PathBuf> {
    read_dir(path)
        .into_iter()
        .filter(|p| {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            let Some(ext) = ext else { return false };
            if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                return false;
            }

            if has_image_header(p) {
                true
            } else {
                tracing::info!(
                    "Skipping '{}': extension '.{}' suggests image but file header disagrees",
                    p.display(),
                    ext,
                );
                false
            }
        })
        .collect()
}

/// Reads the first bytes of `path` and checks for a known image magic number.
fn has_image_header(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };

    // 12 bytes covers all non-SVG magic numbers; SVG needs a larger window.
    let mut buf = [0u8; 512];
    let n = file.read(&mut buf).unwrap_or(0);
    let buf = &buf[..n];

    // JPEG: FF D8 FF
    if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return true;
    }
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if buf.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return true;
    }
    // GIF87a / GIF89a
    if buf.starts_with(b"GIF87a") || buf.starts_with(b"GIF89a") {
        return true;
    }
    // WebP: RIFF????WEBP
    if n >= 12 && buf.starts_with(b"RIFF") && &buf[8..12] == b"WEBP" {
        return true;
    }

    false
}

pub fn read_dir(path: &PathBuf) -> Vec<PathBuf> {
    let root = path;
    let ignore_patterns = load_ignore_patterns(&root);

    WalkDir::new(&root)
        .into_iter()
        .filter_entry(|entry| {
            // Always descend into the root itself
            if entry.depth() == 0 {
                return true;
            }
            let relative = entry.path().strip_prefix(&root).unwrap_or(entry.path());
            !is_ignored(relative, &ignore_patterns)
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().is_dir() {
                return None;
            }
            tracing::debug!("{}", entry.path().display());
            Some(entry.path().to_path_buf())
        })
        .collect()
}

/// Loads ignore patterns from `.dockeepignore` (preferred) or `.gitignore`.
fn load_ignore_patterns(root: &Path) -> Vec<String> {
    let dockeepignore = root.join(".dockeepignore");
    let gitignore = root.join(".gitignore");

    let ignore_file = if dockeepignore.exists() {
        Some(dockeepignore)
    } else if gitignore.exists() {
        Some(gitignore)
    } else {
        None
    };

    ignore_file
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map(|content| {
            content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Returns `true` when `path` (relative to the project root) matches any ignore pattern.
fn is_ignored(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    // Also check just the file/dir name for simple patterns like "*.log" or "node_modules"
    let name_str = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    patterns
        .iter()
        .any(|pattern| glob_matches(pattern, &path_str) || glob_matches(pattern, &name_str))
}

/// Converts a gitignore-style glob pattern into a [`Regex`] and tests it against `text`.
fn glob_matches(pattern: &str, text: &str) -> bool {
    // Strip a trailing slash that marks directory-only patterns; we still match
    // the name so that the WalkDir filter_entry can prune the whole subtree.
    let pattern = pattern.trim_end_matches('/');

    // Strip a leading slash that anchors to the repo root (we already work with
    // root-relative paths, so the anchor is redundant here).
    let pattern = pattern.trim_start_matches('/');

    if pattern.is_empty() {
        return false;
    }

    let regex_src = glob_to_regex(pattern);
    Regex::new(&regex_src)
        .map(|re| re.is_match(text.as_ref()))
        .unwrap_or(false)
}

/// Converts a gitignore glob pattern to a regex string.
///
/// Rules:
/// - `**`  → matches any sequence of characters including `/`
/// - `*`   → matches any sequence of characters except `/`
/// - `?`   → matches any single character except `/`
/// - All other regex meta-characters are escaped.
fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::from("(?i)^");
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume second '*'
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => regex.push_str("[^/]"),
            // Escape regex meta-characters
            '.' | '+' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            other => regex.push(other),
        }
    }

    regex.push('$');
    regex
}
