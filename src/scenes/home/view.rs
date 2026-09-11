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

use crate::components::add_project_dialog::{AddProjectDialog, AddProjectDialogEvent};
use crate::components::project_info::{ProjectInfo, ProjectInfoEvent};
use crate::scenes::app::MyApp;
use crate::scenes::settings::view::SettingsPage;
use crate::utils::app_icons::AppIcons;
use crate::utils::app_projects::AppProjects;
use crate::utils::git::get_git_repo_info;
use crate::utils::prelude::to_human_datetime;
pub struct HomePage {
    app: WeakEntity<MyApp>,
    search: Entity<InputState>,
    loading: bool,
    projects: Vec<Entity<ProjectInfo>>,
    error: Option<Error>,
    add_project_dialog: Entity<AddProjectDialog>,
    _project_saved_subscription: Subscription,
    _project_delete_subscriptions: Vec<Subscription>,
}

impl HomePage {
    pub fn new(app: WeakEntity<MyApp>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("label.search_projects")));
        let app_for_subscription = app.clone();

        // kick off the background load
        Self::load_projects(app.clone(), cx);
        let add_project_dialog = cx.new(|cx| AddProjectDialog::new(window, cx));
        // watch for project saved events
        let _project_saved_subscription = cx.subscribe_in(
            &add_project_dialog,
            window,
            move |_this, _dialog, event: &AddProjectDialogEvent, window, cx| match event {
                AddProjectDialogEvent::ProjectSaved => {
                    window.close_dialog(cx);
                    HomePage::load_projects(app_for_subscription.clone(), cx);
                }
            },
        );

        Self {
            app,
            search,
            loading: true,
            projects: Vec::new(),
            error: None,
            add_project_dialog: add_project_dialog,
            _project_saved_subscription,
            _project_delete_subscriptions: Vec::new(),
        }
    }
    fn load_projects(app: WeakEntity<MyApp>, cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_spawn(async move { AppProjects::load() })
                .await;
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
                            last_accessed: to_human_datetime(project.last_accessed),
                            open_delete_dialog: false,
                        }
                    })
                })
                .collect();
            entity
                .update(cx, |this, cx| {
                    this._project_delete_subscriptions = projects
                        .iter()
                        .map(|project_entity| {
                            cx.subscribe(project_entity, |this, _, event: &ProjectInfoEvent, cx| {
                                match event {
                                    ProjectInfoEvent::Delete(path) => {
                                        this.projects.retain(|p| {
                                            p.read(cx).path.to_string_lossy().as_ref()
                                                != path.as_str()
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
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }
}
impl Render for HomePage {
    fn render(&mut self, window: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.app.clone();

        let dialog_layer = Root::render_dialog_layer(window, element_cx);

        let loading = self.loading;
        let search = self.search.clone();
        let add_project_dialog = self.add_project_dialog.clone();
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
                TitleBar::new()
                    .p_4()
                    .child(div().flex().items_center().child(t!("title.home")))
                    .child(
                        div().flex().items_center().child(
                            Button::new("back")
                                .ghost()
                                .child(AppIcons::Settings)
                                .on_click(move |_, _, cx| {
                                    if let Some(app) = app.upgrade() {
                                        let settings_view: AnyView =
                                            cx.new(|_| SettingsPage::new(app.downgrade())).into();
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
                                        AddProjectDialog::open(
                                            window,
                                            cx,
                                            add_project_dialog.clone(),
                                        );
                                    }),
                            )
                            .child(
                                Button::new("reload-projects")
                                    .child(AppIcons::Refresh)
                                    .on_click(element_cx.listener(|this, _, _, cx| {
                                        Self::load_projects(this.app.clone(), cx);
                                    })),
                            ),
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
                                        .grid_cols(3)
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
