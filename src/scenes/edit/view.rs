use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Error;
use gpui_kit::component::button::*;
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use gpui_kit::{Pixels, Size, px, size};
use rust_i18n::t;

use crate::scenes::app::MyApp;
use crate::scenes::edit::edit_item::EditItem;
use crate::scenes::home::view::HomePage;
use crate::utils::app_icons::AppIcons;
use crate::utils::files;

pub struct EditPage {
    app: WeakEntity<MyApp>,
    loading: bool,
    error: Option<Error>,
    pub path: PathBuf,
    title: String,
    items: Vec<Entity<EditItem>>,
    scroll_handle: VirtualListScrollHandle,
}

impl EditPage {
    pub fn new(
        path: PathBuf,
        title: String,
        app: WeakEntity<MyApp>,
        cx: &mut Context<Self>,
    ) -> Self {
        // kick off the background load
        Self::load_project(cx, &path);

        Self {
            app,
            loading: true,
            error: None,
            path,
            title,
            items: Vec::new(),
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
    fn load_project(cx: &mut Context<Self>, path: &PathBuf) {
        let path = path.clone();
        cx.spawn(async move |entity, cx| {
            // let mut sizing = 0;
            let reads = files::read_images(&path)
                .iter()
                .map(|f| {
                    cx.new(|_| {
                        // sizing = sizing + 1;
                        EditItem { path: f.clone() }
                    })
                })
                .collect();
            // let item_sizes = Rc::new((0..sizing).map(|_| size(px(200.), px(30.))).collect());

            entity
                .update(cx, |entity, cx| {
                    entity.loading = false;
                    entity.items = reads;
                    // entity.item_sizes = item_sizes;
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }
    fn header(&self) -> impl IntoElement {
        let app = self.app.clone();
        div().child(
            div()
                .h_flex()
                .child(Label::new(self.title.clone()).text_3xl().flex_grow_1())
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
    fn render(&mut self, window: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        let width = window.viewport_size().width;

        let cols: usize = if width < px(640.) {
            1
        } else if width < px(1024.) {
            2
        } else {
            3
        };
        let dialog_layer = Root::render_dialog_layer(window, element_cx);

        let loading = self.loading;
        let error = self.error.as_ref().map(|error| error.to_string());
        let items = &self.items;
        let sizing = items.len();
        let rows = (sizing + cols - 1) / cols; // ceil division
        let row_height = px(250.);
        let row_width = px(150.);
        let row_gap = px(6.);
        let row_size = size(row_width, row_height + row_gap);
        let item_sizes = Rc::new((0..rows).map(|_| row_size).collect());
        div()
            .p_3()
            .size_full()
            .bg(element_cx.theme().background)
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
                    .when(loading, |cx| {
                        cx.v_flex()
                            .justify_center()
                            .items_center()
                            .child(Spinner::new().with_size(px(48.0)))
                            .child(Label::new(t!("label.loading_project")))
                    })
                    .when(!loading && error.clone().is_none(), |cx| {
                        cx.when(items.is_empty(), |cx| {
                            cx.v_flex()
                                .justify_center()
                                .items_center()
                                .child(Label::new(t!("error.no_image_found")))
                        })
                        .when(!items.is_empty(), |cx| {
                            cx.child(
                                v_virtual_list(
                                    element_cx.entity().clone(),
                                    "my-list",
                                    item_sizes,
                                    move |view, visible_range, _, cx| {
                                        visible_range
                                            .map(|row_ix| {
                                                div().h_flex().gap_2().w_full().children(
                                                    (0..cols).filter_map(|col| {
                                                        let item_ix = row_ix * cols + col; // both usize
                                                        view.items
                                                            .get(item_ix)
                                                            .map(|item| item.clone())
                                                    }),
                                                )
                                            })
                                            .collect()
                                    },
                                )
                                .track_scroll(&self.scroll_handle),
                            )
                        })
                    }),
            )
            .children(dialog_layer)
    }
}
