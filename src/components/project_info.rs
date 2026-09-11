use std::path::PathBuf;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::label::Label;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::scenes::app::MyApp;
use crate::scenes::edit::view::EditPage;
use crate::utils::app_icons::AppIcons;
use crate::utils::app_projects::{AppProjectInfo, AppProjects};
use crate::utils::git::GitRepoInfo;
use crate::utils::prelude::open_in_file_explorer;
pub enum ProjectInfoEvent {
    Delete(String),
}

#[derive(Clone)]

pub struct ProjectInfo {
    pub app: WeakEntity<MyApp>,
    pub index: u16,
    pub name: String,
    pub path: PathBuf,
    pub repo_info: Option<GitRepoInfo>,
    pub last_accessed: String,
    pub open_delete_dialog: bool,
}
impl EventEmitter<ProjectInfoEvent> for ProjectInfo {}

impl Render for ProjectInfo {
    fn render(&mut self, _: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open_delete_dialog;
        let entity = element_cx.entity().downgrade();
        let open_entity = entity.clone();
        let ok_entity = entity.clone();
        let cancel_entity = entity.clone();
        let action_entity = entity.clone();
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
            .child(AlertDialog::new(cx)
                                .open(open)
                                .on_open_change(move |open, _, _, cx| {
                                    _ = entity.update(cx, |this, cx| {
                                        this.alert_dialog_open = open;
                                        cx.notify();
                                    });
                                })
                                .on_ok(move |_, _, cx| {
                                    _ = ok_entity.update(cx, |this, cx| {
                                        this.alert_dialog_open = false;
                                        cx.notify();
                                    });
                                    true
                                })
                                .backdrop(
                                    AlertDialogBackdrop::new()
                                        .absolute()
                                        .inset_0()
                                        .bg(super::example_rgb(0x000000))
                                        .opacity(0.18),
                                ) .popup(
                                                       AlertDialogPopup::new()
                                                           .flex()
                                                           .items_center()
                                                           .justify_center()
                                                           .child(
                                                               div()
                                                                   .w_72()
                                                                   .p_3()
                                                                   .bg(super::example_rgb(0xffffff))
                                                                   .border_1()
                                                                   .border_color(super::example_rgb(0x171717))
                                                                   .child(
                                                                       AlertDialogTitle::new()
                                                                           .child("Delete project?"),
                                                                   )
                                                                   .child(
                                                                       AlertDialogDescription::new()
                                                                           .mt_2()
                                                                           .text_xs()
                                                                           .text_color(super::example_rgb(0x525252))
                                                                           .child(
                                                                               "This permanently deletes Acme Studio and all of its data.",
                                                                           ),
                                                                   )  .child(
                                                                                                         div()
                                                                                                             .mt_3()
                                                                                                             .flex()
                                                                                                             .justify_end()
                                                                                                             .gap_2()
                                                                                                             .child(AlertDialogCancel::new().child(
                                                                                                                 Button::new("cancel-delete")
                                                                                                                     .px_3()
                                                                                                                     .h_7()
                                                                                                                     .flex()
                                                                                                                     .items_center()
                                                                                                                     .text_xs()
                                                                                                                     .border_1()
                                                                                                                     .border_color(super::example_rgb(0xd4d4d4))
                                                                                                                     .on_click(move |_, _, cx| {
                                                                                                                         _ = cancel_entity.update(cx, |this, cx| {
                                                                                                                             this.alert_dialog_open = false;
                                                                                                                             cx.notify();
                                                                                                                         });
                                                                                                                     })
                                                                                                                     .child("Cancel"),
                                                                                                             )) .child(AlertDialogAction::new().child(
                                                                                                                                                            Button::new("confirm-delete")
                                                                                                                                                                .px_3()
                                                                                                                                                                .h_7()
                                                                                                                                                                .flex()
                                                                                                                                                                .items_center()
                                                                                                                                                                .text_xs()
                                                                                                                                                                .border_1()
                                                                                                                                                                .border_color(super::example_rgb(0x171717))
                                                                                                                                                                .bg(super::example_rgb(0x171717))
                                                                                                                                                                .text_color(super::example_rgb(0xffffff))
                                                                                                                                                                .on_click(move |_, _, cx| {
                                                                                                                                                                    _ = action_entity.update(cx, |this, cx| {
                                                                                                                                                                        this.alert_dialog_open = false;
                                                                                                                                                                        cx.notify();
                                                                                                                                                                    });
                                                                                                                                                                })
                                                                                                                                                                .child("Delete"),
                                                                                                                                                        )))
    }
}
impl ProjectInfo {
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
        let index = self.index.clone();
        let path = self.path.to_string_lossy().to_string();
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
                        .on_click(cx.listener(move |_, _, _, cx| {
                            if let Err(e) = AppProjects::remove_project(&path) {
                                tracing::warn!("remove_project failed: {e:?}");
                            }
                            cx.emit(ProjectInfoEvent::Delete(path.clone()));
                        })),
                )
                .text_lg(),
        )
    }
    fn body(&self, cx: &mut App) -> impl IntoElement {
        let path = self.path.clone();
        let index = self.index.clone();
        let last_accessed = self.last_accessed.clone();
        let app = self.app.clone();
        let project_info = AppProjectInfo {
            name: self.name.clone(),
            path: self.path.to_string_lossy().to_string(),
            last_accessed: 0,
        };
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
                                Label::new(t!("label.last_accessed", datetime = last_accessed))
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
                            .on_click(move |_, _, _| {
                                let _ = open_in_file_explorer(path.as_path());
                            })
                    })
                    .child(
                        Button::new(format!("open-in-editor-{}", index))
                            .label(t!("label.open_in_editor"))
                            .primary()
                            .child(AppIcons::ExternalLink)
                            .on_click(move |_, _, cx| {
                                project_info.update_lastaccess();
                                if let Some(app) = app.upgrade() {
                                    let settings_view: AnyView =
                                        cx.new(|cx| EditPage::new(app.downgrade(), cx)).into();
                                    app.update(cx, |app, cx| {
                                        app.navigate_to(settings_view, cx);
                                    });
                                }
                            }),
                    ),
            )
    }
}
