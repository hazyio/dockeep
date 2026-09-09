use std::path::PathBuf;

use gpui_kit::component::*;
use gpui_kit::*;
#[derive(Clone)]

pub struct LoadedProjectInfo {
    pub name: String,
    pub path: Option<PathBuf>,
}

#[derive(IntoElement)]
pub struct ProjectInfo {
    pub info: LoadedProjectInfo,
}
impl ProjectInfo {
    pub fn new(info: LoadedProjectInfo) -> Self {
        Self { info}
    }
}
impl RenderOnce for ProjectInfo {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .p_2()
            .rounded_md()
            .bg(cx.theme().secondary)
            .text_color(cx.theme().secondary_foreground)
            .flex_grow_1()
            .v_flex()
            .gap_2()
            .border_1()
            .child(self.info.name)
    }
}
