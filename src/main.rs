#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod components;
pub mod config;
pub mod files;
pub mod scenes;
pub mod utils;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::{
    config::AppConfig,
    scenes::{app::MyApp, home::view::HomePage},
    utils::{app_theme::AppTheme, logging::init_logging},
};
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");
fn setup(cx: &mut App) {
    // This must be called before using any GPUI Component features.
    gpui_kit::init(cx);
    AppTheme::load_all(cx);
    let config = AppConfig::load();
    config.theme.switch_to(cx);
    config.language.set();
}

fn main() {
    let log_dir = AppConfig::log_dir();
    std::fs::create_dir_all(&log_dir).ok();
    let _guard = init_logging(&log_dir); // bind it as _guard it lives until main returns
    gpui_kit::application()
        .with_assets(utils::asset_source::Assets)
        .run(move |cx| {
            setup(cx);

            cx.spawn(async move |cx| {
                cx.open_window(
                    WindowOptions {
                        window_decorations: Some(WindowDecorations::Client), // no WM frame
                        titlebar: Some(TitlebarOptions {
                            title: Some(SharedString::new("DocKeep")),
                            appears_transparent: true, // hides the native bar on Windows/macOS
                            ..Default::default()
                        }),
                        is_resizable: true,
                        app_id: Some("dockeep".into()),
                        ..Default::default()
                    },
                    |window, cx| {
                        let view = cx.new(|app| {
                            let my_app_handle = app.weak_entity();
                            let home = app.new(|cx| HomePage::new(my_app_handle, window, cx));
                            MyApp { view: home.into() }
                        });
                        // This first level on the window, should be a Root.
                        cx.new(|cx| Root::new(view, window, cx).bordered(false))
                    },
                )
                .expect("Failed to open window");
            })
            .detach();
        });
}
