use std::path::PathBuf;

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
struct Item {}
pub struct EditPage {
    app: WeakEntity<MyApp>,
    loading: bool,
    error: Option<Error>,
    path: PathBuf,
    title: String,
    items: Vec<Item>,
}

impl EditPage {
    pub fn new(
        path: PathBuf,
        title: String,
        app: WeakEntity<MyApp>,
        cx: &mut Context<Self>,
    ) -> Self {
        // kick off the background load
        Self::load_project(cx);

        Self {
            app,
            loading: true,
            error: None,
            path,
            title,
            items: Vec::new(),
        }
    }
    fn load_project(cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {}).detach();
    }
    fn header(&self) -> impl IntoElement {
        let app = self.app.clone();
        div().child(
            div()
                .h_flex()
                .child(div().flex_grow_1().child(self.title.clone()).text_3xl())
                .child(
                    Button::new("close-edit-project")
                        .danger()
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
        )
    }
}
impl Render for EditPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_layer = Root::render_dialog_layer(window, cx);

        let loading = self.loading;
        let error = self.error.as_ref().map(|error| error.to_string());
        div()
            .p_3()
            .size_full()
            .bg(cx.theme().background)
            .v_flex()
            .gap_2()
            .child(self.header())
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
