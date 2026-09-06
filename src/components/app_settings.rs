use gpui_kit::{component::WindowExt, *};
use rust_i18n::t;

pub struct AppSettings;

impl AppSettings {
    pub fn open(window: &mut Window, cx: &mut App) {
        window.open_sheet(cx, |sheet, _, _| {
            sheet
                .title(t!("title.settings"))
                .overlay(true)
                .overlay_closable(true)
                .child("Sheet settings content")
        });
    }
}
