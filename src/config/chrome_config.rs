use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeConfig {
    pub path: String,
    pub use_attach: bool,
    pub attach_port: u16,
}

fn find_chrome() -> String {
    #[cfg(target_os = "windows")]
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];

    #[cfg(target_os = "macos")]
    let candidates = ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"];

    #[cfg(target_os = "linux")]
    let candidates = [
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
    ];

    candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|p| p.to_string())
        .unwrap_or_default()
}

impl Default for ChromeConfig {
    fn default() -> Self {
        Self {
            path: find_chrome(),
            use_attach: false,
            attach_port: 9222,
        }
    }
}
