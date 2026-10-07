use std::path::PathBuf;

use gpui_kit::{
    base::{Disableable, IndexPath, StyledExt, input::InputState},
    component::{
        ActiveTheme, WindowExt,
        button::{Button, ButtonVariants},
        dialog::{Dialog, DialogFooter, DialogHeader, DialogTitle},
        input::Input,
        label::Label,
        select::{Select, SelectState},
        switch::Switch,
    },
    prelude::FluentBuilder,
    *,
};
use std::path::Path;
use std::sync::Arc;

use rfd::FileDialog;

use crate::{config::project_settings_data::ProjectSettingsData, utils::app_icons::AppIcons};

pub struct ProjectSettings {
    project_path: PathBuf,
    data: ProjectSettingsData,
    save_format_state: Entity<SelectState<Vec<String>>>,
    save_to: Entity<InputState>,
    error: Option<String>,
}

impl ProjectSettings {
    pub fn new(project_path: &Path, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let project_settings = ProjectSettingsData::load(project_path);
        let save_value = crate::config::ImageFormat::all()
            .iter()
            .map(|lang| lang.to_value().to_string())
            .collect::<Vec<_>>();
        let selected_format_index = save_value
            .iter()
            .position(|f| f == project_settings.save_format.to_value())
            .unwrap_or(0);
        let state = cx.new(|cx| {
            SelectState::new(
                save_value,
                Some(IndexPath::new(selected_format_index)),
                window,
                cx,
            )
        });
        let make_save_to = project_path.join(project_settings.save_to_dir.clone());
        let save_to = cx.new(|cx| {
            InputState::new(window, cx).default_value(make_save_to.to_string_lossy().to_string())
        });

        Self {
            project_path: project_path.to_path_buf(),
            data: project_settings,
            save_format_state: state,
            save_to,
            error: None,
        }
    }
    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let project_path = self.project_path.clone();

        let save_to = self.save_to.update(cx, |input, _| input.value().clone());
        let save_format = self.save_format_state.read(cx).selected_value().unwrap();
        let rel_path = PathBuf::from(save_to.as_str())
            .strip_prefix(&project_path)
            .unwrap()
            .to_path_buf();
        let settings = ProjectSettingsData {
            save_to_dir: rel_path,
            save_format: crate::config::ImageFormat::from_value(save_format.as_str()),
            save_with_tab_title: self.data.save_with_tab_title,
            auto_commit: self.data.auto_commit,
        };
        settings.save(&project_path);
        self.data = settings;
        cx.notify();
    }

    fn select_save_to(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match FileDialog::new()
            .set_directory(&self.project_path)
            .pick_folder()
        {
            Some(folder) => {
                let project_path = self.project_path.clone();
                if !folder.starts_with(&project_path) {
                    self.error = Some(t!("error.selected_folder_not_in_project").to_string());
                    return;
                }

                self.save_to.update(cx, |input, cx| {
                    input.set_value(
                        SharedString::from(folder.to_string_lossy().to_string()),
                        window,
                        cx,
                    );
                });
                self.error = None;
            }
            None => {
                self.error = Some(t!("error.failed_to_select_save_to").to_string());
            }
        };
        cx.notify();
    }
}
impl Render for ProjectSettings {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        Dialog::new(cx)
            .trigger(Button::new("open-project-settings").child(AppIcons::Settings))
            .content(move |content, _, cx| {
                let entity = Arc::new(entity.clone());
                // read current state at the time the dialog renders
                let data = entity.read(cx).data.clone();
                let save_with_tab_title_check = data.save_with_tab_title;
                let error = entity.read(cx).error.clone();
                let save_disabled = error.is_some();
                let auto_commit_check = data.auto_commit;
                let save_format_state = entity.read(cx).save_format_state.clone();
                let entity1 = entity.clone();
                let entity2 = entity.clone();
                let entity3 = entity.clone();
                let save_entity = entity.clone();
                let save_to = entity.read(cx).save_to.clone();

                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child(t!("title.project_settings"))),
                    )
                    .when_some(error, |dialog, data| {
                        dialog.child(
                            div()
                                .v_flex()
                                .gap_2()
                                .mt_2()
                                .child(Label::new(data).text_color(cx.theme().danger)),
                        )
                    })
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
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .child(Input::new(&save_to).readonly(true))
                                    .child(
                                        Button::new("edit-project-settings-save-to")
                                            .child(AppIcons::Pencil)
                                            .on_click(move |_, window, cx| {
                                                entity3.update(cx, |this, cx| {
                                                    this.select_save_to(window, cx);
                                                });
                                            }),
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
                                    .label(t!("label.save"))
                                    .disabled(save_disabled)
                                    .on_click(move |_, window, cx| {
                                        save_entity.update(cx, |this, cx| {
                                            this.save_settings(cx);
                                            window.close_dialog(cx);
                                        });
                                    }),
                            ),
                    )
            })
    }
}
