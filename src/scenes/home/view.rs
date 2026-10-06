use anyhow::Error;
use gpui_kit::component::button::*;
use gpui_kit::component::input::Input;
use gpui_kit::component::input::InputState;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use rust_i18n::t;

use crate::components::sort_button::SortButton;
use crate::components::sort_button::SortButtonEvent;
use crate::components::window_decor::WindowDecor;
use crate::config::AppConfig;
use crate::scenes::app::MyApp;
use crate::scenes::home::add_project_dialog::{AddProjectDialog, AddProjectDialogEvent};
use crate::scenes::home::project_info::ProjectInfo;
use crate::scenes::home::project_info::ProjectInfoEvent;
use crate::scenes::settings::view::SettingDefaultOpen;
use crate::utils::app_icons::AppIcons;
use crate::utils::app_projects::AppProjects;
use crate::utils::git::get_git_repo_info;
use crate::utils::prelude::open_settings;
use crate::utils::prelude::to_human_datetime;

pub struct HomePage {
    app: WeakEntity<MyApp>,
    search: Entity<InputState>,
    loading: bool,
    projects: Vec<Entity<ProjectInfo>>,
    error: Option<Error>,
    add_project_dialog: Entity<AddProjectDialog>,
    sort_button: Entity<SortButton>,
    _project_saved_subscription: Subscription,
    _project_delete_subscriptions: Vec<Subscription>,
    _sort_button_subscription: Subscription,
}

impl HomePage {
    pub fn new(app: WeakEntity<MyApp>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("label.search_projects")));
        let app_for_subscription = app.clone();

        // kick off the background load
        Self::load_projects(app.clone(), cx);
        // Add project dialog
        let add_project_dialog = cx.new(|cx| AddProjectDialog::new(window, cx));
        // watch for project saved events
        let _project_saved_subscription = cx.subscribe_in(
            &add_project_dialog,
            window,
            move |_this, _dialog, event: &AddProjectDialogEvent, window, cx| match event {
                AddProjectDialogEvent::ProjectSaved => {
                    // reload projects when any saved
                    window.close_dialog(cx);
                    HomePage::load_projects(app_for_subscription.clone(), cx);
                }
            },
        );
        // Sort button
        let sort_button =
            cx.new(|_| SortButton::new(SortButtonEvent::SortLastAccessedDescending, true));
        let _sort_button_subscription =
            Self::build_sort_button_subscription(&sort_button, window, cx);

        Self {
            app,
            search,
            loading: true,
            projects: Vec::new(),
            error: None,
            add_project_dialog: add_project_dialog,
            _project_saved_subscription,
            _project_delete_subscriptions: Vec::new(),
            _sort_button_subscription,
            sort_button,
        }
    }
    fn build_sort_button_subscription(
        entity: &Entity<SortButton>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe_in(
            entity,
            window,
            move |this, _button, _: &SortButtonEvent, _window, cx| this.apply_sort(cx),
        )
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
        let mut keyed: Vec<(String, Entity<ProjectInfo>)> =
            self.projects
                .iter()
                .map(|project| {
                    let project_enitty = project.read(cx);
                    let key = match current_sort {
                        SortButtonEvent::SortNameAscending
                        | SortButtonEvent::SortNameDescending => project_enitty.name.to_lowercase(),
                        SortButtonEvent::SortLastAccessedAscending
                        | SortButtonEvent::SortLastAccessedDescending => {
                            project_enitty.last_accessed_timestamp.to_string()
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
        self.projects = keyed.into_iter().map(|(_, project)| project).collect();
        cx.notify();
    }

    fn load_projects(app: WeakEntity<MyApp>, cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {
            // start loading
            entity
                .update(cx, |entity, cx| {
                    entity.loading = true;
                    cx.notify();
                })
                .ok();
            let result = cx
                .background_spawn(async move { AppProjects::load() })
                .await;
            let config = AppConfig::load();
            let projects: Vec<Entity<ProjectInfo>> = result
                .0
                .iter()
                .enumerate()
                .map(|(index, project)| {
                    cx.new(|_| {
                        let app = app.clone();

                        let path = std::path::Path::new(&project.path).to_path_buf();
                        ProjectInfo {
                            app: app,
                            index: index as u16,
                            name: project.name.clone(),
                            repo_info: get_git_repo_info(&path),
                            path,
                            last_accessed_datetime: to_human_datetime(
                                project.last_accessed_datetime,
                                config.time_format,
                                config.date_format,
                            ),
                            last_accessed_timestamp: project.last_accessed_datetime,
                        }
                    })
                })
                .collect();
            entity
                .update(cx, |this, cx| {
                    // attach delete subscriptions
                    this._project_delete_subscriptions = projects
                        .iter()
                        .map(|project_entity| {
                            cx.subscribe(project_entity, |this, _, event: &ProjectInfoEvent, cx| {
                                match event {
                                    ProjectInfoEvent::Delete(path) => {
                                        // remove project from filesystem
                                        let _ = AppProjects::remove_project(path);
                                        // remove project from view
                                        this.projects.retain(|p| {
                                            p.read(cx).path.as_path() != path.as_path()
                                        });
                                        cx.notify();
                                    }
                                }
                            })
                        })
                        .collect();
                    this.projects = projects;
                    this.loading = false;
                    this.error = result.1;
                    // apply default sort, cx.notify() is called automatically
                    this.apply_sort(cx);
                })
                .ok();
        })
        .detach();
    }
}

impl Render for HomePage {
    fn render(&mut self, window: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        let width = window.viewport_size().width;

        let cols = if width < px(640.) {
            1
        } else if width < px(1024.) {
            2
        } else {
            3
        };

        let dialog_layer = Root::render_dialog_layer(window, element_cx);

        let loading = self.loading;
        let search = self.search.clone();
        let add_project_dialog = self.add_project_dialog.clone();
        let sort_button = self.sort_button.clone();

        let error = self.error.as_ref().map(|error| error.to_string());
        let query = self.search.read(element_cx).value().to_lowercase();
        let project_to_show: Vec<_> = self
            .projects
            .iter()
            .filter(|p| {
                let info = p.read(element_cx);
                query.is_empty()
                    || info.name.to_lowercase().contains(&query)
                    || info.path.to_string_lossy().to_lowercase().contains(&query)
            })
            .cloned()
            .collect();
        div()
            .v_flex()
            .gap_2()
            .size_full()
            .child(
                WindowDecor::new(t!("title.home")).before_decor(
                    Button::new("back")
                        .ghost()
                        .child(AppIcons::Settings)
                        .on_click(move |_, window, cx| {
                            open_settings(SettingDefaultOpen::General, window, cx);
                        }),
                ),
            )
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .p_2()
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .child(Input::new(&search))
                            .child(
                                Button::new("add-project")
                                    .label(t!("label.add_project"))
                                    .on_click(move |_, window, cx| {
                                        let add_project_dialog = add_project_dialog.clone();
                                        AddProjectDialog::open(window, cx, add_project_dialog);
                                    }),
                            )
                            .child(
                                Button::new("reload-projects")
                                    .child(AppIcons::Refresh)
                                    .on_click(element_cx.listener(|this, _, _, cx| {
                                        Self::load_projects(this.app.clone(), cx);
                                    })),
                            )
                            .child(sort_button),
                    )
                    .size_full()
                    .child(
                        div()
                            .size_full()
                            .mt_10()
                            .when(error.is_some(), |cx| {
                                let error = error.clone().unwrap();
                                cx.child(div().child(error.to_string()))
                            })
                            .when(!loading && !project_to_show.is_empty(), |cx| {
                                cx.child(
                                    div()
                                        .id("project-grid")
                                        .pb_1_4()
                                        .overflow_y_scrollbar()
                                        .size_full()
                                        .grid()
                                        .grid_cols(cols)
                                        .gap_2()
                                        .children(project_to_show.clone()),
                                )
                            })
                            .when(!loading && project_to_show.is_empty(), |cx| {
                                cx.v_flex()
                                    .size_full()
                                    .items_center()
                                    .justify_center()
                                    .child(t!("label.no_projects_found"))
                            })
                            .when(loading, |cx| {
                                cx.v_flex()
                                    .items_center()
                                    .justify_center()
                                    .child(Spinner::new().with_size(px(48.0)))
                                    .child(t!("label.loading_projects"))
                            }),
                    ),
            )
            .children(dialog_layer)
    }
}
