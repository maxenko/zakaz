use crate::{inf, err};
use crate::system::types::UIMessage;
use crate::MainWindow;
use slint::{Weak, SharedString};

pub fn get_ui_message_handler(weak_handle: Weak<MainWindow>) -> impl Fn(UIMessage) -> () {
    let ui_handle = weak_handle.clone();
    inf!("UI message handler created");

    move |msg| {
        let ui_handle = ui_handle.clone();

        match msg {
            UIMessage::UpdateCounter(count) => {
                inf!("Updating counter to: {}", count);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_counter(count);
                    } else {
                        err!("Failed to get Window pointer");
                    }
                });
            }
            UIMessage::StatusMessage(message) => {
                inf!("Status: {}", message);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_status_message(SharedString::from(message));
                    } else {
                        err!("Failed to get Window pointer");
                    }
                });
            }
            UIMessage::ErrorMessage(message) => {
                err!("Error: {}", message);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_error_message(SharedString::from(message));
                        // Clear error after 5 seconds
                        let ui_handle = ui_handle.clone();
                        slint::Timer::single_shot(std::time::Duration::from_secs(5), move || {
                            if let Some(ui) = ui_handle.upgrade() {
                                ui.set_error_message(SharedString::from(""));
                            }
                        });
                    } else {
                        err!("Failed to get Window pointer");
                    }
                });
            }
            UIMessage::RuntimeStarted => {
                inf!("Runtime started");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_runtime_running(true);
                    } else {
                        err!("Failed to get Window pointer");
                    }
                });
            }
            UIMessage::RuntimeStopped => {
                inf!("Runtime stopped");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_runtime_running(false);
                    } else {
                        err!("Failed to get Window pointer");
                    }
                });
            }
        }
    }
}