use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use futures::channel::oneshot;
use gpui_kit::component::button::Button;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use headless_chrome::protocol::cdp::Emulation::{self};
use headless_chrome::protocol::cdp::Page::{self};
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

use crate::config::AppConfig;
use crate::utils::app_icons::AppIcons;
use crate::files::files::save_screenshot;
use crate::utils::random::random_string;
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
}

impl ScreenshotSize {
    fn dimensions(&self) -> (f64, f64) {
        let config = AppConfig::load();
        match self {
            ScreenshotSize::Desktop => {
                let desktop = config.capture_setting.desktop_capture_sizing;
                (desktop.width, desktop.height)
            }
            ScreenshotSize::Mobile => {
                let mobile = config.capture_setting.mobile_capture_sizing;
                (mobile.width, mobile.height)
            }
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
    pub replace_path: Option<PathBuf>,
    taking_cropped: bool,
}
impl EventEmitter<BrowserActionsEvents> for BrowserActions {}

impl BrowserActions {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            state: BrowserActionsState::Stopped,
            browser: None,
            working_dir: working_dir,
            replace_path: None,
            taking_cropped: false,
        }
    }
    // fn is_browser_alive(browser: &Browser) -> bool {
    //     browser.get_version().is_ok()
    // }
    pub fn open_url(&mut self, url: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = &self.get_first_tab(window, cx) {
            tab.navigate_to(url).ok();
        }
    }
    fn launch_browser(chrome_path: PathBuf) -> anyhow::Result<Browser> {
        let opts = LaunchOptionsBuilder::default()
            .headless(false)
            .path(Some(chrome_path))
            .build()
            .map_err(anyhow::Error::from)?;
        Ok(Browser::new(opts)?)
    }

    fn attach_browser(port: u16) -> anyhow::Result<Browser> {
        let url = format!("http://127.0.0.1:{port}/json/version");
        let body: serde_json::Value = ureq::get(&url)
            .call()
            .with_context(|| format!("no Chrome listening on port {port}"))?
            .body_mut()
            .read_json()?;

        let ws_url = body["webSocketDebuggerUrl"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("webSocketDebuggerUrl missing in /json/version"))?
            .to_string();

        Ok(Browser::connect(ws_url)?)
    }
    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.state {
            BrowserActionsState::Stopped => {
                let chrome_path = AppConfig::load().chrome_config.path;
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
                // Launching / Attaching Chrome blocks until the DevTools port is ready,
                // so it has to run off the main thread.
                let (tx, rx) = oneshot::channel::<anyhow::Result<Browser>>();
                std::thread::spawn(move || {
                    let app_config = AppConfig::load();
                    let attach = app_config.chrome_config.use_attach;
                    let port = app_config.chrome_config.attach_port;

                    let result = if attach {
                        Self::attach_browser(port)
                    } else {
                        Self::launch_browser(chrome_path)
                    };

                    let _ = tx.send(result);
                });
                cx.spawn_in(window, async move |this: WeakEntity<BrowserActions>, cx| {
                    let result = rx.await;

                    this.update_in(cx, |this, window, cx| {
                        match result {
                            Ok(Ok(browser)) => {
                                this.browser = Some(Arc::new(browser));
                                this.state = BrowserActionsState::Running;
                            }
                            Ok(Err(e)) => {
                                tracing::error!("failed to launch browser: {}", e);
                                this.state = BrowserActionsState::Stopped;
                                window.push_notification(t!("error.failed_to_start_browser"), cx);
                            }
                            Err(e) => {
                                tracing::error!("launch thread dropped sender: {}", e);
                                this.state = BrowserActionsState::Stopped;
                                window.push_notification(t!("error.failed_to_start_browser"), cx);
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
    fn restart(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stop(window, cx);
        self.start(window, cx);
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
                Some(browser)
                // if Self::is_browser_alive(browser) {
                //     Some(browser)
                // } else {
                //     // return state to stopped
                //     self.state = BrowserActionsState::Stopped;
                //     cx.notify();
                //     window.push_notification(t!("error.browser_is_not_running"), cx);
                //     None
                // }
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
    fn capture_screenshot(tab: &Tab, clip: Option<Page::Viewport>) -> Result<Vec<u8>> {
        let app_config = AppConfig::load();
        tab.capture_screenshot(
            app_config.capture_setting.image_format.to_cdp_format(),
            Some(app_config.capture_setting.quality as u32),
            clip,
            app_config.capture_setting.capture_from_surface,
        )
    }
    pub fn queue_replace_path(&mut self, path: PathBuf) {
        self.replace_path = Some(path);
    }
    pub fn cancel_replace(&mut self) {
        self.replace_path = None;
    }

    fn start_capture_cropped(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let replace_path = self.replace_path.clone();
        let working_dir = self.working_dir.clone();
        let entity = cx.entity().downgrade(); // grab it here, while we still have Context<Self>
        self.get_active_tab(window, cx, move |tab, window, cx| {
            let replace_path = replace_path.clone();
            let working_dir = working_dir.clone();
            let entity = entity.clone();
            // generate a random key for this capture session
            let key = random_string(32);
            let full_evaluation = format!("window.__dockeep_selection_{}", key);
            // bring tab into focus
            let _ = tab.bring_to_front();
            // inject the selector overlay
            let inject = tab.evaluate(
                &include_str!("../../../assets/selector.js")
                    .replace("0replace_key0", key.clone().as_str()),
                false,
            );
            if let Err(e) = inject {
                tracing::error!("Failed to inject selection overlay: {}", e);
                window.push_notification(t!("error.failed_to_inject_selector"), cx);
                return;
            }
            cx.spawn(async move |cx| {
                let _ = entity.update(cx, |this, cx| {
                    this.taking_cropped = true;
                    cx.notify();
                });
                let m_tab = tab.clone();
                let m_full_evaluation = full_evaluation.clone();
                // listen for selection update
                let selection = cx
                    .background_spawn(async move {
                        let started_at = Instant::now();
                        let crop_timeout = AppConfig::load().capture_setting.crop_timeout;
                        tracing::info!("waiting for selector {}", m_full_evaluation.clone());

                        let rr = loop {
                            if Instant::now() - started_at > Duration::from_secs(crop_timeout) {
                                tracing::error!("Timed out waiting for selector");
                                break Err(anyhow::anyhow!("Timed out waiting for selector"));
                            }
                            match m_tab.evaluate(
                                &format!("JSON.stringify({})", m_full_evaluation.clone()),
                                false,
                            ) {
                                Ok(result) => {
                                    // parse selection
                                    let Some(json_str) =
                                        result.value.and_then(|v| v.as_str().map(String::from))
                                    else {
                                        // continue if result is undefined/null
                                        tracing::debug!("{} is undefined/null", m_full_evaluation);
                                        thread::sleep(Duration::from_millis(100));
                                        continue;
                                    };

                                    let Ok(selection) =
                                        serde_json::from_str::<serde_json::Value>(&json_str)
                                    else {
                                        break Err(anyhow::anyhow!(
                                            "failed to parse selection json"
                                        ));
                                    };
                                    if selection["x"].as_f64().is_none() {
                                        // continue if selection is not available
                                        tracing::error!("{} is undefined/null", m_full_evaluation);
                                        thread::sleep(Duration::from_millis(100));
                                        continue;
                                    }
                                    tracing::info!("Found selector, closing loop, {:?}", selection);
                                    break Ok(selection);
                                }
                                Err(err) => {
                                    tracing::error!(
                                        "failed to evaluate {}: Failed to get {:?}",
                                        m_full_evaluation,
                                        err
                                    );
                                    break Err(anyhow::anyhow!(
                                        "failed to evaluate {}",
                                        m_full_evaluation
                                    ));
                                }
                            }
                        };
                        rr
                    })
                    .await;
                let Ok(selection) = selection else {
                    // selection is not available, stop taking cropped
                    tracing::error!("failed to get value for {}", full_evaluation.clone());
                    return;
                };

                let clip = Page::Viewport {
                    x: selection["x"].as_f64().unwrap(),
                    y: selection["y"].as_f64().unwrap(),
                    width: selection["width"].as_f64().unwrap(),
                    height: selection["height"].as_f64().unwrap(),
                    scale: 1.0,
                };

                let capture_data = Self::capture_screenshot(&tab, Some(clip));
                let capture_url = tab.get_url();
                let saved = save_screenshot(
                    capture_data,
                    replace_path,
                    working_dir.clone(),
                    &capture_url,
                );

                let update = entity.update(cx, |_, cx| {
                    if saved.saved {
                        if saved.is_replaced {
                            cx.emit(BrowserActionsEvents::Replace(saved.save_path.clone()));
                        } else {
                            cx.emit(BrowserActionsEvents::Add(saved.save_path.clone()));
                        }
                    }
                    cx.notify();
                });
                if let Err(err) = update {
                    // for logging only
                    tracing::error!("failed to update browser actions: {:?}", err);
                };
            })
            .detach();
        });
    }

    fn capture(&mut self, size: ScreenshotSize, window: &mut Window, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade(); // grab it here, while we still have Context<Self>
        let replace_path = self.replace_path.clone();
        let working_dir = self.working_dir.clone();

        self.get_active_tab(window, cx, move |tab, _window, cx| {
            let working_dir = working_dir.clone();
            let replace_path = replace_path.clone();

            let (width, height) = size.dimensions();
            tracing::info!("Capturing screenshot, size: {}x{}", width, height);

            // set device metrics override
            if let Err(e) = tab.call_method(Emulation::SetDeviceMetricsOverride {
                width: width as u32,
                height: height as u32,
                device_scale_factor: 1.0,
                mobile: false,
                scale: None,
                screen_width: None,
                screen_height: None,
                position_x: None,
                position_y: None,
                dont_set_visible_size: None,
                screen_orientation: None,
                viewport: None,
                display_feature: None,
                device_posture: None,
            }) {
                tracing::error!("Failed to set device metrics override: {:?}", e);
                // window.push_notification(t!("error.failed_to_set_device_metrics_override"), cx);
                return;
            }

            let entity = entity.clone(); // move a clone into the async block below

            cx.spawn(async move |cx| {
                let capture_url = tab.get_url();

                let result = cx
                    .background_spawn(async move {
                        let capture_data = Self::capture_screenshot(&tab, None);

                        // reset device metrics override
                        let _ = tab.call_method(Emulation::ClearDeviceMetricsOverride(None));
                        capture_data
                    })
                    .await;

                let saved =
                    save_screenshot(result, replace_path, working_dir.clone(), &capture_url);
                if saved.saved {
                    if let Err(e) = entity.update(cx, |_, cx| {
                        if saved.is_replaced {
                            cx.emit(BrowserActionsEvents::Replace(saved.save_path.clone()));
                        } else {
                            cx.emit(BrowserActionsEvents::Add(saved.save_path.clone()));
                        }
                        cx.notify();
                    }) {
                        tracing::error!("failed to emit event: {:?}", e);
                    }
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
            .when_some(self.replace_path.clone(), |cx, value| {
                cx.child(
                    Button::new("replace-path")
                        .text_color(element_cx.theme().yellow)
                        .child(AppIcons::SquareExclamationPoint)
                        .tooltip(t!("label.replacing_path", path = value.to_string_lossy())),
                )
            })
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
                        .child(AppIcons::CircleDot)
                        .tooltip(t!("label.stop_browser"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.stop(window, cx);
                        })),
                )
                .child(
                    Button::new("restart-browser")
                        .child(AppIcons::RefreshCw)
                        .tooltip(t!("label.restart_browser"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.restart(window, cx);
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
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.capture(ScreenshotSize::Desktop, window, cx);
                        })),
                )
                .child(
                    Button::new("take-mobile-screenshot")
                        .child(AppIcons::Smartphone)
                        .tooltip(t!("label.take_mobile_screenshot"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.capture(ScreenshotSize::Mobile, window, cx);
                        })),
                )
                .child(
                    Button::new("take-end-crop-screenshot")
                        .child(AppIcons::Crop)
                        .tooltip(t!("label.take_crop_screenshot"))
                        .on_click(element_cx.listener(|this, _, window, cx| {
                            this.start_capture_cropped(window, cx);
                        })),
                )
            })
    }
}
