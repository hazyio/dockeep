use std::path::PathBuf;

use gpui_kit::component::*;
use gpui_kit::{component::label::Label, *};

pub struct EditItem {
    pub path: PathBuf,
}

impl EditItem {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
impl Render for EditItem {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .flex_1()
            .w(relative(1.))
            .h(relative(0.8))
            .justify_center()
            .items_center()
            .bg(cx.theme().muted)
            .rounded_md()
            .child(
                img(self.path.clone())
                    .rounded_md()
                    .max_w(relative(1.))
                    .h(px(210.)),
            )
    }
}
