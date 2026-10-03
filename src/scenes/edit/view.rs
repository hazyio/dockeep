use std::ops::{Div, Sub};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;

use anyhow::Error;
use gpui_kit::base::input::InputState;
use gpui_kit::component::button::*;
use gpui_kit::component::input::Input;
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use gpui_kit::{px, size};
use rust_i18n::t;

use crate::components::sort_button::{SortButton, SortButtonEvent};
use crate::components::window_decor::WindowDecor;
use crate::config::AppConfig;
use crate::scenes::app::MyApp;
use crate::scenes::edit::browser_actions::{BrowserActions, BrowserActionsEvents};
use crate::scenes::edit::image_view::{ImageView, ImageViewEvents};
use crate::scenes::home::view::HomePage;
use crate::utils::app_icons::AppIcons;
use crate::utils::files;
use crate::utils::git::{add_path_and_commit, remove_path_and_commit};

pub struct EditPage {
    is_git_repo: bool,
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
    search: Entity<InputState>,
    sort_button: Entity<SortButton>,
    _sort_button_subscription: Subscription,
}

impl EditPage {
    pub fn new(
        path: PathBuf,
        title: String,
        window: &mut Window,
        app: WeakEntity<MyApp>,
        cx: &mut Context<Self>,
        is_git_repo: bool,
    ) -> Self {
        // kick off the background load
        Self::load_project(cx, &path);
        let m_path = path.clone();
        let browser_action = cx.new(move |_| BrowserActions::new(m_path.clone()));
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("label.search_images")));
        // Sort button
        let sort_button = cx.new(|_| SortButton::new(true));
        let _sort_button_subscription =
            Self::build_sort_button_subscription(&sort_button, window, cx);

        Self {
            search,
            is_git_repo,
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
            sort_button,
            _sort_button_subscription,
        }
    }
    /// Sorts `projects` in place using `key`, then notifies the view.
    fn apply_sort(&mut self, cx: &mut Context<Self>) {
        let current_sort = self.sort_button.read(cx).current_sort();
        let ascending = match current_sort {
            SortButtonEvent::SortNameAscending | SortButtonEvent::SortLastAccessedAscending => true,
            SortButtonEvent::SortNameDescending | SortButtonEvent::SortLastAccessedDescending => {
                false
            }
        };
        let mut keyed: Vec<(String, Entity<ImageView>)> =
            self.items
                .iter()
                .map(|project| {
                    let project_enitty = project.read(cx);
                    let key = match current_sort {
                        SortButtonEvent::SortNameAscending
                        | SortButtonEvent::SortNameDescending => project_enitty
                            .path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_lowercase(),
                        SortButtonEvent::SortLastAccessedAscending
                        | SortButtonEvent::SortLastAccessedDescending => {
                            project_enitty.last_modified_timestamp.to_string()
                        }
                    };

                    (key, project.clone())
                })
                .collect();

        keyed.sort_by(move |a, b| {
            if ascending {
                a.0.cmp(&b.0)
            } else {
                b.0.cmp(&a.0)
            }
        });
        self.items = keyed.into_iter().map(|(_, project)| project).collect();
        cx.notify();
    }
    fn build_sort_button_subscription(
        entity: &Entity<SortButton>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe_in(
            entity,
            window,
            move |this, _button, _: &SortButtonEvent, _window, cx| {
                this.apply_sort(cx);
            },
        )
    }
    fn build_browser_action_subscription(
        entity: &Entity<BrowserActions>,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe(entity, move |this, _, event: &BrowserActionsEvents, cx| {
            let app_config = AppConfig::load();
            let repo_path = this.path.clone();
            let is_git_repo = this.is_git_repo;
            match event {
                BrowserActionsEvents::Add(p) => {
                    tracing::debug!("Adding Image: {:?}", p);
                    let entity = this.build_image_entity(p, this.items.len(), cx);
                    let subscription = this.build_image_subscription(&entity, cx);
                    this._image_action_subscription.push(subscription);
                    this.items.push(entity);
                    if app_config.git_setting.auto_commit && is_git_repo {
                        // if auto-commit is enabled, make a new commit for the added image
                        if let Err(e) =
                            add_path_and_commit(&repo_path, p, "description.add_screenshot")
                        {
                            tracing::error!("Failed to commit image: {:?}", e);
                        }
                    };
                    // apply sort after adding the image
                    this.apply_sort(cx);
                }
                BrowserActionsEvents::Replace(p) => {
                    tracing::debug!("Replacing Image: {:?}", p);
                    if app_config.git_setting.auto_commit && is_git_repo {
                        // if auto-commit is enabled, make a new commit for the replacement image
                        if let Err(e) =
                            add_path_and_commit(&repo_path, p, "description.replace_screenshot")
                        {
                            tracing::error!("Failed to commit image: {:?}", e);
                        }
                    };
                    if let Some(entity) = this.items.iter().find(|item| item.read(cx).path == *p) {
                        entity.update(cx, |image_view, cx| {
                            // Replace the cache so the next render picks up the new file contents.
                            image_view.bust_cache(cx);
                            // update the last modified timestamp
                            image_view.update_last_modified_with_now();
                            cx.notify();
                        });
                    }
                    // no need to sort after replacing an image, only data changed
                }
            }
        })
    }
    fn build_image_subscription(
        &mut self,
        entity: &Entity<ImageView>,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe(entity, |this, image_view, event: &ImageViewEvents, cx| {
            let app_config = AppConfig::load();
            let repo_path = this.path.clone();
            let is_git_repo = this.is_git_repo;
            match event {
                ImageViewEvents::Replace(path) => {
                    tracing::debug!("Editing Image: {:?}", path);
                    if let Some(active_replace) =
                        this.items.iter().find(|item| item.read(cx).is_replacing)
                    {
                        // remove the active replace path
                        active_replace.update(cx, |item, cx| {
                            item.is_replacing = false;
                            cx.notify();
                        });
                    };
                    this.browser_action.update(cx, |action, cx| {
                        action.queue_replace_path(path.clone());
                        cx.notify();
                    });
                    image_view.update(cx, |item, cx| {
                        item.is_replacing = true;
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
                    if app_config.git_setting.auto_commit && is_git_repo {
                        if let Err(e) = remove_path_and_commit(
                            &repo_path,
                            path.clone().as_path(),
                            "description.delete_screenshot",
                        ) {
                            tracing::error!("Failed to commit image: {:?}", e);
                        }
                    };
                    this.browser_action.update(cx, |action, cx| {
                        // cancel replace if the path matches
                        if let Some(replace_path) = &action.replace_path {
                            if replace_path.as_os_str() == path.as_os_str() {
                                action.cancel_replace();
                            }
                        }
                        cx.notify();
                    });
                    cx.notify();
                }
                ImageViewEvents::CancelReplace => {
                    this.browser_action.update(cx, |action, cx| {
                        action.cancel_replace();
                        cx.notify();
                    });
                    image_view.update(cx, |item, cx| {
                        item.is_replacing = false;
                        cx.notify();
                    });
                }
            }
        })
    }
    /// Builds an image entity for the given `path` and `index`.
    fn build_image_entity(
        &mut self,
        path: &PathBuf,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Entity<ImageView> {
        let path = path.clone();
        let last_modified_timestamp = files::last_modified(&path).unwrap_or(SystemTime::UNIX_EPOCH);
        let last_modified_timestamp = last_modified_timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        cx.new(move |cx| ImageView::new(path, last_modified_timestamp, index, cx))
    }
    /// Loads the project at `path` and populates `self.items` with the images.
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
                    entity.apply_sort(cx);
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
        let query = self.search.read(element_cx).value().to_lowercase();
        // get images to show
        let images_to_show: Vec<_> = self
            .items
            .iter()
            .filter(|p| {
                let info = p.read(element_cx);
                // use path to filter, so users can do /path/to/image
                query.is_empty() || info.path.to_string_lossy().to_lowercase().contains(&query)
            })
            .cloned()
            .collect();
        let error = self.error.as_ref().map(|error| error.to_string());
        let items = &self.items;
        let sizing = images_to_show.len();
        let rows = (sizing + cols - 1) / cols; // ceil division
        let row_height = px(250.);
        let col_width = width.div(cols as f32).sub(px(12.)); //remove gap

        let row_gap = px(8.);
        let row_size = size(col_width, row_height + row_gap);
        let item_sizes = Rc::new((0..rows).map(|_| row_size).collect());
        let show_image_full_view = self.show_image_full_view.clone();

        let search = self.search.clone();
        let sort_button = self.sort_button.clone();

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
                            .when(images_to_show.is_empty(), |cx| {
                                cx.child(
                                    div()
                                        .size_full()
                                        .v_flex()
                                        .justify_center()
                                        .items_center()
                                        .child(Label::new(t!("error.no_image_found"))),
                                )
                            })
                            .when(!images_to_show.is_empty(), |cx| {
                                cx.child(
                                    div()
                                        .h_flex()
                                        .gap_2()
                                        .child(Input::new(&search))
                                        .child(
                                            Button::new("open-project-settings")
                                                .child(AppIcons::Settings)
                                                .on_click(element_cx.listener(|this, _, _, cx| {
                                                    
                                                    // Self::load_projects(this.app.clone(), cx);
                                                })),
                                        )
                                        .child(sort_button),
                                )
                                .child(
                                    v_virtual_list(
                                        element_cx.entity().clone(),
                                        "my-list",
                                        item_sizes,
                                        move |_, visible_range, _, _| {
                                            visible_range
                                                .map(|row_ix| {
                                                    div().h_flex().gap_2().w_full().children(
                                                        (0..cols).filter_map(|col| {
                                                            let item_ix = row_ix * cols + col; // both usize
                                                            images_to_show.get(item_ix).map(
                                                                |item| {
                                                                    div()
                                                                        .w(col_width)
                                                                        .flex_none()
                                                                        .child(item.clone())
                                                                },
                                                            )
                                                        }),
                                                    )
                                                })
                                                .collect()
                                        },
                                    )
                                    .track_scroll(&self.scroll_handle),
                                )
                                .vertical_scrollbar(&self.scroll_handle)
                            })
                    }),
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
