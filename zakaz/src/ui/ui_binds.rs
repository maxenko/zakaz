use std::sync::Arc;

use crate::{
    MainWindow,
    system::{
        runtime::Runtime,
        types::{ChartMessage, RuntimeInMessage},
    },
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
    
    // Bind chart pan
    let rt = runtime.clone();
    ui.on_chart_pan(move |dx, dy| {
        // Convert from Slint length to f64 pixels
        let dx_pixels = dx as f64;
        let dy_pixels = dy as f64;
        rt.tell(RuntimeInMessage::Chart(ChartMessage::Pan { 
            dx: dx_pixels, 
            dy: dy_pixels 
        }));
    });
    
    // Bind chart zoom
    let rt = runtime.clone();
    ui.on_chart_zoom(move |factor, x, y| {
        let center_x = x as f64;
        let center_y = y as f64;
        rt.tell(RuntimeInMessage::Chart(ChartMessage::Zoom { 
            factor: factor as f64, 
            center_x, 
            center_y 
        }));
    });
    
    // Bind chart reset zoom
    let rt = runtime.clone();
    ui.on_chart_reset_zoom(move || {
        rt.tell(RuntimeInMessage::Chart(ChartMessage::ResetZoom));
    });
    
    // Bind load test chart button
    let rt = runtime.clone();
    ui.on_load_test_chart(move || {
        // Connect to paper account and load chart
        // We need to properly handle the async response, so we'll do this in sequence
        let rt_inner = rt.clone();
        tokio::spawn(async move {
            // First connect to paper account
            let (tx, rx) = tokio::sync::oneshot::channel();
            rt_inner.tell(RuntimeInMessage::IB(crate::ib::messages::IBMessage::ConnectPaper {
                response: tx,
            }));
            
            // Wait for connection result
            match rx.await {
                Ok(Ok(())) => {
                    // Connection successful, now load chart data
                    rt_inner.tell(RuntimeInMessage::Chart(ChartMessage::UpdateChart {
                        symbol: "AAPL".to_string(),
                        theme: None,
                    }));
                }
                Ok(Err(e)) => {
                    rt_inner.tell(RuntimeInMessage::Error(format!("Failed to connect: {}", e)));
                }
                Err(_) => {
                    rt_inner.tell(RuntimeInMessage::Error("Connection response channel error".to_string()));
                }
            }
        });
    });
}