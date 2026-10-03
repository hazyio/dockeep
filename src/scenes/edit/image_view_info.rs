use std::path::PathBuf;

use gpui_kit::base::input::InputState;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::Input;
use gpui_kit::component::popover::Popover;
use gpui_kit::component::separator::Separator;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::utils::app_icons::AppIcons;

pub struct ImageViewInfo {
    path: PathBuf,
    edit_name: Option<Entity<InputState>>,
}
impl ImageViewInfo {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            edit_name: None,
        }
    }
}
impl Render for ImageViewInfo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.edit_name.is_none() {
            let default_name = self.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            //  first render, create the edit_name input state. very inefficient the best way it to accpet window in [`Self::new`] but it has a cascading change.
            let edit_name = cx.new(|cx| InputState::new(window, cx).default_value(default_name));
            self.edit_name = Some(edit_name);
        }
        Popover::new("open-image-info-popover")
            .anchor(Anchor::TopCenter)
            .trigger(
                Button::new("open-image-info")
                    .outline()
                    .child(AppIcons::Info)
                    .tooltip(t!("label.open_image_info")),
            )
            .when_some(self.edit_name.clone(), |cx, edit_name| {
                cx.child(t!("label.file_name"))
                    .child(Input::new(&edit_name))
                    .child(Separator::horizontal())
                    .child(t!("label.captured_from"))
                    .child("https://google.com")
                    .child(Separator::horizontal())
                    .child(t!("label.created_at"))
                    .child("12/05/2026 11:44am")
                    .child(Separator::horizontal())
                    .child(t!("label.updated_at"))
                    .child("12/05/2026 11:44am")
            })
    }
}
