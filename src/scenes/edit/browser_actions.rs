use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::oneshot;
use gpui_kit::component::button::Button;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

use crate::utils::app_config::AppConfig;
use crate::utils::app_icons::AppIcons;
use gpui_kit::component::WindowExt;
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserActionsState {
    Stopped,
    Starting,
    Running,
}

enum ScreenshotSize {
    Desktop,
    Mobile,
    Crop,
}

impl ScreenshotSize {
    fn dimensions(&self) -> (u32, u32) {
        match self {
            ScreenshotSize::Desktop => (1920, 1080),
            ScreenshotSize::Mobile => (390, 844),
            ScreenshotSize::Crop => (1280, 720),
        }
    }
}
pub enum BrowserActionsEvents {
    Add(PathBuf),
    Replace(PathBuf),
}

pub struct BrowserActions {
    state: BrowserActionsState,
    browser: Option<Arc<Browser>>,
    working_dir: PathBuf,
    replace_path: Option<PathBuf>,
}
impl EventEmitter<BrowserActionsEvents> for BrowserActions {}

impl BrowserActions {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            state: BrowserActionsState::Stopped,
            browser: None,
            working_dir: working_dir,
            replace_path: None,
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
            Some(browser) => {
                if Self::is_browser_alive(browser) {
                    Some(browser)
                } else {
                    // return state to stopped
                    self.state = BrowserActionsState::Stopped;
                    cx.notify();
                    window.push_notification(t!("error.browser_is_not_running"), cx);
                    None
                }
            }
            None => {
                // return state to stopped
                self.state = BrowserActionsState::Stopped;
                cx.notify();
                window.push_notification(t!("error.browser_is_not_running"), cx);
                return None;
            }
        }
    }
    fn get_first_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<Arc<Tab>> {
        let browser = self.get_browser(window, cx)?;
        let tabs = browser.get_tabs().lock().unwrap();
        match tabs.first() {
            Some(tab) => Some(tab.clone()),
            None => {
                window.push_notification(t!("error.no_active_tab"), cx);
                None
            }
        }
    }

    fn get_active_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        on_select: impl Fn(Arc<Tab>, &mut Window, &mut App) + 'static,
    ) {
        let Some(browser) = self.get_browser(window, cx) else {
            return;
        };
        let browser = browser.clone();
        let tabs: Vec<Arc<Tab>> = browser.get_tabs().lock().unwrap().clone();

        match tabs.len() {
            0 => {
                window.push_notification(t!("error.no_active_tab"), cx);
            }
            1 => {
                on_select(tabs.into_iter().next().unwrap(), window, cx);
            }
            _ => {
                tracing::info!("Multiple tabs, showing dialog");
                let on_select = Arc::new(on_select);
                window.open_dialog(cx, move |dialog, _, _| {
                    let tab_items = tabs.iter().enumerate().map(|(i, tab)| {
                        let title = tab.get_title().unwrap_or_else(|_| format!("Tab {}", i + 1));
                        let tab = tab.clone();
                        let on_select = on_select.clone();
                        Button::new(format!("select-tab-{}", i))
                            .label(title)
                            .on_click(move |_, window, cx| {
                                on_select(tab.clone(), window, cx);
                                window.close_dialog(cx);
                            })
                    });
                    dialog
                        .title(t!("dialog.select_tab"))
                        .child(div().v_flex().gap_2().p_2().children(tab_items))
                        .footer(
                            div().h_flex().justify_end().child(
                                Button::new("cancel-tab-select")
                                    .outline()
                                    .label(t!("label.cancel"))
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                    }),
                            ),
                        )
                });
            }
        }
    }
    fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = self.get_first_tab(window, cx) {
            if let Err(e) = tab.bring_to_front() {
                tracing::error!("Failed to bring tab to front: {:?}", e);
                window.push_notification(t!("error.failed_to_bring_to_front"), cx);
            };
        }
    }
    pub fn queue_replace_path(&mut self, path: PathBuf) {
        self.replace_path = Some(path);
    }

    fn capture(&mut self, _size: ScreenshotSize, window: &mut Window, cx: &mut Context<Self>) {
        let working_dir = self.working_dir.clone();
        let replace_path = self.replace_path.clone();
        let entity = cx.entity().downgrade(); // grab it here, while we still have Context<Self>

        self.get_active_tab(window, cx, move |tab, _window, cx| {
            let working_dir = working_dir.clone();
            let replace_path = replace_path.clone();
            let entity = entity.clone(); // move a clone into the async block below

            cx.spawn(async move |cx| {
                let result = cx
                    .background_spawn(async move {
                        tab.capture_screenshot(
                            CaptureScreenshotFormatOption::Png,
                            Some(100),
                            None,
                            false,
                        )
                    })
                    .await;

                match result {
                    Ok(data) => {
                        let (path, is_replace) = if let Some(rp) = replace_path {
                            (rp, true)
                        } else {
                            let filename = chrono::Local::now()
                                .format("Screenshot_%Y%m%d_%H%M%S.png")
                                .to_string();
                            (working_dir.join(&filename), false)
                        };

                        if let Err(e) = std::fs::write(&path, &data) {
                            tracing::error!("Failed to save screenshot to {:?}: {}", path, e);
                            return;
                        }

                        // Fresh async continuation — no longer nested inside the
                        // click-handler's borrow of this same entity, so this is safe.
                        if let Err(e) = entity.update(cx, |this, cx| {
                            if is_replace {
                                this.replace_path = None;
                                cx.emit(BrowserActionsEvents::Replace(path.clone()));
                            } else {
                                cx.emit(BrowserActionsEvents::Add(path.clone()));
                            }
                            cx.notify();
                        }) {
                            tracing::error!("failed to emit event: {:?}", e);
                        }
                    }
                    Err(e) => tracing::error!("Failed to capture screenshot: {}", e),
                }
            })
            .detach();
        });
    }
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
            .when_some(self.replace_path.clone(), |cx, value| {
                cx.child(
                    Button::new("replace-path")
                        .text_color(element_cx.theme().yellow)
                        .child(AppIcons::SquareExclamationPoint)
                        .tooltip(t!("label.replacing_path", path = value.to_string_lossy()))
                        .on_click(element_cx.listener(|this, _, _, cx| {
                            this.replace_path = None;
                            cx.notify();
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
                        .child(AppIcons::CircleDot)
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
                        .on_click(element_cx.listener(|this, _, _window, _cx| {
                            this.capture(ScreenshotSize::Desktop, _window, _cx);
                        })),
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
