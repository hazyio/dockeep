use gpui_kit::{
    base::{IndexPath, StyledExt, input::InputState},
    component::{
        WindowExt,
        button::{Button, ButtonVariants},
        dialog::{Dialog, DialogFooter, DialogHeader, DialogTitle},
        input::Input,
        label::Label,
        select::{Select, SelectState},
        switch::Switch,
    },
    *,
};
use serde::{Deserialize, Serialize};

use crate::utils::app_icons::AppIcons;
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ProjectSettingsData {
    pub save_with_tab_title: bool,
    pub auto_commit: bool,
}
pub struct ProjectSettings {
    pub data: ProjectSettingsData,
    pub save_format_state: Entity<SelectState<Vec<String>>>,
    pub save_to: Entity<InputState>,
}

impl ProjectSettings {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let save_value = crate::config::ImageFormat::all()
            .iter()
            .map(|lang| lang.to_value().to_string())
            .collect::<Vec<_>>();
        let state = cx.new(|cx| {
            SelectState::new(
                save_value,
                Some(IndexPath::default()), // Select first item
                window,
                cx,
            )
        });
        let save_to = cx.new(|cx| InputState::new(window, cx).default_value("./"));

        Self {
            data: ProjectSettingsData {
                save_with_tab_title: true,
                auto_commit: true,
            },
            save_format_state: state,
            save_to,
        }
    }
    fn select_save_to(&mut self) {}
}
impl Render for ProjectSettings {
    fn render(&mut self, window: &mut Window, parent_cx: &mut Context<Self>) -> impl IntoElement {
        let entity = parent_cx.entity();
        
        Dialog::new(parent_cx)
            .trigger(Button::new("open-project-settings").child(AppIcons::Settings))
            .content(move |content, _, cx| {
                // read current state at the time the dialog renders
                let data = entity.read(cx).data.clone();
                let save_with_tab_title_check = data.save_with_tab_title;
                let auto_commit_check = data.auto_commit;
                let save_format_state = entity.read(cx).save_format_state.clone();
                let entity1 = entity.clone();
                let entity2 = entity.clone();
                let save_to = entity.read(cx).save_to.clone();
                
                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child(t!("title.project_settings"))),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_2()
                            .mt_2()
                            .child(Label::new(t!("label.image_save_format")))
                            .child(Select::new(&save_format_state))
                            .child(
                                Switch::new("switch-save-with-tab-title")
                                    .mt_3()
                                    .checked(save_with_tab_title_check)
                                    .label(t!("label.save_with_tab_title").to_string())
                                    .on_click(move |checked: &bool, _window, cx| {
                                        let checked = *checked;
                                        entity1.update(cx, |this, cx| {
                                            this.data.save_with_tab_title = checked;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                Switch::new("switch-auto-commit")
                                    .checked(auto_commit_check)
                                    .label(t!("label.auto_commit").to_string())
                                    .on_click(move |checked: &bool, _window, cx| {
                                        let checked = *checked;
                                        entity2.update(cx, |this, cx| {
                                            this.data.auto_commit = checked;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(Label::new(t!("label.save_image_to")))
                            .child(
                                div().h_flex().gap_2().child(Input::new(&save_to)).child(
                                    Button::new("edit-project-settings-save-to")
                                        .child(AppIcons::Pencil)
                                        .on_click(parent_cx.listener(|_, _, _, cx| {})),
                                ),
                            ),
                    )
                    .child(
                        DialogFooter::new()
                            .child(
                                Button::new("close-project-settings")
                                    .outline()
                                    .label(t!("label.close"))
                                    .on_click(|_, window, cx| window.close_dialog(cx)),
                            )
                            .child(
                                Button::new("save-project-settings")
                                    .primary()
                                    .label(t!("label.save")),
                            ),
                    )
            })
    }
}
