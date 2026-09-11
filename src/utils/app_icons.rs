use gpui_kit::{App, IntoElement, RenderOnce, Window, component::Icon};
#[derive(IntoElement)]
pub enum AppIcons {
    Trash,
    Settings,
    Close,
    ExternalLink,
    Folder,
    GitCommit,
    Check,
    RotateCcwClock,
    Refresh,
}

impl RenderOnce for AppIcons {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self {
            AppIcons::Trash => Icon::default().path("trash.svg"),
            AppIcons::Settings => Icon::default().path("settings.svg"),
            AppIcons::Close => Icon::default().path("close.svg"),
            AppIcons::ExternalLink => Icon::default().path("external-link.svg"),
            AppIcons::Folder => Icon::default().path("folder.svg"),
            AppIcons::GitCommit => Icon::default().path("git-commit-horizontal.svg"),
            AppIcons::Check => Icon::default().path("check.svg"),
            AppIcons::RotateCcwClock => Icon::default().path("rotate-ccw-clock.svg"),
            AppIcons::Refresh => Icon::default().path("refresh.svg"),
        }
    }
}
