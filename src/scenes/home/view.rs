use anyhow::Error;
use gpui_kit::base::input::InputState;
use gpui_kit::component::button::*;
use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use rust_i18n::t;

use crate::components::add_project_dialog::AddProjectDialog;
use crate::components::project_info::{LoadedProjectInfo, ProjectInfo};
use crate::scenes::app::MyApp;
use crate::scenes::settings::view::SettingsPage;
use crate::utils::app_projects::AppProjects;
use crate::utils::prelude::path_exists_or_none;
pub struct Home {
    app: WeakEntity<MyApp>,
    search: Entity<InputState>,
    loading: bool,
    projects: Vec<LoadedProjectInfo>,
    error: Option<Error>,
    add_project_dialog: Entity<AddProjectDialog>,
}

impl Home {
    pub fn new(app: WeakEntity<MyApp>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("label.search_projects")));
        // kick off the background load
        Self::load_projects(cx);

        Self {
            app,
            search,
            loading: true,
            projects: Vec::new(),
            error: None,
            add_project_dialog: cx.new(|cx| AddProjectDialog::new(window, cx)),
        }
    }
    fn load_projects(cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_spawn(async move { AppProjects::load() })
                .await;
            let projects = result
                .0
                .iter()
                .map(|info| LoadedProjectInfo {
                    name: info.name.clone(),
                    path: path_exists_or_none(&info.path.clone()),
                })
                .collect();
            entity
                .update(cx, |this, cx| {
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
impl Render for Home {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.app.clone();
        let dialog_layer = Root::render_dialog_layer(window, cx);

        let loading = self.loading;
        let search = self.search.clone();
        let add_project_dialog = self.add_project_dialog.clone();
        let error = self.error.as_ref().map(|error| error.to_string());
        let project_to_show = self.projects;
        div()
            .v_flex()
            .gap_2()
            .size_full()
            .child(
                TitleBar::new()
                    .p_3()
                    .child(div().flex().items_center().child(t!("title.home")))
                    .child(
                        div().flex().items_center().child(
                            Button::new("back")
                                .ghost()
                                .child(IconName::Settings)
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
                        div().h_flex().gap_2().child(Input::new(&search)).child(
                            Button::new("add-project")
                                .label(t!("label.add_project"))
                                .on_click(move |_, window, cx| {
                                    let add_project_dialog = add_project_dialog.clone();

                                    window.open_dialog(cx, move |dialog, _, _| {
                                        dialog
                                            .title(t!("title.add_new_project"))
                                            .h_1_2()
                                            .w_1_3()
                                            .child(add_project_dialog.clone())
                                    })
                                }),
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
                                        .children(
                                            project_to_show
                                                .iter()
                                                .map(|project| ProjectInfo::new(project.clone())),
                                        ),
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
