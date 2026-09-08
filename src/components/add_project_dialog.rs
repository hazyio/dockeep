use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;
#[derive(IntoElement)]
pub struct AddProjectDialog {}
impl AddProjectDialog {
    pub fn new() -> Self {
        Self {}
    }
}
impl RenderOnce for AddProjectDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .v_flex()
            .flex_1()
            .min_h_0()
            .size_full()
            .items_center()
            .justify_center()
            .child(Button::new("select-project").label(t!("label.select_project_folder")))
    }
}
