mod system;
mod ui;
mod error;

use std::sync::Arc;
use slint::ComponentHandle;
use crate::system::runtime::Runtime;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    // Logging is initialized by the Log module on first use

    // Create UI
    let ui = MainWindow::new()?;
    let ui_handle = ui.as_weak();
    let ui_arc = Arc::new(ui);

    // Set up runtime
    let runtime = Runtime::new().await;
    let ui_message_handler = crate::ui::ui_message_handler::get_ui_message_handler(ui_handle.clone());
    runtime.ui_events.lock().await.subscribe_send_only(ui_message_handler).await;
    runtime.start();

    // Bind UI events to runtime
    ui::ui_binds::bind_ui_events(runtime.clone(), ui_arc.clone());

    // Run the UI
    ui_arc.run()
}
