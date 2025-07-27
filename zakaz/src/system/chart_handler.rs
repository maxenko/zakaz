use tokio::sync::oneshot;
use std::sync::Arc;
use crate::{
    inf, err,
    charts::{
        CandlestickChart, ViewportController, ChartTheme,
    },
    system::{
        state::State,
        types::{RuntimeOutMessage, UIMessage, ChartMessage},
    },
};

pub async fn handle_chart_message(
    msg: ChartMessage,
    state: State,
    reply_channel: Option<oneshot::Sender<RuntimeOutMessage<State>>>,
) -> State {
    let mut state_local = state.clone();
    
    match msg {
        ChartMessage::UpdateChart { symbol, theme } => {
            inf!("Updating chart for {}", symbol);
            
            // Get IB client
            if let Some(ib_client) = &state_local.ib_client {
                // Fetch historical data
                match ib_client.lock().await.get_historical_data(&symbol, 100, "1 day").await {
                    Ok(historical_data) => {
                        // Store data in state
                        state_local.chart_data = Some((symbol.clone(), historical_data.bars.clone()));
                        
                        // Update or create viewport controller
                        if state_local.viewport_controller.is_none() {
                            state_local.viewport_controller = Some(Arc::new(tokio::sync::Mutex::new(
                                ViewportController::new(historical_data.bars.len())
                            )));
                        } else {
                            state_local.viewport_controller.as_ref().unwrap()
                                .lock().await
                                .update_data_length(historical_data.bars.len());
                        }
                        
                        // Generate chart
                        if let Err(e) = generate_and_send_chart(&state_local, theme).await {
                            err!("Failed to generate chart: {}", e);
                            state.send_message_to_ui(UIMessage::ErrorMessage(
                                format!("Failed to generate chart: {}", e)
                            ));
                        }
                    }
                    Err(e) => {
                        err!("Failed to fetch data for chart: {}", e);
                        state.send_message_to_ui(UIMessage::ErrorMessage(
                            format!("Failed to fetch chart data: {}", e)
                        ));
                    }
                }
            } else {
                state.send_message_to_ui(UIMessage::ErrorMessage(
                    "IB client not connected".to_string()
                ));
            }
        }
        
        ChartMessage::Pan { dx, dy } => {
            if let Some(controller) = &state_local.viewport_controller {
                controller.lock().await.pan(dx, dy);
                if let Err(e) = generate_and_send_chart(&state_local, None).await {
                    err!("Failed to update chart after pan: {}", e);
                }
            }
        }
        
        ChartMessage::Zoom { factor, center_x, center_y } => {
            if let Some(controller) = &state_local.viewport_controller {
                controller.lock().await.zoom(factor, center_x, center_y);
                if let Err(e) = generate_and_send_chart(&state_local, None).await {
                    err!("Failed to update chart after zoom: {}", e);
                }
            }
        }
        
        ChartMessage::ResetZoom => {
            if let Some(controller) = &state_local.viewport_controller {
                controller.lock().await.reset_zoom();
                if let Err(e) = generate_and_send_chart(&state_local, None).await {
                    err!("Failed to update chart after reset: {}", e);
                }
            }
        }
        
        ChartMessage::SetViewport(viewport) => {
            if let Some(controller) = &state_local.viewport_controller {
                controller.lock().await.set_viewport(viewport);
                if let Err(e) = generate_and_send_chart(&state_local, None).await {
                    err!("Failed to update chart after viewport change: {}", e);
                }
            }
        }
    }
    
    // Send acknowledgment if needed
    if let Some(channel) = reply_channel {
        let _ = channel.send(RuntimeOutMessage::Ok);
    }
    
    state_local
}

async fn generate_and_send_chart(
    state: &State,
    theme: Option<ChartTheme>,
) -> Result<(), crate::error::AppError> {
    if let Some((symbol, bars)) = &state.chart_data {
        if let Some(controller) = &state.viewport_controller {
            let viewport = controller.lock().await.get_viewport();
            
            // Use provided theme or default
            let chart_theme = theme.unwrap_or_else(|| {
                state.chart_theme.as_ref()
                    .cloned()
                    .unwrap_or_default()
            });
            
            // Use fixed size for now, but this could be made dynamic
            let width = 800;
            let height = 600;
            
            // Create chart
            let chart = CandlestickChart::new(width, height, chart_theme);
            
            // Render to buffer (using bitmap for performance)
            let buffer = chart.render_to_buffer(bars, &viewport)?;
            
            // Send to UI
            state.send_message_to_ui(UIMessage::ChartImageUpdate {
                image_data: buffer,
                width,
                height,
                symbol: symbol.clone(),
            });
        }
    }
    
    Ok(())
}