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
            UIMessage::IBConnectionStatus { paper_connected, live_connected, active_account } => {
                inf!("IB connection status update - Paper: {}, Live: {}, Active: {:?}", 
                    paper_connected, live_connected, active_account);
                // TODO: Update UI with IB connection status
            }
            UIMessage::IBOrderTemplateUpdate { templates } => {
                inf!("Order templates updated: {} templates", templates.len());
                // TODO: Update UI with order templates
            }
            UIMessage::IBMarketData { symbol, bid, ask, last, volume } => {
                inf!("Market data for {}: bid={}, ask={}, last={}, volume={}", 
                    symbol, bid, ask, last, volume);
                // TODO: Update UI with market data
            }
            UIMessage::ChartImageUpdate { image_data, width, height, symbol } => {
                inf!("Chart image update for {} ({}x{})", symbol, width, height);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        // Create Slint image from RGB data
                        let pixel_buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                            &image_data,
                            width,
                            height,
                        );
                        let image = slint::Image::from_rgb8(pixel_buffer);
                        
                        // Update UI
                        ui.set_chart_image(image);
                        ui.set_chart_symbol(symbol.into());
                    } else {
                        err!("Failed to get Window pointer");
                    }
                });
            }
        }
    }
}