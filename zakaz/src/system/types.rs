use std::fmt;

use chrono::{DateTime, Local, Utc};

use crate::charts::ChartViewport;
use crate::ib::messages::IBMessage;

#[derive(Debug)]
#[allow(dead_code)]
pub enum RuntimeInMessage<S> {
    Start,
    Stop,
    /// Get a state copy
    State,
    /// Set a new state, overriding any previous state
    NewState(S),
    /// Increment counter (for demo)
    IncrementCounter,
    /// Decrement counter (for demo)
    DecrementCounter,
    /// Reset counter to zero
    ResetCounter,
    /// Send an error message to runtime
    Error(String),
    /// IB-related messages
    IB(IBMessage),
    /// Chart-related messages
    Chart(ChartMessage),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RuntimeOutMessage<S> {
    /// Used to signify that the runtime has started
    Started(DateTime<Local>),
    /// A copy of the current state, with the time of the request
    State(S, DateTime<Utc>),
    /// Error message
    Error(String),
    /// Used to signify a successful operation with an optional message
    OkMsg(String),
    /// Used to signify a successful operation
    Ok,
    /// The message was not handled
    Unhandled(RuntimeInMessage<S>),
}

#[derive(Debug, Clone)]
pub enum UIMessage {
    /// Update the counter display
    UpdateCounter(i32),
    /// Show status message
    StatusMessage(String),
    /// Show error message
    ErrorMessage(String),
    /// Runtime started
    RuntimeStarted,
    /// Runtime stopped
    RuntimeStopped,
    /// IB connection status changed
    IBConnectionStatus {
        paper_connected: bool,
        live_connected: bool,
        active_account: Option<crate::ib::AccountType>,
    },
    /// IB order template update
    IBOrderTemplateUpdate {
        templates: Vec<crate::ib::OrderTemplate>,
    },
    /// IB market data update
    IBMarketData {
        symbol: String,
        bid: f64,
        ask: f64,
        last: f64,
        volume: i64,
    },
    /// Chart image update
    ChartImageUpdate {
        image_data: Vec<u8>,
        width: u32,
        height: u32,
        symbol: String,
    },
}

impl fmt::Display for UIMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UIMessage::UpdateCounter(count) => write!(f, "Update counter: {}", count),
            UIMessage::StatusMessage(msg) => write!(f, "Status: {}", msg),
            UIMessage::ErrorMessage(msg) => write!(f, "Error: {}", msg),
            UIMessage::RuntimeStarted => write!(f, "Runtime started"),
            UIMessage::RuntimeStopped => write!(f, "Runtime stopped"),
            UIMessage::IBConnectionStatus { paper_connected, live_connected, active_account } => {
                write!(f, "IB Status - Paper: {}, Live: {}, Active: {:?}", 
                    paper_connected, live_connected, active_account)
            },
            UIMessage::IBOrderTemplateUpdate { templates } => {
                write!(f, "Order templates updated: {} templates", templates.len())
            },
            UIMessage::IBMarketData { symbol, last, .. } => {
                write!(f, "Market data for {}: ${:.2}", symbol, last)
            },
            UIMessage::ChartImageUpdate { symbol, width, height, .. } => {
                write!(f, "Chart updated for {} ({}x{})", symbol, width, height)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChartMessage {
    /// Update chart with new data
    UpdateChart {
        symbol: String,
        theme: Option<crate::charts::ChartTheme>,
    },
    /// Pan the chart
    Pan {
        dx: f64,
        dy: f64,
    },
    /// Zoom the chart
    Zoom {
        factor: f64,
        center_x: f64,
        center_y: f64,
    },
    /// Reset zoom
    ResetZoom,
    /// Set viewport directly
    SetViewport(ChartViewport),
}