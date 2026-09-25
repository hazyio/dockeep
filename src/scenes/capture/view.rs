use std::path::PathBuf;

use gpui_kit::*;

pub struct CapturePage {
    pub path: Option<PathBuf>,
}
impl CapturePage {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

impl Render for CapturePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("This is a child")
    }
}
