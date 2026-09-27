use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::component::button::Button;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::utils::app_icons::AppIcons;
use crate::utils::prelude::open_in_file_explorer;
pub enum ImageViewEvents {
    Replace(PathBuf),
    OpenInFullscreen(PathBuf),
    Delete(PathBuf),
    CancelReplace,
}

pub struct ImageView {
    pub path: PathBuf,
    pub index: usize,
    pub is_replacing: bool,

    image_cache: Entity<RetainAllImageCache>,
}

impl ImageView {
    pub fn new(path: PathBuf, index: usize, cx: &mut Context<Self>) -> Self {
        Self {
            path,
            index,
            image_cache: RetainAllImageCache::new(cx), // Context<T> derefs to App, satisfies `&mut App`
            is_replacing: false,
        }
    }
}

impl EventEmitter<ImageViewEvents> for ImageView {}

impl ImageView {
    /// Replaces the image cache so the next render reloads the file from disk.
    pub fn bust_cache(&mut self, cx: &mut Context<Self>) {
        self.image_cache = RetainAllImageCache::new(cx);
    }
}

impl Render for ImageView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let path = self.path.clone();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("Unknown"));

        let m_path = path.clone();
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
                            .child(
                                Button::new(format!("view-image-{}", self.index))
                                    .child(AppIcons::Eye)
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
