use gpui_kit::component::button::*;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::*;
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use rfd::AsyncFileDialog;

use crate::utils::{
    app_projects::{AppProjectError, AppProjectInfo, AppProjects},
    prelude::now_timestamp,
};

pub enum AddProjectDialogEvent {
    ProjectSaved,
}

#[derive(Clone)]
struct NewProjectInfo {
    path: Entity<InputState>,
    name: Entity<InputState>,
}
pub struct AddProjectDialog {
    new_project_info: Option<NewProjectInfo>,
    error: Option<SharedString>,
    _input_subscription: Option<Subscription>,
}
impl EventEmitter<AddProjectDialogEvent> for AddProjectDialog {}

impl AddProjectDialog {
    pub fn open(window: &mut Window, cx: &mut App, add_project_dialog: Entity<AddProjectDialog>) {
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title(t!("title.add_new_project"))
                .w_1_3()
                .child(add_project_dialog.clone())
        })
    }

    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            new_project_info: None,
            error: None,
            _input_subscription: None,
        }
    }

    fn select_folder_location(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |entity, cx| {
            let result = cx
                .background_spawn(async move {
                    // open select file window
                    AsyncFileDialog::new()
                        .set_title(t!("label.select_project_folder"))
                        .pick_folder()
                        .await
                })
                .await;

            if let Some(handle) = result {
                let folder_name: SharedString = handle
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string()
                    .into();
                let path: SharedString = handle.path().display().to_string().into();
                tracing::info!(?path, "Selected folder");

                entity
                    .update_in(cx, |this, window, cx| {
                        let path_state =
                            cx.new(|cx| InputState::new(window, cx).default_value(path));
                        let name_state: Entity<InputState> = cx.new(|cx| {
                            InputState::new(window, cx)
                                .pattern(regex::Regex::new(r"^[a-zA-Z0-9 ]*$").unwrap())
                                .default_value(folder_name)
                        });

                        let subscription =
                            cx.subscribe(&name_state, |this, _, event: &InputEvent, cx| {
                                // clear error on change
                                if matches!(event, InputEvent::Change) {
                                    this.error = None;
                                    cx.notify();
                                }
                            });

                        this.new_project_info = Some(NewProjectInfo {
                            path: path_state,
                            name: name_state,
                        });
                        this._input_subscription = Some(subscription);
                        cx.notify();
                    })
                    .ok();
            }
        })
        .detach();
    }
    fn save(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(new_project_info) = self.new_project_info.as_ref() {
            let value: SharedString = new_project_info.name.read(cx).value().clone().trim().into();
            if value.is_empty() {
                self.error = Some(SharedString::from(t!("error.name_cannot_be_empty")));
                cx.notify();
                return;
            }
            if value.len() < 2 {
                self.error = Some(SharedString::from(t!("error.name_too_short")));
                cx.notify();
                return;
            }
            match AppProjects::add_project(AppProjectInfo {
                name: value.to_string(),
                path: new_project_info.path.read(cx).value().clone().to_string(),
                last_accessed: now_timestamp(),
            }) {
                Ok(_) => {
                    tracing::info!("Project added successfully");
                    // send a project saved event, so any subscribers can reload projects
                    cx.emit(AddProjectDialogEvent::ProjectSaved);
                    // close the dialog
                    self.new_project_info = None;
                    self.error = None;
                    self._input_subscription = None;
                }
                Err(AppProjectError::AlreadyExists(name)) => {
                    self.error = Some(t!("error.project_already_exists", name = name).into());
                }
                Err(AppProjectError::Other(e)) => {
                    self.error = Some(e.to_string().into());
                }
                _ => {
                    self.error = Some(t!("error.unknown_error").into());
                }
            };
            cx.notify();
        }
    }
    fn cancel(&mut self, _: &ClickEvent, win: &mut Window, cx: &mut Context<Self>) {
        self.new_project_info = None;
        self.error = None;
        self._input_subscription = None;
        win.close_dialog(cx);
        cx.notify();
    }
}

impl Render for AddProjectDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .w_full()
            .when_none(&self.new_project_info, |el| {
                el.h(px(150.0)).justify_center().items_center().child(
                    Button::new("select-project")
                        .primary()
                        .label(t!("label.select_project_folder"))
                        .on_click(cx.listener(Self::select_folder_location)),
                )
            })
            .when_some(self.new_project_info.clone(), |el, info| {
                el.child(
                    div()
                        .mt_5()
                        .flex_1()
                        .v_flex()
                        .w_full()
                        .gap_3()
                        .child(Label::new(t!("label.project_name")))
                        .child(Input::new(&info.name))
                        .child(Label::new(t!("label.project_path")))
                        .child(Input::new(&info.path).readonly(true))
                        .child(
                            Label::new(self.error.clone().unwrap_or("default".into()))
                                .text_color(cx.theme().danger)
                                .when_some(self.error.clone(), |el, _| el.visible())
                                .when_none(&self.error.clone(), |el| el.invisible()),
                        )
                        .child(
                            div()
                                .h_flex()
                                .mt_auto()
                                .gap_2()
                                .child(
                                    Button::new("cancel")
                                        .secondary()
                                        .flex_grow_1()
                                        .ghost()
                                        .label(t!("label.cancel"))
                                        .on_click(cx.listener(Self::cancel)),
                                )
                                .child(
                                    Button::new("select-project")
                                        .secondary()
                                        .flex_grow_1()
                                        .label(t!("label.save"))
                                        .on_click(cx.listener(Self::save)),
                                ),
                        ),
                )
            })
    }
}
