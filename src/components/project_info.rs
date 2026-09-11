use gpui_kit::component::label::Label;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::utils::app_icons::AppIcons;
use crate::utils::prelude::is_git_repo;
pub enum ProjectInfoEvent {
    Delete(String),
}
#[derive(Clone)]
struct LoadedProjectInfo {
    name: String,
    is_git: bool,
}
#[derive(Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    loading: bool,
    info: Option<LoadedProjectInfo>,
}
impl EventEmitter<ProjectInfoEvent> for ProjectInfo {}

impl ProjectInfo {
    pub fn new(name: String, path: String, cx: &mut Context<Self>) -> Self {
        // load in background
        Self::load(cx);
        Self {
            name,
            path,
            loading: true,
            info: None,
        }
    }
    fn load(cx: &mut Context<Self>) {
        cx.spawn(async move |entity, cx| {
            let (name, path) = entity
                .read_with(cx, |info, _| {
                    // read the name and path from the info
                    (
                        info.name.clone(),
                        std::path::Path::new(&info.path).to_path_buf(),
                    )
                })
                .unwrap(); // this should never fail

            if !path.exists() {
                // if the path doesn't exist, clear the info and stop loading
                entity
                    .update(cx, |this, cx| {
                        this.info = None;
                        this.loading = false;
                        cx.notify();
                    })
                    .ok();
                return;
            }
            let result = cx
                .background_spawn(async move {
                    let is_git = is_git_repo(&path);
                    LoadedProjectInfo {
                        is_git,
                        name: name.to_string(),
                    }
                })
                .await;
            entity
                .update(cx, |this, cx| {
                    this.info = Some(result);
                    this.loading = false;
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }
}
impl Render for ProjectInfo {
    fn render(&mut self, _: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        let loading = self.loading;
        let info = &self.info;
        div()
            .p_2()
            .rounded_md()
            .bg(element_cx.theme().muted)
            .text_color(element_cx.theme().muted_foreground)
            .flex_grow_1()
            .v_flex()
            .gap_2()
            .border_1()
            .when(loading, |cx| {
                cx.v_flex()
                    .items_center()
                    .justify_center()
                    .child(Spinner::new().with_size(px(32.0)))
            })
            .when_none(info, |cx| cx.v_flex().child(title(&self.name, element_cx)))
            .when_some(info.clone(), |cx, value| {
                cx.child(title(&value.name, element_cx))
                    .child(body(&value.name))
            })
    }
}

fn title(name: &str, cx: &mut Context<ProjectInfo>) -> impl IntoElement {
    div().child(
        div()
            .h_flex()
            .child(Label::new(name).font_bold().flex_grow_1())
            .text_color(cx.theme().danger)
            .child(div().child(AppIcons::Trash).cursor_pointer()),
    )
}
fn body(name: &str) -> impl IntoElement {
    div()
        .p_2()
        .v_flex()
        .gap_2()
        .border_1()
        .flex_grow_1()
        .child(Label::new(name).font_bold())
}
