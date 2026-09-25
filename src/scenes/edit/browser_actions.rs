use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::oneshot;
use gpui_kit::component::button::Button;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

use crate::utils::app_config::AppConfig;
use crate::utils::app_icons::AppIcons;
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserActionsState {
    Stopped,
    Starting,
    Running,
}

struct ScreenshotSize {
    width: u32,
    height: u32,
}

pub struct BrowserActions {
    state: BrowserActionsState,
    browser: Option<Arc<Browser>>,
}

impl BrowserActions {
    pub fn new() -> Self {
        Self {
            state: BrowserActionsState::Stopped,
            browser: None,
        }
    }
    fn is_browser_alive(browser: &Browser) -> bool {
        browser.get_version().is_ok()
    }
    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.state {
            BrowserActionsState::Stopped => {
                let chrome_path = AppConfig::load().chrome_path;
                if chrome_path.is_empty() {
                    window.push_notification(t!("error.chrome_path_not_found"), cx);
                    return;
                }
                let chrome_path = PathBuf::from(chrome_path);
                if !chrome_path.exists() {
                    window.push_notification(t!("error.chrome_path_not_found"), cx);
                    return;
                }

                self.state = BrowserActionsState::Starting;
                cx.notify();
                // Launching Chrome blocks until the DevTools port is ready,
                // so it has to run off the main thread.
                let (tx, rx) = oneshot::channel::<anyhow::Result<Browser>>();
                std::thread::spawn(move || {
                    let result = LaunchOptionsBuilder::default()
                        .headless(false)
                        .path(Some(chrome_path))
                        .build()
                        .map_err(anyhow::Error::from)
                        .and_then(|opts| Browser::new(opts).map_err(anyhow::Error::from));

                    let _ = tx.send(result);
                });
                cx.spawn(async move |this: WeakEntity<BrowserActions>, cx| {
                    let result = rx.await;

                    this.update(cx, |this, cx| {
                        match result {
                            Ok(Ok(browser)) => {
                                this.browser = Some(Arc::new(browser));
                                this.state = BrowserActionsState::Running;
                            }
                            Ok(Err(e)) => {
                                tracing::error!("failed to launch browser: {}", e);
                                this.state = BrowserActionsState::Stopped;
                            }
                            Err(e) => {
                                tracing::error!("launch thread dropped sender: {}", e);
                                this.state = BrowserActionsState::Stopped;
                            }
                        }
                        cx.notify();
                    })
                    .ok();
                })
                .detach();
            }
            BrowserActionsState::Starting => {
                window.push_notification(t!("error.browser_is_starting"), cx);
            }
            BrowserActionsState::Running => {
                window.push_notification(t!("error.browser_already_running"), cx);
            }
        }
    }
    fn stop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.state {
            BrowserActionsState::Stopped => {
                window.push_notification(t!("error.browser_is_not_running"), cx);
            }
            BrowserActionsState::Starting => {
                self.browser = None;
                self.state = BrowserActionsState::Stopped;
                cx.notify();
            }
            BrowserActionsState::Running => {
                self.browser = None;
                self.state = BrowserActionsState::Stopped;
                cx.notify();
            }
        }
    }
    fn get_browser(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<&Arc<Browser>> {
        if self.state != BrowserActionsState::Running {
            window.push_notification(t!("error.browser_is_not_running"), cx);
            return None;
        }
        match &self.browser {
            Some(browser) => Some(browser),
            None => {
                // return state to stopped
                self.state = BrowserActionsState::Stopped;
                cx.notify();
                window.push_notification(t!("error.browser_is_not_running"), cx);
                return None;
            }
        }
    }
    fn get_first_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<&Arc<Tab>> {
        let browser = self.get_browser(window, cx)?;
        let tabs = browser.get_tabs().lock().unwrap();
        match tabs.first() {
            Some(tab) => Some(tab),
            None => {
                window.push_notification(t!("error.no_active_tab"), cx);
                return None;
            }
        }
    }
    fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state != BrowserActionsState::Running {
            window.push_notification(t!("error.browser_is_not_running"), cx);
            return;
        }
        match &self.browser {
            Some(browser) => {
                if !Self::is_browser_alive(browser) {
                    self.browser = None;
                    self.state = BrowserActionsState::Stopped;
                    cx.notify();
                    window.push_notification(t!("error.browser_already_closed"), cx);
                    return;
                }

                let tabs = browser.get_tabs().lock().unwrap();
                let Some(tab) = tabs.first() else {
                    window.push_notification(t!("error.no_active_tab"), cx);
                    return;
                };
                if let Err(e) = tab.bring_to_front() {
                    tracing::error!("{:?}", e);
                    window.push_notification(t!("error.failed_to_bring_to_front"), cx);
                }
            }
            None => {
                // return state to stopped
                self.state = BrowserActionsState::Stopped;
                cx.notify();
                window.push_notification(t!("error.browser_is_not_running"), cx);
                return;
            }
        }
    }
    fn capture(&mut self, size: ScreenshotSize, window: &mut Window, cx: &mut Context<Self>) {}
}

impl Render for BrowserActions {
    fn render(&mut self, _: &mut Window, element_cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("browser-command")
            .h_flex()
            .justify_end()
            .text_2xl()
            .gap_2()
            .when(self.state == BrowserActionsState::Stopped, |cx| {
                cx.child(
                    Button::new("start-browser")
                        .text_color(element_cx.theme().green)
                        .child(AppIcons::Play)
                        .tooltip(t!("label.start_browser"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.start(window, cx);
                        })),
                )
            })
            .when(self.state == BrowserActionsState::Starting, |cx| {
                cx.child(
                    Button::new("starting-browser")
                        .text_color(element_cx.theme().yellow)
                        .child(Spinner::new()),
                )
            })
            .when(self.state == BrowserActionsState::Running, |cx| {
                cx.child(
                    Button::new("stop-browser")
                        .text_color(element_cx.theme().red)
                        .child(AppIcons::PlayOff)
                        .tooltip(t!("label.stop_browser"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.stop(window, cx);
                        })),
                )
                .child(
                    Button::new("focus-browser")
                        .child(AppIcons::Eye)
                        .tooltip(t!("label.focus_browser"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.focus(window, cx);
                        })),
                )
                .child(
                    Button::new("take-desktop-screenshot")
                        .child(AppIcons::Monitor)
                        .tooltip(t!("label.take_desktop_screenshot"))
                        .on_click(element_cx.listener(|_this, _, _window, _cx| {})),
                )
                .child(
                    Button::new("take-mobile-screenshot")
                        .child(AppIcons::Smartphone)
                        .tooltip(t!("label.take_mobile_screenshot"))
                        .on_click(element_cx.listener(|_this, _, _window, _cx| {})),
                )
                .child(
                    Button::new("take-crop-screenshot")
                        .child(AppIcons::Crop)
                        .tooltip(t!("label.take_crop_screenshot"))
                        .on_click(element_cx.listener(|_this, _, _window, _cx| {})),
                )
            })
    }
}
