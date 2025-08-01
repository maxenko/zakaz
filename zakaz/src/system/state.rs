use std::sync::Arc;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::system::{runtime::Runtime, types::UIMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Version number for state tracking
    pub version: u64,
    /// Counter value for demo
    pub counter: i32,
    /// Application start time
    pub start_time: DateTime<Local>,
    /// Is the runtime running
    pub is_running: bool,
    /// Runtime reference (not serialized)
    #[serde(skip)]
    pub runtime: Option<Arc<Runtime>>,
    /// IB client instance (not serialized)
    #[serde(skip)]
    pub ib_client: Option<Arc<tokio::sync::Mutex<crate::ib::IBClient>>>,
    /// Chart data (not serialized)
    #[serde(skip)]
    pub chart_data: Option<(String, Vec<crate::ib::types::HistoricalBar>)>,
    /// Chart viewport controller (not serialized)
    #[serde(skip)]
    pub viewport_controller: Option<Arc<tokio::sync::Mutex<crate::charts::ViewportController>>>,
    /// Chart theme (not serialized)
    #[serde(skip)]
    pub chart_theme: Option<crate::charts::ChartTheme>,
}

impl State {
    pub fn new() -> Self {
        Self {
            version: 0,
            counter: 0,
            start_time: Local::now(),
            is_running: false,
            runtime: None,
            ib_client: None,
            chart_data: None,
            viewport_controller: None,
            chart_theme: None,
        }
    }

    pub fn load_or_default() -> (Self, bool) {
        // For now, just return default state
        // In the future, this could load from disk
        (Self::new(), false)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), std::io::Error> {
        // For now, do nothing
        // In the future, this could save to disk
        Ok(())
    }

    pub fn send_message_to_ui(&self, msg: UIMessage) {
        if let Some(runtime) = &self.runtime {
            let runtime = runtime.clone();
            let msg = msg.clone();
            tokio::spawn(async move {
                if let Ok(ui_events) = runtime.ui_events.try_lock() {
                    ui_events.notify(msg).await;
                }
            });
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}