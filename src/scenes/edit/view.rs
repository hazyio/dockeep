use anyhow::Error;
use gpui_kit::component::button::*;
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use rust_i18n::t;

use crate::scenes::app::MyApp;
use crate::scenes::home::view::HomePage;
use crate::utils::app_icons::AppIcons;

pub struct EditPage {
    app: WeakEntity<MyApp>,
    loading: bool,
    error: Option<Error>,
}

impl EditPage {
    pub fn new(app: WeakEntity<MyApp>, cx: &mut Context<Self>) -> Self {
        // kick off the background load
        Self::load_project(cx);

        Self {
            app,
            loading: true,
            error: None,
        }
    }
    fn load_project(cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {}).detach();
    }
}
impl Render for EditPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.app.clone();
        let dialog_layer = Root::render_dialog_layer(window, cx);

        let loading = self.loading;
        let error = self.error.as_ref().map(|error| error.to_string());
        div()
            .size_full()
            .bg(cx.theme().background)
            .v_flex()
            .gap_2()
            .child(
                TitleBar::new()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .child("Editing Project Helo Wolrd"),
                    )
                    .child(
                        div().flex().items_center().child(
                            Button::new("Close")
                                .ghost()
                                .child(AppIcons::Close)
                                .child(t!("label.close"))
                                .on_click(move |_, window, cx| {
                                    if let Some(app) = app.upgrade() {
                                        let settings_view: AnyView = cx
                                            .new(|cx| HomePage::new(app.downgrade(), window, cx))
                                            .into();
                                        app.update(cx, |app, cx| {
                                            app.navigate_to(settings_view, cx);
                                        });
                                    }
                                }),
                        ),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .mt_10()
                    .when_some(error.clone(), |cx, error| {
                        cx.child(div().child(error.to_string()))
                    })
                    .when(!loading && error.clone().is_none(), |cx| {
                        cx.child(
                            div()
                                .pb_1_4()
                                .overflow_y_scrollbar()
                                .size_full()
                                .grid()
                                .grid_cols(3)
                                .gap_2()
                                .child(Label::new("It should work")),
                        )
                    }),
            )
            .children(dialog_layer)
    }
}
