# Dockeep

[![CI](https://github.com/hazyio/dockeep/actions/workflows/ci.yml/badge.svg)](https://github.com/hazyio/dockeep/actions/workflows/ci.yml)
[![Release](https://github.com/hazyio/dockeep/actions/workflows/release.yml/badge.svg)](https://github.com/hazyio/dockeep/actions/workflows/release.yml)

**Dockeep** is a cross-platform desktop app built with Rust and GPUI for capturing screenshots of pages in **Chrome and Chromium-based browsers**. It controls the browser through the Chrome DevTools Protocol (CDP), so screenshots are rendered by the browser itself instead of being grabbed from the operating system's desktop. Captures are organized into projects, saved with their source URL embedded when possible, and can be committed to Git automatically.

> **Browser-only capture:** Dockeep does not capture arbitrary desktop windows, native applications, or the whole screen. Every capture comes from a Chrome/Chromium page that Dockeep is controlling.

## Features

- **Chrome/Chromium capture over CDP** — launch a visible browser or attach to one that is already running with remote debugging enabled.
- **Three capture modes** — desktop-sized viewport, mobile-sized viewport, and a draggable/resizable region selector injected into the page.
- **Tab picker** — choose which tab to capture when the browser has more than one open.
- **Source URL metadata** — the page URL is embedded in format-specific metadata and shown in the image info popover, with a button to reopen it in the browser.
- **Project library** — add project folders, search, sort by name or last access, and see Git branch/dirty state at a glance.
- **Image library** — recursively scan a project for images, filter by path, preview fullscreen, rename, delete, replace, and open in the file manager.
- **Formats and quality** — PNG, JPEG, and WebP with configurable quality, save directory, and filename style.
- **Git integration** — optionally auto-commit added, replaced, and deleted captures, with localized commit messages.
- **Desktop conveniences** — 8 themes, 6 UI languages, configurable date/time formats, and daily logs with one-click export.

## How it works

1. Dockeep launches Chrome/Chromium, or attaches to a running browser through its DevTools port.
2. It uses CDP to resize the viewport, inject the region selector, and capture the page.
3. The image is saved into the selected project, and the source URL is embedded when possible.
4. If auto-commit is enabled and the project is a Git repository, Dockeep stages the image and creates a commit.

## Requirements

- **Chrome or Chromium** — Firefox and Safari are not supported. Other Chromium-based browsers can often be used by pointing Dockeep at their executable, but Chrome/Chromium are the tested targets.
- **Rust stable 1.85+** — the crate uses the Rust 2024 edition.
- **A graphical desktop session** — Dockeep is a GPUI desktop application.
- **A Git repository** — optional, only needed for auto-commit features. Dockeep uses libgit2 through the `git2` crate, so the Git CLI is not required.

### Linux build dependencies

On Debian/Ubuntu, install the packages used by the CI and release workflows:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libssl-dev \
  libgtk-3-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libvulkan-dev \
  libasound2-dev \
  libfontconfig1-dev \
  libfuse2
```

## Installation

### Build from source

```bash
git clone https://github.com/hazyio/dockeep.git
cd dockeep

# Run in development
cargo run

# Build an optimized binary
cargo build --release
./target/release/dockeep
```

On Windows the binary is `target\release\dockeep.exe`.

### Pre-built packages

Releases are created or updated on every push to `master`, using the version in `Cargo.toml`. The latest release link always points to the newest build:

- **Linux** — [Download latest](https://github.com/hazyio/dockeep/releases/latest): Debian (`.deb`), AppImage, and RPM packages for x86_64.
- **Windows** — [Download latest](https://github.com/hazyio/dockeep/releases/latest): NSIS installer for x86_64.
- **macOS** — [Download latest](https://github.com/hazyio/dockeep/releases/latest): DMG for Apple Silicon (`aarch64`). Intel Macs can build from source.

## Quick start

1. **Add a project** — click **Add Project** on the home screen and choose the folder where screenshots should live. Dockeep remembers projects in `projects.json`.
2. **Open the project** — click **Open in Editor** on the project card; this opens the project's image library.
3. **Configure Chrome** — Dockeep auto-detects Chrome/Chromium on first launch. If detection fails, set the executable path under **Settings → Chrome**.
4. **Start the browser** — use the green play button in the project view, then navigate to the page you want to capture.
5. **Capture** — choose desktop viewport, mobile viewport, or region capture.
6. **Manage captures** — click an image for a fullscreen preview, or open its info popover to rename it, inspect its metadata, or reopen the source URL.
7. **Configure saving** — open **Project Settings** to choose the save format, output folder, filename style, and per-project auto-commit behavior.

## Chrome setup

### Launch mode (default)

Set the Chrome/Chromium executable under **Settings → Chrome → Chrome Executable Path**. Dockeep launches a visible browser window using that executable. On first run it looks for:

- **Windows:** `C:\Program Files\Google\Chrome\Application\chrome.exe`, `C:\Program Files (x86)\Google\Chrome\Application\chrome.exe`
- **macOS:** `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
- **Linux:** `/usr/bin/google-chrome`, `/usr/bin/google-chrome-stable`, `/usr/bin/chromium`, `/usr/bin/chromium-browser`, `/snap/bin/chromium`

### Attach mode

Attach mode connects to an already-running browser instead of launching a new one. Start Chrome/Chromium with remote debugging enabled:

```bash
google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/dockeep-chrome
```

or:

```bash
chromium --remote-debugging-port=9222 --user-data-dir=/tmp/dockeep-chrome
```

Then enable **Settings → Chrome → Use Attach** and set the port to `9222`. Using a separate `--user-data-dir` avoids conflicts with an already-running profile. Recent Chrome versions also require a non-default user-data directory for remote debugging, so keep the flag in the command.

You can verify the DevTools endpoint before connecting:

```bash
curl http://127.0.0.1:9222/json/version
```

> **Security note:** the DevTools port gives full control over the browser profile. Keep it bound to localhost, do not expose it to untrusted networks, and use a dedicated profile when attaching.

## Capture modes

### Desktop viewport

Dockeep overrides the browser viewport to the configured desktop size (default **1920×1080**), captures the visible viewport, and then restores the previous viewport. This is a viewport capture, **not** a scrolling full-page capture.

### Mobile viewport

Same workflow as desktop capture, but with the mobile size (default **375×667**). This resizes the viewport; it does not emulate a mobile user agent, touch input, or device pixel ratio.

### Region capture

Dockeep injects `assets/selector.js` into the active page, showing a draggable and resizable selection box with live width/height labels.

- Drag the box to move it, or use the eight handles to resize it.
- Click **Select** to capture the selected region.
- Click **Cancel** or press **Esc** in the page to abort.
- Dockeep polls for the selection until the configured **crop timeout** expires (default **15 seconds**).
- The region is captured with `captureBeyondViewport`, so selections that extend outside the current viewport can be included.

Some privileged pages (for example `chrome://` URLs) may reject script injection. Use viewport capture or a normal web page for region capture.

### Multiple tabs

If the controlled browser has more than one tab, Dockeep shows a tab picker before capturing. With a single tab, it captures that tab directly. **Focus Browser** brings the first tab to the front.

### Replacing an existing screenshot

Click the replace icon on an image card, then take the next capture. Dockeep saves the new capture over that image instead of creating a new file. Press **Esc** or click the cancel button to stop replacing. If the project's save format changed, the file extension is updated and the old file is removed.

### Filenames and saving

- If **Save with Tab Title** is enabled and the tab has a title, the title is used with spaces, `/`, and `\` replaced by `-`.
- Otherwise the filename is `Screenshot_YYYYMMDD_HHMMSS`.
- Files use the project's save format: `.png`, `.jpeg`, or `.webp`.
- Dockeep refuses to overwrite an existing file during a new capture; use the replace workflow instead.
- The configured save directory must exist. The Project Settings folder picker only allows folders inside the project root.

## Embedded capture metadata

Only the source URL is embedded — Dockeep does not embed the tab title, timestamp, or other metadata. The key is `dockeep_capture_url`:

| Format | Storage |
| --- | --- |
| PNG | `tEXt` chunk with keyword `dockeep_capture_url` |
| JPEG | `COM` segment containing `dockeep_capture_url=<url>` |
| WebP | XMP chunk containing `<dockeep_capture_url>...</dockeep_capture_url>` |

The image info popover reads this value and offers a button to reopen the URL in the controlled browser. If embedding fails, the image is still saved without metadata, and the popover shows **Unknown**.

> Screenshots can contain sensitive URLs, including query parameters or tokens. Review images before sharing them publicly.

## Projects and the image library

### Home screen

- Lists every project stored in `projects.json`.
- Search by project name or path.
- Sort by name or last accessed time.
- Add a project folder or remove a project from the list. Removing a project **does not delete any files**.
- Shows the Git branch and whether the repository is clean or dirty.
- Updates the project's last-accessed timestamp when you open it.
- If the repository is dirty and auto-commit is enabled, Dockeep asks for confirmation before opening the project.

### Project view

- Recursively scans the project for images, skipping paths matched by `.dockeepignore` or `.gitignore`.
- Only files with a PNG, JPEG, or WebP extension **and** a matching image header are listed.
- Filter images by path or filename, sort by filename or last modified time, and refresh the scan.
- Click an image to open a fullscreen preview; click again or press **Esc** to close it.
- The info popover shows the filename, dimensions, capture URL, created/modified timestamps, file type, and size.
- Rename a file in place (the extension is preserved), open it in the file manager, or delete it with confirmation.
- Delete removes the file from disk. If the global auto-commit setting is enabled, the deletion is committed too.

## Git integration

Auto-commit is available in **Settings → Git** and in each project's **Project Settings**:

- New captures and replaced captures are committed when the **project's** auto-commit setting is enabled.
- Deletions are committed when the **global** auto-commit setting is enabled.
- Commit messages are localized. The commit language can follow the UI language or use a specific language selected in **Settings → Git**.
- Auto-commit only runs when the project is inside a Git repository. If no Git signature is configured, Dockeep uses `dockeep <dockeep@local>`.

## Configuration

Dockeep uses two levels of settings:

- **Global settings** in `app_config.json` control the application, capture engine, Chrome connection, and Git behavior.
- **Project settings** in `.dockeep` control where and how captures are saved for a specific project.

### Settings pages

| Page | Controls |
| --- | --- |
| General | Language, theme, date/time format, export logs |
| Capture | Save with tab title, capture from surface, image format, crop timeout, quality, desktop/mobile viewport sizes |
| Chrome | Executable path, attach mode, DevTools port |
| Git | Auto-commit, commit language |
| Project Settings | Save format, tab-title filenames, auto-commit, save folder (per project) |

### Storage locations

`<config_dir>` is the platform configuration directory:

| Platform | `<config_dir>` |
| --- | --- |
| Linux | `${XDG_CONFIG_HOME:-~/.config}` |
| macOS | `~/Library/Application Support` |
| Windows | `%APPDATA%` (Roaming) |

| Data | Location |
| --- | --- |
| Global settings | `<config_dir>/hazyio_dockeep/app_config.json` |
| Project registry | `<config_dir>/hazyio_dockeep/projects.json` |
| Logs | `<config_dir>/hazyio_dockeep/logs/` |
| Per-project settings | `<project>/.dockeep` |
| Captures | `<project>/<save_to_dir>/` |

### Global settings (`app_config.json`)

The file is pretty-printed JSON. If it is missing, Dockeep creates it with defaults. If it is invalid, Dockeep logs a warning, falls back to defaults, and rewrites the file.

```json
{
  "theme": "ShadcnDark",
  "time_format": "TwentyFourHour",
  "date_format": "DayMonthYear",
  "language": "English",
  "chrome_config": {
    "path": "/usr/bin/google-chrome",
    "use_attach": false,
    "attach_port": 9222
  },
  "capture_setting": {
    "save_with_tab_title": true,
    "quality": 100,
    "capture_from_surface": true,
    "image_format": "Png",
    "mobile_capture_sizing": { "width": 375.0, "height": 667.0 },
    "desktop_capture_sizing": { "width": 1920.0, "height": 1080.0 },
    "crop_timeout": 15
  },
  "git_setting": {
    "auto_commit": false,
    "use_custom_language": true,
    "language": "English"
  }
}
```

Key fields:

- `theme` — `AuroraLight`, `AyuLight`, `AyuDark`, `TokyoNight`, `TokyoStorm`, `TokyoMoon`, `ShadcnLight`, or `ShadcnDark` (default).
- `language` — `English`, `Deutsch`, `Spanish`, `Korean`, `Russian`, or `ChineseSimplified`.
- `time_format` — `TwentyFourHour` or `TwelveHour`.
- `date_format` — `DayMonthYear`, `MonthDayYear`, or `YearMonthDay`.
- `chrome_config.path` — Chrome/Chromium executable.
- `chrome_config.use_attach` — connect to an existing browser instead of launching one.
- `chrome_config.attach_port` — DevTools port used in attach mode (default `9222`).
- `capture_setting.quality` — 0–100; applies to JPEG and WebP. PNG is lossless.
- `capture_setting.capture_from_surface` — capture the rendered page surface directly (recommended; default `true`).
- `capture_setting.desktop_capture_sizing` / `mobile_capture_sizing` — viewport sizes used by the two viewport capture modes.
- `capture_setting.crop_timeout` — seconds to wait for a region selection (default `15`).
- `git_setting.auto_commit` — global auto-commit switch.
- `git_setting.use_custom_language` / `language` — use the UI language or a specific commit language.

The Capture settings page also exposes format and tab-title controls. The save pipeline reads the **project's** `.dockeep` file for those values, so use Project Settings when a capture saves with an unexpected format or filename.

### Project settings (`.dockeep`)

`.dockeep` is JSON despite having no extension. It is written by the Project Settings dialog and is authoritative for saving:

```json
{
  "save_format": "Png",
  "save_with_tab_title": true,
  "auto_commit": false,
  "save_to_dir": "screenshots"
}
```

- `save_format` — `Png`, `Jpeg`, or `Webp`.
- `save_with_tab_title` — use the page title for filenames instead of a timestamp.
- `auto_commit` — commit new and replaced captures for this project.
- `save_to_dir` — path relative to the project root. An empty string saves directly in the project root. The directory must exist.

If `.dockeep` does not exist, Dockeep uses built-in defaults: PNG, tab-title filenames, the project root, and no auto-commit. Open Project Settings once to create the file and choose your preferred behavior.

### Ignore patterns

The recursive image scan reads **either** `.dockeepignore` **or** `.gitignore` from the project root:

- If `.dockeepignore` exists, it is used and `.gitignore` is ignored entirely.
- Otherwise `.gitignore` is used as a fallback.
- Nested ignore files are not read.

Patterns use a simplified, case-insensitive gitignore-style glob syntax:

- `*` matches any characters except `/`.
- `**` matches any characters including `/`.
- `?` matches a single character except `/`.
- A leading `/` anchors to the project root; a trailing `/` is accepted for directory-style patterns.
- Lines starting with `#` are comments.

Example `.dockeepignore`:

```gitignore
# Ignore generated screenshot folders
/screenshots/

# Ignore temporary files
*.tmp

# Ignore common build directories
/node_modules/
/target/
```

Ignore patterns only affect image scanning. They do not prevent Dockeep from saving captures into a folder.

### `codebook.toml`

`codebook.toml` is a word list for the Codebook spell-checker. It is **not** application configuration or packaging metadata. Packaging metadata lives in `Cargo.toml` under `[package.metadata.packager]` and `[package.metadata.generate-rpm]`.

## Themes and languages

**Themes:** Aurora Light, Ayu Light, Ayu Dark, Tokyo Night, Tokyo Storm, Tokyo Moon, Shadcn Light, Shadcn Dark (default).

**UI languages:** English, Deutsch, Español, 한국어, Русский, 简体中文.

Language files live in `locales/`. The UI language, theme, date format, and time format are all changeable from **Settings → General**.

## Logging and diagnostics

- Logs are written daily to `<config_dir>/hazyio_dockeep/logs/`.
- The last 7 log files are kept.
- In debug builds, logs also go to the console. Set `RUST_LOG` to change the level, for example `RUST_LOG=debug cargo run`.
- **Settings → General → Export Logs** bundles the logs into `dockeep-logs-YYYYMMDD-HHMMSS.txt` in your Downloads folder.
- Panics are captured in the log, which is useful on Windows where the release binary has no console.

## Development

```bash
# Formatting
cargo fmt --all --check

# Lints
cargo clippy --all-targets -- -D warnings

# Tests
cargo test --all
```

CI runs formatting, Clippy, and tests on every pull request to `master`.

### Project layout

```
src/config/       Settings structs and persistence
src/files/        Image scanning, saving, metadata, ignore patterns
src/scenes/       Home, project library, and settings UI
src/utils/        Git, themes, i18n, logging, project registry
assets/           Icons and the region selector injected into pages
locales/          UI translations
themes/           Theme JSON files
```

### Packaging

Package metadata is configured in `Cargo.toml`. The release workflow uses `cargo-packager` and, on Linux, `cargo-generate-rpm`. Install the tools first if needed:

```bash
# Linux
cargo install cargo-packager cargo-generate-rpm

# Windows / macOS
cargo install cargo-packager
```

Then build packages for the current platform:

```bash
# Linux
cargo packager --release --formats deb,appimage
cargo generate-rpm --output dist/

# Windows
cargo packager --release --formats nsis

# macOS
cargo packager --release --formats dmg
```

Artifacts are written to `dist/`.

## Troubleshooting

**"Chrome executable path not found"**
Set the correct executable under **Settings → Chrome → Chrome Executable Path**. On Linux, verify the path with `which google-chrome` or `which chromium`. Flatpak installations are not auto-detected; point Dockeep at the Flatpak wrapper or executable manually.

**Attach mode fails to start**
Make sure the browser was started with `--remote-debugging-port=<port>` and that the port matches **Settings → Chrome → Port**. Check the endpoint with `curl http://127.0.0.1:9222/json/version`.

**"No active tab"**
Start the browser from Dockeep and open at least one page before capturing.

**Region capture fails or times out**
Some pages block injected scripts or are privileged pages such as `chrome://`. Increase the crop timeout or use desktop/mobile viewport capture. Make sure you click **Select** or press **Esc** in the page.

**No images are listed**
Check that the capture directory is inside the project, that files are valid PNG/JPEG/WebP images, and that `.dockeepignore`/`.gitignore` are not excluding them. Use the refresh button to rescan.

**"Cannot save to existing file"**
A file with the generated name already exists. Enable tab-title filenames, choose a different save folder, or use the replace workflow on the existing image.

**Captures save as PNG even though the format was changed**
The project's `.dockeep` is authoritative. Open **Project Settings** in the project view and set the format there.

**Linux build fails**
Install the dependencies listed under [Linux build dependencies](#linux-build-dependencies). GPUI also needs a working graphical session and Vulkan/OpenGL support.

## Roadmap

- [ ] Full-page (scrolling) capture.
- [ ] Screen recording / casting support.
- [ ] Per-project overrides for viewport size, quality, and crop timeout.
- [ ] Broader automated coverage of capture and Git workflows.

## Contributing

Contributions are welcome. Before opening a pull request:

1. Run `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all`.
2. Add or update tests for behavior changes.
3. When adding UI strings, update `locales/en.yaml` and, where possible, the other locale files.
4. Keep the README and configuration examples in sync with the code.

## License

MIT
