use std::ops::{Div, Sub};
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

use crate::components::window_decor::WindowDecor;
use crate::scenes::app::MyApp;
use crate::scenes::edit::image_full_view::ImageFullView;
use crate::scenes::edit::image_view::{ImageView, ImageViewEvents};
use crate::scenes::home::view::HomePage;
use crate::utils::app_icons::AppIcons;
use crate::utils::files;

pub struct EditPage {
    app: WeakEntity<MyApp>,
    loading: bool,
    error: Option<Error>,
    pub path: PathBuf,
    title: String,
    image_full_view: Entity<ImageFullView>,
    items: Vec<Entity<ImageView>>,
    scroll_handle: VirtualListScrollHandle,
    _image_action_subscription: Vec<Subscription>,
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
            _image_action_subscription: Vec::new(),
            image_full_view: cx.new(|_| ImageFullView::new(None)),
        }
    }
    fn build_image_subscription(
        &mut self,
        entity: &Entity<ImageView>,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe(entity, |this, _, event: &ImageViewEvents, cx| match event {
            ImageViewEvents::Edit(path) => {
                tracing::debug!("Editing Image: {:?}", path);
            }
        })
    }
    fn load_project(cx: &mut Context<Self>, path: &PathBuf) {
        let path = path.clone();
        cx.spawn(async move |entity, cx| {
            // Runs on a background thread, why? UI stays responsive.

            let paths = cx
                .background_spawn(async move { files::read_images(&path) })
                .await;
            // Back on the foreground executor.
            entity
                .update(cx, |entity, cx| {
                    let mut entities = Vec::new();
                    let mut entities_subscriptions = Vec::new();
                    for (index, path) in paths.iter().enumerate() {
                        // build the entity and subscribe to it
                        let built_entity = cx.new(move |_| ImageView::new(path.clone(), index));
                        entities_subscriptions
                            .push(entity.build_image_subscription(&built_entity, cx));
                        entities.push(built_entity);
                    }
                    entity.items = entities;
                    entity._image_action_subscription = entities_subscriptions;
                    entity.loading = false;
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }
}
impl Render for EditPage {
    fn render(&mut self, window: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        let width = window.viewport_size().width.sub(px(12.)); //remove padding

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
        let col_width = width.div(cols as f32).sub(px(12.)); //remove gap

        let row_gap = px(8.);
        let row_size = size(col_width, row_height + row_gap);
        let app = self.app.clone();
        let item_sizes = Rc::new((0..rows).map(|_| row_size).collect());
        div()
            .size_full()
            .child(
                WindowDecor::new(t!("label.editing_project", name = self.title.clone()))
                    .before_decor(
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
            .child(
                div()
                    .p_3()
                    .pb_12()
                    .size_full()
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
                                    move |view, visible_range, _, _| {
                                        visible_range
                                            .map(|row_ix| {
                                                div().h_flex().gap_2().w_full().children(
                                                    (0..cols).filter_map(|col| {
                                                        let item_ix = row_ix * cols + col; // both usize
                                                        view.items.get(item_ix).map(|item| {
                                                            div()
                                                                .w(col_width)
                                                                .flex_none()
                                                                .child(item.clone())
                                                        })
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
