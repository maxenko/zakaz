use std::fmt;
use chrono::{DateTime, Local, Utc};

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
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
}

impl fmt::Display for UIMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UIMessage::UpdateCounter(count) => write!(f, "Update counter: {}", count),
            UIMessage::StatusMessage(msg) => write!(f, "Status: {}", msg),
            UIMessage::ErrorMessage(msg) => write!(f, "Error: {}", msg),
            UIMessage::RuntimeStarted => write!(f, "Runtime started"),
            UIMessage::RuntimeStopped => write!(f, "Runtime stopped"),
        }
    }
}