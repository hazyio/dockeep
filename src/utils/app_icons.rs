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
    Pencil,
    Expand,
    Eye,
    SquareExclamationPoint,
    Minus,
    CircleDot,
    Play,
    PlayOff,
    Monitor,
    Smartphone,
    Crop,
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
            AppIcons::Pencil => Icon::default().path("pencil.svg"),
            AppIcons::Expand => Icon::default().path("expand.svg"),
            AppIcons::Eye => Icon::default().path("eye.svg"),
            AppIcons::SquareExclamationPoint => {
                Icon::default().path("square-exclamation-point.svg")
            }
            AppIcons::Minus => Icon::default().path("minus.svg"),
            AppIcons::CircleDot => Icon::default().path("circle-dot.svg"),
            AppIcons::Play => Icon::default().path("play.svg"),
            AppIcons::PlayOff => Icon::default().path("play-off.svg"),
            AppIcons::Monitor => Icon::default().path("monitor.svg"),
            AppIcons::Smartphone => Icon::default().path("smartphone.svg"),
            AppIcons::Crop => Icon::default().path("crop.svg"),
        }
    }
}
