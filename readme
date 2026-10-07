# Dockeep

**Dockeep** is a cross-platform desktop application for capturing, organizing, and managing screenshots and screen content. Built with Rust and GPUI, it provides a clean interface for capturing windows, tabs, or selected areas and saving them with metadata.

## Core Features

### Screenshot Capture
- Capture full screens, specific windows, or selected areas
- Automatically embed capture URLs and metadata in saved images
- Support for PNG, JPEG, and WebP formats
- Configurable save directory and naming conventions

### Ignore Patterns
- **`.gitignore`** - Standard git ignore patterns
- **`.dockeepignore`** - Project-specific ignore patterns (takes precedence over `.gitignore`)

Both `.gitignore` and `.dockeepignore` can be used to skip paths during image scanning and operations. If both files exist, `.dockeepignore` patterns take precedence over `.gitignore` patterns.

### Image Format Support
- PNG, JPEG, and WebP formats
- Configurable default save format
- Format-aware metadata embedding (capture URLs embedded in appropriate format-specific metadata)

### Language & Theme Support
- i18n support with locale files
- Adjustable color themes
- Language selection

## Installation

```bash
# Clone the repository
git clone https://github.com/hazyio/dockeep.git
cd dockeep

# Build the project
cargo build --release

# Or run directly
cargo run
```

## Usage

1. **Launch the application** - Opens a window with capture options
2. **Capture content** - Select area, window, or tab to capture
3. **Saved images** - Automatically saved to configured directory
4. **Manage settings** - Configure save format, directory, and naming preferences through the settings panel

## Configuration

Dockeep uses a two-level configuration system:

### Project-Level Settings (`.dockeep`)
Project-specific settings are saved in `.dockeep` at the **project root path**. These settings take precedence over global settings. The file contains:

- `save_format` - Image format for saves (png/jpeg/webp)
- `save_with_tab_title` - Whether to include tab titles in filenames
- `auto_commit` - Auto-commit captured content
- `save_to_dir` - **Relative path from project root** where captures are saved
  - If empty, saves directly in the project root
  - If set to `screenshots`, saves in `project_root/screenshots/`

Project settings are loaded/saved via `ProjectSettingsData::load()` / `ProjectSettingsData::save()` and persist across sessions.

### Global Settings (`app_config.json`)
Global application settings are stored in the OS configuration directory:
- **Linux/macOS**: `~/.config/hazyio_dockeep/app_config.json`
- Windows: `%APPDATA%\hazyio_dockeep\app_config.json`

These include theme, language, time/date formats, and other global preferences. Modified through the app's Settings panel.

**Important**: When both project-level `.dockeep` and global `app_config.json` exist, the project-level settings take precedence for capture-related options (format, save directory, tab title behavior).

### `codebook.toml`
This file is used for **RPM packaging metadata** and contains:
- Product name, identifier, and output directory
- Package icons
- RPM asset mapping

It is **not** used for application settings configuration.

## Ignore Pattern Files

### `.dockeepignore`
Created in the project root. Takes precedence over `.gitignore`. Uses gitignore-style patterns.

### `.gitignore`
Standard git ignore file. Used as fallback when `.dockeepignore` doesn't exist.

Example `.dockeepignore`:
```
# Ignore screenshot directories
/screenshots/
# Ignore temporary files
*.tmp
# Ignore specific paths
/node_modules/
/target/
```

## Roadmap

- [ ] Add Tests
- [ ] Demo chromiumoxide with screen casting.

## Recent Fixes

- **fix**: `/home/daniel/Projects/dockeep/src/files/files.rs` not replacing extension when format changes

  - Fixed `save_screenshot` function to properly replace file extensions when the save format changes
  - Ensures that when format changes (e.g., from PNG to JPEG), the output file gets the correct extension
  - Previously, the extension was not being replaced, causing saved files to have mismatched extensions

## Building for Distribution

```bash
cargo build --release
```

The packaged binary will be at `target/release/dockeep`. Package metadata is configured in `Cargo.toml` for product name, identifier, and icons.

## Download

Pre-built installers and packages are available for each platform. Releases are automatically created on every push to the `master` branch, and the latest release link always points to the most recent version:

- **Linux**: [Download Latest](https://github.com/hazyio/dockeep/releases/latest) — Debian (.deb), AppImage, and RPM packages
- **Windows**: [Download Latest](https://github.com/hazyio/dockeep/releases/latest) — NSIS installer
- **macOS**: [Download Latest](https://github.com/hazyio/dockeep/releases/latest) — DMG disk image

Alternatively, you can build from source:

```bash
cargo build --release   # Linux/macOS
# For Windows cross-compilation:
cargo build --release --target x86_6-pc-windows-gnu
```

## License

MIT