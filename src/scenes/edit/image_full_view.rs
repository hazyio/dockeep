use std::path::PathBuf;

use gpui_kit::component::popover::Popover;
use gpui_kit::component::*;
use gpui_kit::*;
pub struct ImageFullView {
    pub path: Option<PathBuf>,
}
impl ImageFullView {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}
impl Render for ImageFullView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let path = self.path.clone();

        Popover::new("image-full-view-popover")
            .open(path.is_some())
            .on_open_change(cx.listener(|this, _: &bool, _, cx| {
                // this.path = None;
                cx.notify();
            }))
            .child("This popover's open state is controlled programmatically.")
    }
}
