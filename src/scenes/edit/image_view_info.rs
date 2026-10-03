use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Local, Timelike};
use gpui_kit::base::{Disableable, StyledExt};
use gpui_kit::base::input::InputState;
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::Input;
use gpui_kit::component::label::Label;
use gpui_kit::component::popover::Popover;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::config::AppConfig;
use crate::utils::app_icons::AppIcons;
use crate::utils::{date_format::DateFormat, time_format::TimeFormat};

pub struct ImageViewInfo {
    index: usize,
    path: PathBuf,
    edit_name: Option<Entity<InputState>>,
    last_modified: String,
    created: String,
}
impl ImageViewInfo {
    pub fn new(index: usize, path: PathBuf) -> Self {
        let app_config = AppConfig::load();
        let (created, modified) = match fs::metadata(path.clone()) {
            Ok(meta) => {
                let modified = meta.modified().map_or_else(
                    |_| String::from("Unknown"),
                    |t| format_datetime(t, app_config.date_format, app_config.time_format),
                );
                let created = meta.created().map_or_else(
                    |_| String::from("Unknown"),
                    |t| format_datetime(t, app_config.date_format, app_config.time_format),
                );
                (created, modified)
            }
            Err(e) => {
                tracing::error!("Failed to get metadata for {:?}: {}", path, e);
                (String::from("Unknown"), String::from("Unknown"))
            }
        };

        Self {
            index,
            path,
            edit_name: None,
            last_modified: modified,
            created,
        }
    }
}

/// Formats a filesystem timestamp as `<date> <time>` using the user's chosen
/// [`DateFormat`] and [`TimeFormat`].
fn format_datetime(
    t: std::time::SystemTime,
    date_format: DateFormat,
    time_format: TimeFormat,
) -> String {
    let dt = DateTime::<Local>::from(t);
    format!(
        "{} {}",
        date_format.format_date(dt.year(), dt.month(), dt.day()),
        time_format.format_time(dt.hour(), dt.minute())
    )
}

impl Render for ImageViewInfo {
    fn render(&mut self, window: &mut Window, parent_cx: &mut Context<Self>) -> impl IntoElement {
        let default_name = self.path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

        if self.edit_name.is_none() {
            //  first render, create the edit_name input state. very inefficient the best way it to accpet window in [`Self::new`] but it has a cascading change.

            let edit_name =
                parent_cx.new(|cx| InputState::new(window, cx).default_value(default_name));
            self.edit_name = Some(edit_name);
        }

        Popover::new(format!("open-image-info-popover-{}", self.index))
            .anchor(Anchor::TopCenter)
            .trigger(
                Button::new(format!("open-image-info-{}", self.index))
                    .outline()
                    .child(AppIcons::Info)
                    .tooltip(t!("label.open_image_info")),
            )
            .when_some(self.edit_name.clone(), |cx, edit_name| {
                let file_name_changed = edit_name.read_with(parent_cx, |edit_name, _| {
                    default_name != edit_name.value().as_str()
                });
                let has_value = edit_name.read_with(parent_cx, |edit_name, _| {
                    !edit_name.value().is_empty()
                });
                cx.child(
                    div()
                        .v_flex()
                        .gap_2()
                        .child(
                            Label::new(t!("label.file_name"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(
                            div().h_flex().gap_2().child(Input::new(&edit_name)).child(
                                Button::new(format!("save-image-info-{}", self.index))
                                    .child(AppIcons::Check)
                                    .disabled(!file_name_changed && !has_value),
                            ),
                        )
                        .child(
                            Label::new(t!("label.captured_from"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child("https://google.com")
                        .child(
                            Label::new(t!("label.created_at"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.created.clone())
                        .child(
                            Label::new(t!("label.updated_at"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.last_modified.clone()),
                )
            })
    }
}
