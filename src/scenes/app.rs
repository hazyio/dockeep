use gpui_kit::component::*;
use gpui_kit::*;

pub struct MyApp {
    pub view: AnyView,
}
impl MyApp {
    pub fn navigate_to(&mut self, view: AnyView, cx: &mut Context<Self>) {
        self.view = view;
        cx.notify();
    }
}

impl Render for MyApp {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().v_flex().gap_2().size_full().child(self.view.clone())
    }
}
