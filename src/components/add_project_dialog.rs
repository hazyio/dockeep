use gpui_kit::base::input::*;
use gpui_kit::component::button::*;
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use rfd::AsyncFileDialog;
#[derive(Clone)]
struct NewProjectInfo {
    path: SharedString,
    name: Entity<InputState>,
}
pub struct AddProjectDialog {
    new_project_info: Option<NewProjectInfo>,
}

impl AddProjectDialog {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            new_project_info: None,
        }
    }

    fn select_folder_location(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(t!("label.project_name"))
                .pattern(regex::Regex::new(r"^[a-zA-Z0-9]*$").unwrap())
        });
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_spawn(async move {
                    AsyncFileDialog::new()
                        .set_title(t!("label.select_project_folder"))
                        .pick_folder()
                        .await
                })
                .await;

            if let Some(handle) = result {
                let path: SharedString = handle.path().display().to_string().into();
                tracing::info!(?path, "Selected folder");
                let new_project_info = NewProjectInfo {
                    path,
                    name: name_state,
                };
                entity
                    .update(cx, |this, cx| {
                        this.new_project_info = Some(new_project_info);
                        cx.notify();
                    })
                    .ok();
            }
        })
        .detach();
    }
}

impl Render for AddProjectDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .flex_1()
            .min_h_0()
            .size_full()
            .when_none(&self.new_project_info, |el| {
                el.items_center().justify_center().child(
                    div()
                        .v_flex()
                        .flex_1()
                        .min_h_0()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .child(
                            Button::new("select-project")
                                .label(t!("label.select_project_folder"))
                                .on_click(cx.listener(Self::select_folder_location)),
                        ),
                )
            })
            .when_some(self.new_project_info.clone(), |el, info| {
                let path_input = cx.new(|cx| InputState::new(window, cx).default_value(info.path));

                el.mt_3().gap_3()
                    .child(
                        input_wrapper(cx, &t!("label.project_name")).child(Input::new(&info.name)),
                    )
                    .child(
                        input_wrapper(cx, &t!("label.project_path")).child(Input::new(&path_input)),
                    )
                    .child(
                        Button::new("select-project")
                            .secondary()
                            .label(t!("label.save")),
                    )
            })
    }
}

fn input_wrapper(cx: &mut App, label: &str) -> Div {
    div()
        .border_b_2()
        .p_2()
        .w_full()
        .border_2()
        .border_color(cx.theme().border)
        .bg(cx.theme().secondary)
        .rounded(cx.theme().radius)
        .hover(|style| style.border_color(cx.theme().button_primary_hover))
        .in_focus(|style| style.border_color(cx.theme().button_primary_hover))
        .child(Label::new(label))
        .into()
}
