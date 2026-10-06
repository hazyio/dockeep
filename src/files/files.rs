use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, time::SystemTime};

use crate::config::ImageFormat;
use crate::config::project_settings_data::ProjectSettingsData;
use anyhow::{Error, Result};
use img_parts::{
    Bytes,
    jpeg::{Jpeg, JpegSegment, markers},
    png::{Png, PngChunk},
    riff::{RiffChunk, RiffContent},
    webp::{CHUNK_XMP, WebP},
};
use regex::Regex;
use walkdir::WalkDir;
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
pub struct SaveScreenshotResult {
    pub saved: bool,
    pub is_replaced: bool,
    pub save_path: PathBuf,
    pub path_id: PathBuf,
    pub error: Option<String>,
}
pub fn last_modified(path: &PathBuf) -> std::io::Result<SystemTime> {
    fs::metadata(path)?.modified()
}
const KEY: &str = "dockeep_capture_url";
pub fn rename_file(path: &PathBuf, new_name: &str) -> Result<PathBuf> {
    let parent = path.parent().unwrap_or(Path::new(""));
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let new_path = parent.join(format!("{new_name}.{ext}"));
    if new_path.exists() {
        return Err(anyhow::anyhow!("File already exists: {:?}", new_path));
    }
    fs::rename(path, &new_path)?;
    Ok(new_path)
}
pub fn file_name_with_extension(path: &PathBuf) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}
pub fn file_name_without_extension(path: &PathBuf) -> String {
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{size:.1} {}", UNITS[unit])
}
pub fn read_capture_url(data: &[u8]) -> Option<String> {
    let bytes = Bytes::copy_from_slice(data);

    if data.starts_with(&[0x89, b'P', b'N', b'G']) {
        let png = Png::from_bytes(bytes).ok()?;
        // There can be several tEXt chunks, so scan them all
        png.chunks()
            .iter()
            .filter(|c| c.kind() == *b"tEXt")
            .find_map(|c| {
                let contents = c.contents();
                let nul = contents.iter().position(|&b| b == 0)?;
                let (k, v) = (&contents[..nul], &contents[nul + 1..]);
                (k == KEY.as_bytes()).then(|| String::from_utf8_lossy(v).into_owned())
            })
    } else if data.starts_with(&[0xFF, 0xD8]) {
        let jpeg = Jpeg::from_bytes(bytes).ok()?;
        let prefix = format!("{KEY}=");
        jpeg.segments()
            .iter()
            .filter(|s| s.marker() == markers::COM)
            .find_map(|s| {
                let text = String::from_utf8_lossy(s.contents());
                text.strip_prefix(&prefix).map(str::to_owned)
            })
    } else if data.len() > 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        let webp = WebP::from_bytes(bytes).ok()?;
        let chunk = webp.chunk_by_id(*b"XMP ")?;
        let xmp = String::from_utf8_lossy(chunk.content().data()?);
        let open = format!("<{KEY}>");
        let close = format!("</{KEY}>");
        let start = xmp.find(&open)? + open.len();
        let end = xmp[start..].find(&close)? + start;
        Some(xmp[start..end].to_owned())
    } else {
        None
    }
}
pub fn embed_capture_url(
    encoded: Vec<u8>,
    format: ImageFormat,
    value: &str,
) -> Result<Vec<u8>, img_parts::Error> {
    let out = match format {
        ImageFormat::Png => {
            let mut png = Png::from_bytes(Bytes::from(encoded))?;
            // tEXt = keyword, NUL, text
            let mut payload = KEY.as_bytes().to_vec();
            payload.push(0);
            payload.extend_from_slice(value.as_bytes());

            let chunk = PngChunk::new(*b"tEXt", Bytes::from(payload));
            let idx = png.chunks().len().saturating_sub(1); // before IEND
            png.chunks_mut().insert(idx, chunk);
            png.encoder().bytes().to_vec()
        }
        ImageFormat::Jpeg => {
            let mut jpeg = Jpeg::from_bytes(Bytes::from(encoded))?;
            let comment = format!("{KEY}={value}");
            let seg = JpegSegment::new_with_contents(markers::COM, Bytes::from(comment));
            let idx = jpeg
                .segments()
                .iter()
                .position(|s| s.marker() == markers::SOS)
                .unwrap_or(0);
            jpeg.segments_mut().insert(idx, seg);
            jpeg.encoder().bytes().to_vec()
        }
        ImageFormat::Webp => {
            let mut webp = WebP::from_bytes(Bytes::from(encoded))?;
            let xmp = format!(
                r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><{KEY}>{value}</{KEY}></x:xmpmeta>"#
            );
            webp.chunks_mut().push(RiffChunk::new(
                CHUNK_XMP,
                RiffContent::Data(Bytes::from(xmp)),
            ));
            webp.encoder().bytes().to_vec()
        }
    };
    Ok(out)
}
pub fn save_screenshot(
    data: Result<Vec<u8>, Error>,
    replace_path: Option<PathBuf>,
    working_dir: PathBuf,
    capture_url: &str,
    tab_title: &str,
) -> SaveScreenshotResult {
    let project_settings = ProjectSettingsData::load(&working_dir);
    let tab_title = tab_title.replace("/", "-").replace("\\", "-");
    let save_format = project_settings.save_format;
    let (new_save_path, replace_path) = match replace_path {
        Some(rp) => {
            if rp.extension().unwrap_or_default() != save_format.to_value() {
                // if the extension doesn't match, replace it with the save format
                let new_path = rp.with_extension(save_format.to_value());
                (new_path, Some(rp))
            } else {
                (rp.clone(), Some(rp))
            }
        }
        None => {
            let working_dir = working_dir.clone();
            let filename = if project_settings.save_with_tab_title && !tab_title.is_empty() {
                format!("{}", tab_title.replace(" ", "-"))
            } else {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                format!("Screenshot_{}", timestamp)
            };
            let mut new_path = working_dir
                .join(project_settings.save_to_dir)
                .join(filename);
            new_path.add_extension(save_format.to_value());
            (new_path, None)
        }
    };
    if new_save_path.exists() && replace_path.is_none() {
        tracing::error!(
            "An existing file already exists at the save path: {:?}",
            new_save_path
        );
        // if replace_path is None, this is a new save, this means it is a new file that should not exist yet so we exist.
        return SaveScreenshotResult {
            saved: false,
            is_replaced: replace_path.is_some(),
            save_path: new_save_path.clone(),
            path_id: replace_path.unwrap_or(new_save_path),
            error: Some(t!("error.cannot_save_to_existing_file").to_string()),
        };
    }
    match data {
        Ok(data) => {
            let try_embed = match embed_capture_url(data.clone(), save_format, capture_url) {
                Ok(embedded) => embedded,
                Err(e) => {
                    tracing::warn!("Failed to embed capture URL: {}", e);
                    data
                }
            };
            if let Err(e) = std::fs::write(&new_save_path, &try_embed) {
                tracing::error!("Failed to save screenshot to {:?}: {}", new_save_path, e);
                // If is_replace is Some and the save_path is different from the old path, this means the extension was changed, so remove the created file
                if replace_path.is_some() && new_save_path != replace_path.clone().unwrap() {
                    // remove the created file
                    std::fs::remove_file(&new_save_path).ok();
                }
                return SaveScreenshotResult {
                    saved: false,
                    is_replaced: replace_path.is_some(),
                    save_path: new_save_path.clone(),
                    path_id: replace_path.unwrap_or(new_save_path),
                    error: Some(t!("error.failed_to_save_screenshot").to_string()),
                };
            }

            if replace_path.is_some() {
                let old_path = replace_path.clone().unwrap();
                // remove the old file if the extension was changed
                if old_path != new_save_path {
                    // remove the old file,
                    std::fs::remove_file(&old_path).ok();
                }
            }

            SaveScreenshotResult {
                saved: true,
                is_replaced: replace_path.is_some(),
                save_path: new_save_path.clone(),
                path_id: replace_path.unwrap_or(new_save_path),
                error: None,
            }
        }
        Err(e) => {
            tracing::error!("Failed to capture screenshot: {}", e);
            SaveScreenshotResult {
                saved: false,
                is_replaced: replace_path.is_some(),
                save_path: new_save_path.clone(),
                path_id: replace_path.unwrap_or(new_save_path),
                error: Some(t!("error.failed_to_save_screenshot").to_string()),
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
pub(super) fn has_image_header(path: &Path) -> bool {
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
pub(super) fn load_ignore_patterns(root: &Path) -> Vec<String> {
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
pub(super) fn is_ignored(path: &Path, patterns: &[String]) -> bool {
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
pub(super) fn glob_matches(pattern: &str, text: &str) -> bool {
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
pub(super) fn glob_to_regex(pattern: &str) -> String {
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
