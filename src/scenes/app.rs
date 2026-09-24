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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .size_full()
            .child(self.view.clone())
            // Render the notification layer on top of the app content
            .children(notification_layer)
    }
}
