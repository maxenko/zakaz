use std::sync::Arc;
use crate::{
    system::{types::RuntimeInMessage, runtime::Runtime},
    MainWindow,
};

/// Bind UI events to runtime messages
pub fn bind_ui_events(runtime: Arc<Runtime>, ui: Arc<MainWindow>) {
    // Bind increment button
    let rt = runtime.clone();
    ui.on_increment_clicked(move || {
        rt.tell(RuntimeInMessage::IncrementCounter);
    });

    // Bind decrement button
    let rt = runtime.clone();
    ui.on_decrement_clicked(move || {
        rt.tell(RuntimeInMessage::DecrementCounter);
    });

    // Bind reset button
    let rt = runtime.clone();
    ui.on_reset_clicked(move || {
        rt.tell(RuntimeInMessage::ResetCounter);
    });

    // Bind start button
    let rt = runtime.clone();
    ui.on_start_clicked(move || {
        rt.tell(RuntimeInMessage::Start);
    });

    // Bind stop button
    let rt = runtime.clone();
    ui.on_stop_clicked(move || {
        rt.tell(RuntimeInMessage::Stop);
    });
}