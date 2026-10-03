use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

pub struct Assets;
impl Assets {
    fn keys(&self) -> Vec<&'static str> {
        self.all_icons().iter().map(|i| i.0).collect()
    }
    fn all_icons(&self) -> &'static [(&'static str, &'static [u8])] {
        &[
            ("trash.svg", include_bytes!("../../assets/svg/trash.svg")),
            ("icon.svg", include_bytes!("../../assets/icon.svg")),
            ("loader.svg", include_bytes!("../../assets/svg/loader.svg")),
            (
                "settings.svg",
                include_bytes!("../../assets/svg/settings.svg"),
            ),
            ("search.svg", include_bytes!("../../assets/svg/search.svg")),
            (
                "chevron-down.svg",
                include_bytes!("../../assets/svg/chevron-down.svg"),
            ),
            ("check.svg", include_bytes!("../../assets/svg/check.svg")),
            ("close.svg", include_bytes!("../../assets/svg/close.svg")),
            (
                "window-close.svg",
                include_bytes!("../../assets/svg/close.svg"),
            ),
            (
                "external-link.svg",
                include_bytes!("../../assets/svg/external-link.svg"),
            ),
            ("expand.svg", include_bytes!("../../assets/svg/expand.svg")),
            ("eye.svg", include_bytes!("../../assets/svg/eye.svg")),
            ("folder.svg", include_bytes!("../../assets/svg/folder.svg")),
            (
                "git-commit-horizontal.svg",
                include_bytes!("../../assets/svg/git-commit-horizontal.svg"),
            ),
            (
                "rotate-ccw-clock.svg",
                include_bytes!("../../assets/svg/rotate-ccw-clock.svg"),
            ),
            (
                "refresh.svg",
                include_bytes!("../../assets/svg/refresh.svg"),
            ),
            ("pencil.svg", include_bytes!("../../assets/svg/pencil.svg")),
            (
                "square-exclamation-point.svg",
                include_bytes!("../../assets/svg/square-exclamation-point.svg"),
            ),
            ("minus.svg", include_bytes!("../../assets/svg/minus.svg")),
            (
                "window-minimize.svg",
                include_bytes!("../../assets/svg/minus.svg"),
            ),
            (
                "window-maximize.svg",
                include_bytes!("../../assets/svg/window-maximize.svg"),
            ),
            (
                "window-restore.svg",
                include_bytes!("../../assets/svg/window-restore.svg"),
            ),
            (
                "circle-dot.svg",
                include_bytes!("../../assets/svg/circle-dot.svg"),
            ),
            ("play.svg", include_bytes!("../../assets/svg/play.svg")),
            (
                "play-off.svg",
                include_bytes!("../../assets/svg/play-off.svg"),
            ),
            (
                "monitor.svg",
                include_bytes!("../../assets/svg/monitor.svg"),
            ),
            (
                "smartphone.svg",
                include_bytes!("../../assets/svg/smartphone.svg"),
            ),
            ("crop.svg", include_bytes!("../../assets/svg/crop.svg")),
            ("plus.svg", include_bytes!("../../assets/svg/plus.svg")),
            ("undo-2.svg", include_bytes!("../../assets/svg/undo-2.svg")),
            (
                "chevron-right.svg",
                include_bytes!("../../assets/svg/chevron-right.svg"),
            ),
            (
                "refresh-cw.svg",
                include_bytes!("../../assets/svg/refresh-cw.svg"),
            ),
            (
                "replace.svg",
                include_bytes!("../../assets/svg/replace.svg"),
            ),
            (
                "sliders-horizontal.svg",
                include_bytes!("../../assets/svg/sliders-horizontal.svg"),
            ),
            (
                "arrow-up-wide-narrow.svg",
                include_bytes!("../../assets/svg/arrow-up-wide-narrow.svg"),
            ),
            (
                "list-sort-ascending.svg",
                include_bytes!("../../assets/svg/list-sort-ascending.svg"),
            ),
            (
                "list-sort-descending.svg",
                include_bytes!("../../assets/svg/list-sort-descending.svg"),
            ),
            ("info.svg", include_bytes!("../../assets/svg/info.svg")),
        ]
    }
    fn find(&self, path: &str) -> Option<&'static [u8]> {
        let name = path.rsplit('/').next().unwrap_or(path);
        self.all_icons()
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, bytes)| *bytes)
    }
}
impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.len() < 1 {
            return Ok(None);
        }
        let bytes = match self.find(path) {
            Some(e) => e,
            None => {
                tracing::info!("path {} not found", path);

                self.find("square-exclamation-point.svg").unwrap()
            }
        };
        let ss = Cow::Owned(bytes.to_vec());
        Ok(Some(ss))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(self
            .keys()
            .into_iter()
            .filter(|p| p.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}
