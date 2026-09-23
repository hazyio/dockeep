use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
#[derive(IntoElement)]
pub struct WindowDecor {
    title: String,
    before_decor: Option<AnyElement>,
}
impl WindowDecor {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            before_decor: None,
        }
    }

    pub fn before_decor(mut self, el: impl IntoElement) -> Self {
        self.before_decor = Some(el.into_any_element());
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
            .py_4()
            .child(div().flex().items_center().child(self.title))
            .when_some(self.before_decor, |parent, value| parent.child(value))
    }
}
