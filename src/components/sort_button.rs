use gpui_kit::base::StyledExt;
use gpui_kit::component::{ActiveTheme, button::*};
use gpui_kit::component::popover::Popover;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use rust_i18n::t;

use crate::utils::app_icons::AppIcons;
#[derive(Debug, Clone, PartialEq)]
pub enum SortButtonEvent {
    SortNameAscending,
    SortNameDescending,
    SortLastAccessedAscending,
    SortLastAccessedDescending,
}
pub struct SortButton {
    current_sort: SortButtonEvent,
}
impl EventEmitter<SortButtonEvent> for SortButton {}
impl SortButton {
    pub fn new() -> Self {
        Self {
            current_sort: SortButtonEvent::SortNameAscending,
        }
    }
}
impl Render for SortButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("sort-projects-popover")
            .w(px(200.0))
            .trigger(Button::new("sort-projects").child(AppIcons::SlidersHorizontal))
            .child(
                div()
                    .id("sort-by-name")
                    .h_flex()
                    .gap_1()
                    .child(div().flex_grow_1().child(t!("label.name")))
                    .hover(|e| e.bg(cx.theme().muted))
                    .p_1()
                    .px_2()
                    .rounded_md()
                    .when(
                        self.current_sort != SortButtonEvent::SortNameAscending
                            && self.current_sort != SortButtonEvent::SortNameDescending,
                        |e| {
                            // when the current sort is not name ascending or descending
                            e.on_click(cx.listener(|sort_button, _, _, cx| {
                                sort_button.current_sort = SortButtonEvent::SortNameAscending;
                                cx.emit(SortButtonEvent::SortNameAscending);
                                cx.notify();
                            }))
                        },
                    )
                    .when(
                        self.current_sort == SortButtonEvent::SortNameAscending,
                        |e| {
                            e.child(AppIcons::ListSortAscending).on_click(cx.listener(
                                |sort_button, _, _, cx| {
                                    sort_button.current_sort = SortButtonEvent::SortNameDescending;
                                    cx.emit(SortButtonEvent::SortNameDescending);
                                    cx.notify();
                                },
                            ))
                        },
                    )
                    .when(
                        self.current_sort == SortButtonEvent::SortNameDescending,
                        |e| {
                            e.child(AppIcons::ListSortDescending).on_click(cx.listener(
                                |sort_button, _, _, cx| {
                                    sort_button.current_sort = SortButtonEvent::SortNameAscending;
                                    cx.emit(SortButtonEvent::SortNameAscending);
                                    cx.notify();
                                },
                            ))
                        },
                    ),
            )
            .child(
                div()
                    .id("sort-by-last-accessed")
                    .h_flex()
                    .gap_1()
                    .child(div().flex_grow_1().child(t!("label.last_accessed")))
                    .hover(|e| e.bg(cx.theme().muted))
                    .p_1()
                    .px_2()
                    .rounded_md()
                    .when(
                        self.current_sort != SortButtonEvent::SortLastAccessedAscending
                            && self.current_sort != SortButtonEvent::SortLastAccessedDescending,
                        |e| {
                            // when the current sort is not last accessed ascending or descending
                            e.on_click(cx.listener(|sort_button, _, _, cx| {
                                sort_button.current_sort =
                                    SortButtonEvent::SortLastAccessedAscending;
                                cx.emit(SortButtonEvent::SortLastAccessedAscending);
                                cx.notify();
                            }))
                        },
                    )
                    .when(
                        self.current_sort == SortButtonEvent::SortLastAccessedAscending,
                        |e| {
                            e.child(AppIcons::ListSortAscending).on_click(cx.listener(
                                |sort_button, _, _, cx| {
                                    sort_button.current_sort =
                                        SortButtonEvent::SortLastAccessedDescending;
                                    cx.emit(SortButtonEvent::SortLastAccessedDescending);
                                    cx.notify();
                                },
                            ))
                        },
                    )
                    .when(
                        self.current_sort == SortButtonEvent::SortLastAccessedDescending,
                        |e| {
                            e.child(AppIcons::ListSortDescending).on_click(cx.listener(
                                |sort_button, _, _, cx| {
                                    sort_button.current_sort =
                                        SortButtonEvent::SortLastAccessedAscending;
                                    cx.emit(SortButtonEvent::SortLastAccessedAscending);
                                    cx.notify();
                                },
                            ))
                        },
                    ),
            )
    }
}
