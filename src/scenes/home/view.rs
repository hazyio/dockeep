use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;
use rust_i18n::t;

use crate::components::app_settings::AppSettings;

pub struct Home;
impl Render for Home {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);

        div()
            .h_flex()
            .gap_2()
            .size_full()
            .child(TitleBar::new().child(div().child(t!("title.home"))))
            .children([Button::new("ok")
                .primary()
                .child(IconName::Settings2)
                .label("Let's Go!")
                .on_click(|_, window, cx| {
                    AppSettings::open(window, cx);
                })])
            .children(sheet_layer)
    }
}
