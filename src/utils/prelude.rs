use std::ops::Div;
use std::process::Command;
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use gpui_kit::component::Root;
use gpui_kit::{
    App, AppContext, SharedString, TitlebarOptions, Window, WindowBounds, WindowDecorations,
    WindowOptions,
};

use crate::scenes::settings::view::{SettingDefaultOpen, SettingsPage};
use crate::utils::date_format::DateFormat;
use crate::utils::time_format::TimeFormat;

pub fn open_settings(default_open: SettingDefaultOpen, window: &mut Window, cx: &mut App) -> bool {
    let win_size = window.bounds().size.div(1.5);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::centered(win_size, &cx)),
            window_decorations: Some(WindowDecorations::Client), // no WM frame
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::new("DocKeep Settings")),

                ..Default::default()
            }),
            app_id: Some("dockeep".into()),

            is_resizable: true,
            ..Default::default()
        },
        |window, cx| {
            let view = cx.new(|_| SettingsPage::new(default_open));
            cx.new(|cx| Root::new(view, window, cx).bordered(false))
        },
    )
    .is_ok()
}

pub fn path_exists_or_none(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if !path.exists() {
        return None;
    }
    Some(path.to_path_buf())
}

pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
/// Returns a human-readable "last accessed" string.
/// - Within 30 days: relative "ago" form (e.g. "3 hours ago", "5 days ago").
/// - Older than 30 days: absolute date/time, where the date is formatted
///   according to `date_format` and the time according to `time_format`.
pub fn to_human_datetime(ts: u64, time_format: TimeFormat, date_format: DateFormat) -> String {
    let now = now_timestamp();
    let elapsed = now.saturating_sub(ts);

    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    const THIRTY_DAYS: u64 = 30 * DAY;

    if elapsed < THIRTY_DAYS {
        if elapsed < MINUTE {
            return "just now".to_string();
        } else if elapsed < HOUR {
            let mins = elapsed / MINUTE;
            return format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" });
        } else if elapsed < DAY {
            let hours = elapsed / HOUR;
            return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
        } else {
            let days = elapsed / DAY;
            return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
        }
    }

    // Older than 30 days — format as <date> <time>
    let (year, month, day, hour, minute) = timestamp_to_parts(ts);
    format!(
        "{} {}",
        date_format.format_date(year, month, day),
        time_format.format_time(hour, minute)
    )
}

/// Converts a Unix timestamp (seconds since epoch) to (year, month, day, hour, minute)
/// using the Gregorian calendar (Howard Hinnant's algorithm).
fn timestamp_to_parts(ts: u64) -> (i32, u32, u32, u32, u32) {
    let secs_in_day = (ts % 86400) as u32;
    let days = (ts / 86400) as i64;

    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;

    // Civil date from days since Unix epoch (1970-01-01)
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe as i32 + era as i32 * 400 + if month <= 2 { 1 } else { 0 };

    (year, month, day, hour, minute)
}

pub fn open_in_file_explorer(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // /select, highlights the file itself instead of just opening the folder
        Command::new("explorer")
            .args(["/select,", &path.to_string_lossy()])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux has no universal "reveal and select" — fall back to opening the containing folder
        let dir = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        Command::new("xdg-open").arg(dir).spawn()?;
    }

    Ok(())
}
