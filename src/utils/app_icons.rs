use gpui_kit::{App, IntoElement, RenderOnce, Window, component::Icon};
#[derive(IntoElement)]
pub enum AppIcons {
    Trash,
    Settings,
    Close
}

impl RenderOnce for AppIcons {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self {
            AppIcons::Trash => Icon::default().path("trash.svg"),
            AppIcons::Settings => Icon::default().path("settings.svg"),
            AppIcons::Close => Icon::default().path("close.svg"),
        }
    }
}
