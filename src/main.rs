pub mod components;
pub mod scenes;
pub mod utils;
use gpui_kit::component::*;
use gpui_kit::*;
use tracing_subscriber::EnvFilter;

use crate::{
    scenes::{app::MyApp, home::view::HomePage},
    utils::{app_config::AppConfig, app_theme::AppTheme},
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
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    gpui_kit::application()
        .with_assets(utils::asset_source::Assets)
        .run(move |cx| {
            setup(cx);

            cx.spawn(async move |cx| {
                cx.open_window(
                    WindowOptions {
                        titlebar: None,                                      // no title bar
                        window_decorations: Some(WindowDecorations::Client), // no WM frame
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
