use std::path::Path;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_logging(log_dir: &Path) -> WorkerGuard {
    let file = rolling::Builder::new()
        .rotation(rolling::Rotation::DAILY)
        .filename_prefix("dockeep")
        .filename_suffix("log")
        .max_log_files(7) // keep a week, delete older
        .build(log_dir)
        .expect("failed to create log file");

    let (file_writer, guard) = non_blocking(file);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(file_writer).with_ansi(false)) // no color codes in files
        .with(cfg!(debug_assertions).then(fmt::layer)) // console only in debug
        .init();

    // log panics too, otherwise they vanish with windows_subsystem = "windows"
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(
            "panic: {info}\n{}",
            std::backtrace::Backtrace::force_capture()
        );
        default_hook(info);
    }));

    guard
}

use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use crate::config::AppConfig;

/// Bundles all log files into one file in the Downloads folder.
/// Returns the path of the file it created.
pub fn export_logs() -> io::Result<PathBuf> {
    let log_dir = AppConfig::log_dir();

    let dest_dir = dirs::download_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no Downloads folder found"))?;
    fs::create_dir_all(&dest_dir)?;

    let mut files: Vec<PathBuf> = fs::read_dir(log_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("dockeep") && name.ends_with(".log"))
        })
        .collect();
    // daily file names contain the date, so name order is chronological
    files.sort();

    if files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no log files found",
        ));
    }

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dest = dest_dir.join(format!("dockeep-logs-{stamp}.txt"));
    let mut out = io::BufWriter::new(fs::File::create(&dest)?);

    writeln!(
        out,
        "Dockeep {} on {} {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )?;

    for file in files {
        let name = file.file_name().unwrap_or_default().to_string_lossy();
        writeln!(out, "\n===== {name} =====")?;
        io::copy(&mut fs::File::open(&file)?, &mut out)?;
    }
    out.flush()?;

    Ok(dest)
}
