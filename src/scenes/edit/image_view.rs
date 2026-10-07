use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use gpui_kit::component::button::Button;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::files::prelude as files;
use crate::files::prelude::open_in_file_explorer;
use crate::scenes::edit::image_view_info_popup::{ImageViewInfoPopup, ImageViewInfoPopupEvents};
use crate::utils::app_icons::AppIcons;
pub enum ImageViewEvents {
    Replace(PathBuf),
    OpenInFullscreen(PathBuf),
    Delete(PathBuf),
    CancelReplace,
    OpenUrlInBrowser(String),
}

pub struct ImageView {
    pub path: PathBuf,
    pub index: usize,
    pub is_replacing: bool,
    pub last_modified_timestamp: u64,
    image_cache: Entity<RetainAllImageCache>,
    image_view_popup: Entity<ImageViewInfoPopup>,
    popup_subscription: Option<Subscription>,
}

impl ImageView {
    pub fn new(
        path: PathBuf,
        last_modified_timestamp: u64,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            path: path.clone(),
            index,
            image_cache: RetainAllImageCache::new(cx), // Context<T> derefs to App, satisfies `&mut App`
            is_replacing: false,
            last_modified_timestamp,
            image_view_popup: cx.new(|_| ImageViewInfoPopup::new(index, path)),
            popup_subscription: None,
        }
    }
}

impl EventEmitter<ImageViewEvents> for ImageView {}

impl ImageView {
    /// Replaces the image cache so the next render reloads the file from disk.
    pub fn bust_cache(&mut self, cx: &mut Context<Self>) {
        let path = self.path.clone();

        self.image_cache = RetainAllImageCache::new(cx);
        // reload popup metadata, e.g. last modified timestamp
        self.image_view_popup
            .update(cx, |info, _| info.refresh(path));
        // update the last modified timestamp
        self.update_last_modified_with_now();
    }
    pub fn reload(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.path = path;
        self.bust_cache(cx);

        cx.notify();
    }
    pub fn update_last_modified(&mut self, last_modified_timestamp: u64) {
        self.last_modified_timestamp = last_modified_timestamp;
    }
    pub fn update_last_modified_with_now(&mut self) {
        let last_modified_timestamp =
            files::last_modified(&self.path).unwrap_or(SystemTime::UNIX_EPOCH);
        let last_modified_timestamp = last_modified_timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.update_last_modified(last_modified_timestamp);
    }
    fn build_popup_subscription(
        entity: &Entity<ImageViewInfoPopup>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe_in(
            entity,
            window,
            move |this, _, event: &ImageViewInfoPopupEvents, window, cx| {
                match event {
                    ImageViewInfoPopupEvents::OpenUrl(url) => {
                        cx.emit(ImageViewEvents::OpenUrlInBrowser(url.clone()));
                        cx.notify();
                    }
                    ImageViewInfoPopupEvents::UpdateName(new_name) => {
                        let result = files::rename_file(&this.path, new_name);
                        match result {
                            Ok(new_path) => {
                                this.path = new_path.clone();
                                // update the popup with the new path
                                this.image_view_popup.update(cx, |popup, cx| {
                                    popup.path = new_path;
                                    cx.notify();
                                });
                                cx.notify();
                            }
                            Err(e) => {
                                tracing::error!("failed to rename file: {}", e);
                                window.push_notification(t!("error.failed_to_rename_file"), cx);
                            }
                        }
                    }
                }
                cx.notify();
            },
        )
    }
}

impl Render for ImageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.popup_subscription.is_none() {
            // Subscribe to popup events when the view is first rendered
            self.popup_subscription = Some(Self::build_popup_subscription(
                &self.image_view_popup,
                window,
                cx,
            ));
        }
        let path = self.path.clone();
        let file_name = files::file_name_without_extension(&path);

        let m_path = path.clone();
        let image_info_popup = self.image_view_popup.clone();

        div()
            .v_flex()
            .w(relative(1.))
            .h(px(250.))
            .cursor_pointer()
            .bg(cx.theme().muted)
            .rounded_md()
            .border_1()
            .when(self.is_replacing, |d| d.border_color(cx.theme().red))
            .child(
                img(path.clone())
                    .image_cache(&self.image_cache)
                    .id(format!("fullscreen-view-image-{}", self.index))
                    .flex_grow_1()
                    .w(relative(1.0))
                    .on_click(cx.listener(|this, _, _, cx| {
                        let path = this.path.clone();
                        tracing::debug!("Edit Image: {:?}", path);
                        cx.emit(ImageViewEvents::OpenInFullscreen(path));
                    })),
            )
            .child(
                div()
                    .v_flex()
                    .p_3()
                    .bg(cx.theme().background)
                    .rounded_b_md()
                    .child(div().child(file_name).truncate())
                    .gap_2()
                    .child(
                        div()
                            .h_flex()
                            .gap_3()
                            .justify_end()
                            .child(
                                Button::new(format!("replace-image-{}", self.index))
                                    .when(!self.is_replacing, |button| {
                                        button
                                            .child(AppIcons::Replace)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let path = this.path.clone();
                                                tracing::debug!("Replace Image: {:?}", path);

                                                cx.emit(ImageViewEvents::Replace(path));
                                            }))
                                            .tooltip(t!("label.replace_image"))
                                    })
                                    .when(self.is_replacing, |button| {
                                        button
                                            .child(AppIcons::Close)
                                            .on_click(cx.listener(|_, _, _, cx| {
                                                cx.emit(ImageViewEvents::CancelReplace);
                                            }))
                                            .tooltip(t!("label.cancel_replace"))
                                    }),
                            )
                            .child(image_info_popup)
                            .child(
                                Button::new(format!("view-image-{}", self.index))
                                    .child(AppIcons::Folder)
                                    .on_click(move |_, window, cx| {
                                        tracing::debug!("Opening Image: {:?}", m_path);
                                        window.push_notification(
                                            t!("label.opening_in_file_manager"),
                                            cx,
                                        );

                                        let _ = open_in_file_explorer(m_path.as_path());
                                    })
                                    .tooltip(t!("label.open_in_file_manager")),
                            )
                            .child(
                                Button::new(format!("delete-image-popover-{}", self.index))
                                    .text_color(cx.theme().red)
                                    .child(AppIcons::Trash)
                                    .tooltip(t!("label.delete"))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        let path = Arc::new(this.path.clone());
                                        let entity = cx.entity().downgrade();
                                        tracing::debug!("Deleting    Image: {:?}", path);
                                        window.open_alert_dialog(cx, move |alert, _, _| {
                                            let path = path.clone();
                                            let entity = entity.clone();
                                            alert
                                                .title(t!("dialog.delete_file"))
                                                .description(t!("dialog.delete_file_confirmation"))
                                                .show_cancel(true)
                                                .on_ok(move |_, window, cx| {
                                                    let remove =
                                                        fs::remove_file(path.as_path()).is_ok();
                                                    if remove {
                                                        let _ = entity.update(cx, |_, cx| {
                                                            cx.emit(ImageViewEvents::Delete(
                                                                (*path).clone(),
                                                            ));
                                                        });
                                                    } else {
                                                        window.push_notification(
                                                            "Failed to delete file",
                                                            cx,
                                                        );
                                                    }

                                                    true
                                                })
                                        })
                                    })),
                            ),
                    ),
            )
    }
}
