use gpui_kit::component::*;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct WindowDecor {
    title: String,
    before_decor: Vec<AnyElement>,
}

impl WindowDecor {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            before_decor: Vec::new(),
        }
    }

    pub fn before_decor(mut self, el: impl IntoElement) -> Self {
        self.before_decor.push(el.into_any_element());
        self
    }
}

impl RenderOnce for WindowDecor {
    fn render(
        self,
        _: &mut gpui_kit::Window,
        _: &mut gpui_kit::App,
    ) -> impl gpui_kit::prelude::IntoElement {
        TitleBar::new()
            .child(div().flex().items_center().child(self.title))
            .children(self.before_decor)
    }
}
