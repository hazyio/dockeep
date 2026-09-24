use std::ops::Sub;
use std::path::PathBuf;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::utils::app_icons::AppIcons;
use crate::utils::prelude::open_in_file_explorer;
pub enum ImageViewEvents {
    Edit(PathBuf),
    OpenInFullscreen(PathBuf),
}

pub struct ImageView {
    pub path: PathBuf,
    pub index: usize,
}

impl ImageView {
    pub fn new(path: PathBuf, index: usize) -> Self {
        Self { path, index }
    }
}

impl EventEmitter<ImageViewEvents> for ImageView {}

impl Render for ImageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let window_width = window.viewport_size().width.sub(px(60.)); //add some padding
        let window_height = window.viewport_size().height.sub(px(60.)); //add some padding

        let path = self.path.clone();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("Unknown"));

        let m_path = path.clone();
        let m_index = self.index;
        div()
            .v_flex()
            .w(relative(1.))
            .h(px(250.))
            .cursor_pointer()
            .bg(cx.theme().muted)
            .rounded_md()
            .border_1()
            .child(img(path.clone()).flex_grow_1().w(relative(1.0)))
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
                                Button::new(format!("edit-image-{}", self.index))
                                    .child(AppIcons::Pencil)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let path = this.path.clone();
                                        tracing::debug!("Edit Image: {:?}", path);
                                        cx.emit(ImageViewEvents::Edit(path));
                                    }))
                                    .tooltip(t!("label.edit_image")),
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
                                Button::new(format!("open-image-popover-{}", self.index))
                                    .child(AppIcons::Expand)
                                    .tooltip(t!("label.open_in_fullscreen"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let path = this.path.clone();
                                        tracing::debug!("Edit Image: {:?}", path);
                                        cx.emit(ImageViewEvents::OpenInFullscreen(path));
                                    })),
                            ), // .child(
                               //     Popover::new(format!("popover-image-{}", self.index))
                               //         .anchor(Anchor::TopCenter)
                               //         // .appearance(false)
                               //         // .p_3()
                               //         .m_0()
                               //         .w(window_width)
                               //         .h(window_height)
                               //         .shadow_2xl()
                               //         .rounded_none()
                               //         .trigger(
                               //             Button::new(format!("open-image-popover-{}", self.index))
                               //                 .child(AppIcons::Expand)
                               //                 .tooltip(t!("label.open_in_fullscreen")),
                               //         )
                               //         .content(move |_, _, cx| {
                               //             div().h_flex().justify_end().child(
                               //                 Button::new(format!("close-popover-image-{}", m_index))
                               //                     .primary()
                               //                     .child(AppIcons::Close)
                               //                     .on_click(cx.listener(|_, _, _, cx| {
                               //                         cx.emit(DismissEvent);
                               //                     }))
                               //                     .tooltip(t!("label.close_popup")),
                               //             )
                               //         })
                               //         .child(img(path.clone()).flex_grow_1().w(relative(1.0))),
                               // ),
                    ),
            )
    }
}
