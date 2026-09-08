use gpui_kit::component::*;
use gpui_kit::*;

use crate::utils::app_projects::AppProjectInfo;

#[derive(IntoElement)]
pub struct ProjectInfo {
    pub info: AppProjectInfo,
}
impl ProjectInfo {
    pub fn new(info: AppProjectInfo) -> Self {
        Self { info }
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
