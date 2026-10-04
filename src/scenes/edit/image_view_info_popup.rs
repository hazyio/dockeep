use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Local, Timelike};
use gpui_kit::base::input::{InputEvent, InputState};
use gpui_kit::base::{Disableable, StyledExt};
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::Input;
use gpui_kit::component::label::Label;
use gpui_kit::component::popover::Popover;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::config::AppConfig;
use crate::utils::app_icons::AppIcons;
use crate::utils::files::{self, read_capture_url};
use crate::utils::{date_format::DateFormat, time_format::TimeFormat};
pub enum ImageViewInfoPopupEvents {
    OpenUrl(String),
    UpdateName(String),
}
pub struct ImageViewInfoPopup {
    index: usize,
    path: PathBuf,
    edit_name: Option<Entity<InputState>>,
    last_modified: String,
    created: String,
    capture_url: Option<String>,
    size: String,
    dimensions: String,
    file_type: String,
}
impl EventEmitter<ImageViewInfoPopupEvents> for ImageViewInfoPopup {}

impl ImageViewInfoPopup {
    pub fn new(index: usize, path: PathBuf) -> Self {
        let capture_url = Self::get_capture_url(&path);
        let (created, modified) = Self::get_metadata(&path);
        let dimensions = Self::get_dimensions(&path);
        let size = Self::get_size(&path);
        let file_type = Self::get_file_type(&path);
        Self {
            index,
            path,
            edit_name: None,
            last_modified: modified,
            created,
            capture_url,
            dimensions,
            size,
            file_type,
        }
    }
    fn get_file_type(path: &PathBuf) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string()
    }
    fn get_capture_url(path: &PathBuf) -> Option<String> {
        match std::fs::read(&path) {
            Ok(data) => {
                if let Some(url) = read_capture_url(&data) {
                    println!("captured from {url}");
                    Some(url)
                } else {
                    None
                }
            }
            Err(e) => {
                tracing::error!("Failed to read file {:?}: {}", path, e);
                None
            }
        }
    }
    fn get_dimensions(path: &PathBuf) -> String {
        match imagesize::size(path) {
            Ok(size) => {
                let (w, h) = (size.width, size.height);
                format!("{}x{}", w, h)
            }
            Err(e) => {
                tracing::error!("Failed to get dimensions for {:?}: {}", path, e);
                String::from("Unknown")
            }
        }
    }
    fn get_size(path: &PathBuf) -> String {
        match std::fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();
                files::human_bytes(size)
            }
            Err(e) => {
                tracing::error!("Failed to get size for {:?}: {}", path, e);
                String::from("Unknown")
            }
        }
    }
    fn get_metadata(path: &PathBuf) -> (String, String) {
        let app_config = AppConfig::load();

        match fs::metadata(path.clone()) {
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
        }
    }
    pub fn refresh(&mut self) {
        let path = &self.path;
        let capture_url = Self::get_capture_url(path);
        let (created, modified) = Self::get_metadata(path);
        let dimensions = Self::get_dimensions(path);
        let size = Self::get_size(path);
        self.dimensions = dimensions;
        self.capture_url = capture_url;
        self.created = created;
        self.last_modified = modified;
        self.size = size;
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

impl Render for ImageViewInfoPopup {
    fn render(&mut self, window: &mut Window, parent_cx: &mut Context<Self>) -> impl IntoElement {
        let default_name = files::file_name_without_extension(&self.path);
        let default_name = default_name.as_str();

        if self.edit_name.is_none() {
            //  first render, create the edit_name input state. very inefficient the best way it to accpet window in [`Self::new`] but it has a cascading change.

            let edit_name =
                parent_cx.new(|cx| InputState::new(window, cx).default_value(default_name));
            parent_cx
                .subscribe_in(&edit_name, window, move |this, input, event, window, cx| {
                    // reset to default value on blur
                    let default_value = this
                        .path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    if let InputEvent::Blur = event {
                        input.update(cx, |state, cx| {
                            state.set_value(default_value.clone(), window, cx);
                        });
                    }
                })
                .detach();
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
                let has_value =
                    edit_name.read_with(parent_cx, |edit_name, _| !edit_name.value().is_empty());
                cx.child(
                    div()
                        .min_w(px(350.0))
                        .max_w(px(450.0))
                        .v_flex()
                        .gap_2()
                        .child(
                            Label::new(t!("label.file_name"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4))
                                .truncate(),
                        )
                        .child(
                            div().h_flex().gap_2().child(Input::new(&edit_name)).child(
                                Button::new(format!("save-image-info-{}", self.index))
                                    .child(AppIcons::Check)
                                    .disabled(!file_name_changed && has_value)
                                    .on_click(parent_cx.listener(|this, _, _, cx| {
                                        if let Some(edit_name) = &this.edit_name {
                                            let new_name = edit_name
                                                .read_with(cx, |edit_name, _| {
                                                    edit_name.value().clone()
                                                });
                                            cx.emit(ImageViewInfoPopupEvents::UpdateName(
                                                new_name.to_string(),
                                            ));
                                            cx.notify();
                                        }
                                    })),
                            ),
                        )
                        .child(
                            Label::new(t!("label.dimensions"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.dimensions.clone())
                        .child(
                            Label::new(t!("label.captured_from"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .when_none(&self.capture_url, |cx| cx.child("Unknown"))
                        .when_some(self.capture_url.clone(), |cx, url| {
                            cx.child(
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .child(div().flex_grow_1().child(url))
                                    .child(
                                        Button::new(format!("open-url-for-image-{}", self.index))
                                            .child(AppIcons::SquareArrowOutUpRight)
                                            .tooltip(t!("label.open_url_in_browser"))
                                            .on_click(parent_cx.listener(|this, _, _, cx| {
                                                if let Some(url) = &this.capture_url {
                                                    // if for some freak reason the url is empty, don't open it
                                                    if !url.is_empty() {
                                                        cx.emit(ImageViewInfoPopupEvents::OpenUrl(
                                                            url.clone(),
                                                        ));
                                                        cx.notify();
                                                    }
                                                }
                                            })),
                                    ),
                            )
                        })
                        .child(
                            Label::new(t!("label.created_at"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.created.clone())
                        .child(
                            Label::new(t!("label.updated_at"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.last_modified.clone())
                        .child(
                            Label::new(t!("label.file_type"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.file_type.clone())
                        .child(
                            Label::new(t!("label.size"))
                                .text_color(parent_cx.theme().foreground.opacity(0.4)),
                        )
                        .child(self.size.clone()),
                )
            })
    }
}
