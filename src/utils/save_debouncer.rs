use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui_kit::{App, Task};

use crate::config::AppConfig;

const DEBOUNCE_MS: u64 = 500;

/// Wraps a shared `AppConfig` and debounces saves: calling [`schedule`] cancels
/// any pending save and schedules a fresh one `DEBOUNCE_MS` milliseconds later.
/// Dropping the pending [`Task`] cancels it, so rapid changes only produce one
/// disk write.
#[derive(Clone)]
pub struct SaveDebouncer {
    pub config: Rc<RefCell<AppConfig>>,
    pending: Rc<RefCell<Option<Task<()>>>>,
}

impl SaveDebouncer {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Rc::new(RefCell::new(config)),
            pending: Rc::new(RefCell::new(None)),
        }
    }

    /// Cancel any in-flight save and schedule a new one after `DEBOUNCE_MS` ms.
    pub fn schedule(&self, cx: &mut App) {
        let config = self.config.clone();
        let task = cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(DEBOUNCE_MS))
                .await;
            config.borrow().save();
        });
        // Dropping the old Task cancels it.
        *self.pending.borrow_mut() = Some(task);
    }
}
