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
use gpui_kit::{px, size};
use rust_i18n::t;

use crate::components::window_decor::WindowDecor;
use crate::scenes::app::MyApp;
use crate::scenes::edit::browser_actions::{BrowserActions, BrowserActionsEvents};
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
    show_image_full_view: Option<PathBuf>,
    fullscreen_image_cache: Entity<RetainAllImageCache>,
    items: Vec<Entity<ImageView>>,
    scroll_handle: VirtualListScrollHandle,
    _image_action_subscription: Vec<Subscription>,
    browser_action: Entity<BrowserActions>,
    _browser_action_subscription: Subscription,
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
        let m_path = path.clone();
        let browser_action = cx.new(move |_| BrowserActions::new(m_path.clone()));

        Self {
            app,
            loading: true,
            error: None,
            path,
            title,
            items: Vec::new(),
            scroll_handle: VirtualListScrollHandle::new(),
            _image_action_subscription: Vec::new(),
            show_image_full_view: None,
            _browser_action_subscription: Self::build_browser_action_subscription(
                &browser_action,
                cx,
            ),
            browser_action: browser_action,
            fullscreen_image_cache: RetainAllImageCache::new(cx),
        }
    }
    fn build_browser_action_subscription(
        entity: &Entity<BrowserActions>,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe(
            entity,
            |this, _, event: &BrowserActionsEvents, cx| match event {
                BrowserActionsEvents::Add(p) => {
                    tracing::debug!("Adding Image: {:?}", p);
                    tracing::info!("Adding Image: {:?}", p);
                    let entity = this.build_image_entity(p, this.items.len(), cx);
                    let subscription = this.build_image_subscription(&entity, cx);
                    this._image_action_subscription.push(subscription);
                    this.items.push(entity);

                    cx.notify();
                }
                BrowserActionsEvents::Replace(p) => {
                    tracing::debug!("Replacing Image: {:?}", p);
                    if let Some(entity) = this.items.iter().find(|item| item.read(cx).path == *p) {
                        entity.update(cx, |image_view, cx| {
                            // Replace the cache so the next render picks up the new file contents.
                            image_view.bust_cache(cx);
                            cx.notify();
                        });
                    }
                }
            },
        )
    }
    fn build_image_subscription(
        &mut self,
        entity: &Entity<ImageView>,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe(entity, |this, _, event: &ImageViewEvents, cx| match event {
            ImageViewEvents::Edit(path) => {
                tracing::debug!("Editing Image: {:?}", path);
                this.browser_action.update(cx, |action, cx| {
                    action.queue_replace_path(path.clone());
                    cx.notify();
                });
                cx.notify();
            }
            ImageViewEvents::OpenInFullscreen(path) => {
                tracing::debug!("Opening Image in Fullscreen: {:?}", path);
                this.show_image_full_view = Some(path.clone());
                cx.notify();
            }
            ImageViewEvents::Delete(path) => {
                tracing::debug!("Deleting Image: {:?}", path);
                this.items.retain(|item| item.read(cx).path != *path);
                cx.notify();
            }
        })
    }
    fn build_image_entity(
        &mut self,
        path: &PathBuf,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Entity<ImageView> {
        let path = path.clone();
        cx.new(move |cx| ImageView::new(path, index, cx))
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
                        let built_entity = entity.build_image_entity(path, index, cx);
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
        let dialog_layer = Root::render_dialog_layer(window, element_cx);
        let width = window.viewport_size().width.sub(px(12.)); //remove padding
        let height = window.viewport_size().height;

        let cols: usize = if width < px(640.) {
            1
        } else if width < px(1024.) {
            2
        } else {
            3
        };
        // generate doc for this
        let loading = self.loading;
        let error = self.error.as_ref().map(|error| error.to_string());
        let items = &self.items;
        let sizing = items.len();
        let rows = (sizing + cols - 1) / cols; // ceil division
        let row_height = px(250.);
        let col_width = width.div(cols as f32).sub(px(12.)); //remove gap

        let row_gap = px(8.);
        let row_size = size(col_width, row_height + row_gap);
        let item_sizes = Rc::new((0..rows).map(|_| row_size).collect());
        let show_image_full_view = self.show_image_full_view.clone();

        let app = self.app.clone();
        let browser_action = self.browser_action.clone();
        div()
            .size_full()
            .relative()
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
                    .relative()
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
                        cx.v_flex()
                            .size_full()
                            .gap_3()
                            .child(browser_action)
                            .when(items.is_empty(), |cx| {
                                cx.child(
                                    div()
                                        .size_full()
                                        .v_flex()
                                        .justify_center()
                                        .items_center()
                                        .child(Label::new(t!("error.no_image_found"))),
                                )
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
                    })
                    .vertical_scrollbar(&self.scroll_handle),
            )
            .when_some(show_image_full_view, |this, value| {
                this.child(
                    div()
                        .id("image-full-view-backdrop")
                        .absolute()
                        .inset_0() // top/right/bottom/left = 0
                        .v_flex()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .p_4()
                        .bg(element_cx.theme().background.opacity(0.9)) // dim backdrop
                        .occlude() // block clicks reaching content below
                        .child(
                            div().size_full().v_flex().child(
                                div()
                                    .flex_grow_1()
                                    .v_flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        img(value)
                                            .image_cache(&self.fullscreen_image_cache)
                                            .max_h(height.sub(px(50.)))
                                            .max_w(width.sub(px(50.))),
                                    ),
                            ),
                        )
                        .on_click(element_cx.listener(|this, _, _, cx| {
                            this.show_image_full_view = None;
                            // rebuild the cache to clear any existing images, very inefficient but works for now as it would be more complex to clear individual images.
                            this.fullscreen_image_cache = RetainAllImageCache::new(cx);
                            cx.notify();
                        })),
                )
            })
            .children(dialog_layer)
    }
}
