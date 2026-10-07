use std::path::PathBuf;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::label::Label;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::config::AppConfig;
use crate::files::files::open_in_file_explorer;
use crate::scenes::app::MyApp;
use crate::scenes::edit::view::EditPage;
use crate::utils::app_icons::AppIcons;
use crate::utils::app_projects::AppProjectInfo;
use crate::utils::git::GitRepoInfo;
pub enum ProjectInfoEvent {
    Delete(PathBuf),
}

#[derive(Clone)]

pub struct ProjectInfo {
    pub app: WeakEntity<MyApp>,
    pub index: u16,
    pub name: String,
    pub path: PathBuf,
    pub repo_info: Option<GitRepoInfo>,
    pub last_accessed_datetime: String,
    pub last_accessed_timestamp: u64,
}
impl EventEmitter<ProjectInfoEvent> for ProjectInfo {}

impl Render for ProjectInfo {
    fn render(&mut self, _: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_3()
            .rounded_md()
            .bg(element_cx.theme().muted.divide(0.6))
            .flex_grow_1()
            .v_flex()
            .gap_2()
            .border_1()
            .child(self.title(element_cx))
            .child(self.repo_info())
            .child(self.body(element_cx))
    }
}
impl ProjectInfo {
    fn open_in_project_editor(
        name: String,
        path: PathBuf,
        repo_info: Option<GitRepoInfo>,
        window: &mut Window,
        cx: &mut App,
        app: WeakEntity<MyApp>,
    ) {
        if !path.is_dir() || !path.exists() {
            window.open_alert_dialog(cx, |dialog, _, _| {
                dialog
                    .title(t!("title.project_invalid"))
                    .description(t!("description.project_invalid"))
                    .footer(
                        div().h_flex().justify_end().child(
                            Button::new("button.ok").label(t!("label.ok")).on_click(
                                |_, window, cx| {
                                    window.close_dialog(cx);
                                },
                            ),
                        ),
                    )
            });
            return;
        }
        let project_info = AppProjectInfo {
            name: name.clone(),
            path: path.to_string_lossy().to_string(),
            last_accessed_datetime: 0,
        };
        project_info.update_lastaccess();
        let is_git_repo = repo_info.is_some();
        if let Some(app) = app.upgrade() {
            let settings_view: AnyView = cx
                .new(|cx| {
                    EditPage::new(path.clone(), name, window, app.downgrade(), cx, is_git_repo)
                })
                .into();
            app.update(cx, |app, cx| {
                app.navigate_to(settings_view, cx);
            });
        }
    }

    fn open_delete_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.clone();
        let entity = cx.entity().downgrade();
        let action_entity = entity.clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let action_entity = action_entity.clone();
            dialog
                .title(t!("dialog.delete_title"))
                .child(t!("dialog.delete_description", name = name))
                .footer(
                    div()
                        .h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("cancel-delete")
                                .outline()
                                .label(t!("label.cancel"))
                                .on_click(|_, window, cx| {
                                    window.close_dialog(cx);
                                }),
                        )
                        .child(
                            Button::new("confirm-delete")
                                .danger()
                                .label(t!("label.delete"))
                                .on_click(move |_, window, cx| {
                                    _ = action_entity.update(cx, |this, cx| {
                                        cx.emit(ProjectInfoEvent::Delete(this.path.clone()));

                                        cx.notify();
                                    });
                                    window.close_dialog(cx);
                                }),
                        ),
                )
        });
    }

    fn repo_info(&self) -> impl IntoElement {
        let repo_info = self.repo_info.clone();
        div()
            .h_flex()
            .gap_2()
            .when_some(repo_info.clone(), |cx, value| {
                cx.child(
                    Tag::color(ColorName::Neutral)
                        .child(AppIcons::GitCommit)
                        .gap_1()
                        .child(format!("#{}", value.branch)),
                )
                .when(!value.dirty, |cx| {
                    cx.child(
                        Tag::success()
                            .gap_1()
                            .child(AppIcons::Check)
                            .child(t!("label.clean")),
                    )
                })
                .when(value.dirty, |cx| {
                    cx.child(
                        Tag::danger()
                            .child(AppIcons::Close)
                            .gap_1()
                            .child(t!("label.dirty")),
                    )
                })
            })
            .when_none(&repo_info, |cx| {
                cx.child(
                    Tag::color(ColorName::Neutral)
                        .gap_1()
                        .child(AppIcons::GitCommit)
                        .child(t!("label.not_repo")),
                )
            })
    }
    fn title(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let name = self.name.clone();
        let index = self.index;
        let danger = cx.theme().danger;
        div().child(
            div()
                .h_flex()
                .mb_2()
                .child(Label::new(name).font_bold().flex_grow_1())
                .child(
                    Button::new(format!("project-item-{}", index))
                        .outline()
                        .child(AppIcons::Trash)
                        .cursor_pointer()
                        .text_color(danger)
                        .on_click(
                            cx.listener(move |this, _, win, cx| this.open_delete_dialog(win, cx)),
                        ),
                )
                .text_lg(),
        )
    }
    fn body(&self, cx: &mut App) -> impl IntoElement {
        let name = self.name.clone();
        let path = self.path.clone();
        let index = self.index;
        let last_accessed_datetime = self.last_accessed_datetime.clone();
        let app = self.app.clone();
        let repo_info = self.repo_info.clone();
        div()
            .p_2()
            .v_flex()
            .gap_2()
            .border_1()
            .flex_grow_1()
            .child(
                div()
                    .v_flex()
                    .gap_3()
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .p_2()
                            .child(AppIcons::Folder)
                            .child(Label::new(path.to_string_lossy()))
                            .bg(cx.theme().background)
                            .rounded_md(),
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .p_2()
                            .child(AppIcons::RotateCcwClock)
                            .child(
                                Label::new(t!(
                                    "label.last_accessed_datetime",
                                    datetime = last_accessed_datetime
                                ))
                                .text_sm(),
                            )
                            .bg(cx.theme().background)
                            .rounded_md(),
                    )
                    .my(px(10.0))
                    .mb(px(25.0))
                    .rounded_md(),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .justify_between()
                    .child({
                        let path = path.clone();
                        Button::new(format!("open-in-local-{}", index))
                            .label(t!("label.open_in_file_manager"))
                            .outline()
                            .on_click(move |_, window, cx| {
                                window.push_notification(t!("label.opening_in_file_manager"), cx);

                                let _ = open_in_file_explorer(path.as_path());
                            })
                    })
                    .child(
                        Button::new(format!("open-in-editor-{}", index))
                            .label(t!("label.open_in_editor"))
                            .primary()
                            .on_click(move |_, window, cx| {
                                let is_dirty = repo_info.as_ref().is_some_and(|r| r.dirty);
                                let app_config = AppConfig::load();
                                if is_dirty && app_config.git_setting.auto_commit {
                                    let name = name.clone();
                                    let path = path.clone();
                                    let repo_info = repo_info.clone();
                                    let app = app.clone();
                                    window.open_alert_dialog(cx, move |alert, _, _| {
                                        let name = name.clone();
                                        let path = path.clone();
                                        let repo_info = repo_info.clone();
                                        let app = app.clone();
                                        alert
                                            .title(t!("dialog.dirty_repo_title"))
                                            .description(t!("dialog.dirty_repo_description"))
                                            .show_cancel(true)
                                            .on_ok(move |_, window, cx| {
                                                Self::open_in_project_editor(
                                                    name.clone(),
                                                    path.clone(),
                                                    repo_info.clone(),
                                                    window,
                                                    cx,
                                                    app.clone(),
                                                );
                                                true // Return true to close dialog
                                            })
                                    });
                                } else {
                                    Self::open_in_project_editor(
                                        name.clone(),
                                        path.clone(),
                                        repo_info.clone(),
                                        window,
                                        cx,
                                        app.clone(),
                                    );
                                }
                            }),
                    ),
            )
    }
}
