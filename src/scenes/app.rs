use gpui_kit::component::*;
use gpui_kit::*;

pub struct MyApp {
    pub view: AnyView,
}
impl Render for MyApp {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().h_flex().gap_2().size_full().child(self.view.clone())
    }
}
