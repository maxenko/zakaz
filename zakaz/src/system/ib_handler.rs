use std::sync::Arc;

use tokio::sync::oneshot;

use crate::{
    err, inf, wrn,
    ib::{messages::*, IBClient},
    system::{
        state::State,
        types::{RuntimeOutMessage, UIMessage},
    },
};

// Helper macro for oneshot channels
macro_rules! notify_oneshot {
    ($reply_channel:expr, $message:expr) => {
        if let Some(sender) = $reply_channel {
            let _ = sender.send($message);
        }
    };
}

pub async fn handle_ib_message(
    msg: IBMessage,
    state: State,
    reply_channel: Option<oneshot::Sender<RuntimeOutMessage<State>>>,
) -> State {
    let mut state_local = state.clone();
    
    // Ensure IB client is initialized
    let ib_client = if let Some(client) = &state_local.ib_client {
        client.clone()
    } else {
        let client = Arc::new(tokio::sync::Mutex::new(IBClient::new()));
        state_local.ib_client = Some(client.clone());
        client
    };
    
    match msg {
        IBMessage::ConnectPaper { response } => {
            inf!("Connecting to IB paper account...");
            match ib_client.lock().await.connect_paper().await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage("Connected to paper account".to_string()));
                    update_connection_status(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to connect to paper account: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(format!("Paper connection failed: {}", e)));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::ConnectLive { response } => {
            wrn!("Connecting to IB LIVE account...");
            match ib_client.lock().await.connect_live().await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage("Connected to LIVE account".to_string()));
                    update_connection_status(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to connect to live account: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(format!("Live connection failed: {}", e)));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::Disconnect => {
            inf!("Disconnecting from IB...");
            ib_client.lock().await.disconnect().await;
            state.send_message_to_ui(UIMessage::StatusMessage("Disconnected from IB".to_string()));
            update_connection_status(&state, &ib_client).await;
            notify_oneshot!(reply_channel, RuntimeOutMessage::Ok);
        }
        
        IBMessage::SwitchToPaper { response } => {
            inf!("Switching to paper account...");
            match ib_client.lock().await.switch_to_paper().await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage("Switched to paper account".to_string()));
                    update_connection_status(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to switch to paper: {}", e);
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::SwitchToLive { response } => {
            wrn!("Switching to LIVE account...");
            match ib_client.lock().await.switch_to_live().await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage("Switched to LIVE account".to_string()));
                    update_connection_status(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to switch to live: {}", e);
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::GetConnectionStatus { response } => {
            let status = ib_client.lock().await.get_connection_status().await;
            let _ = response.send(status);
        }
        
        IBMessage::CreateTemplate { name, symbol, side, quantity, limit_price, stop_price, time_in_force, model, response } => {
            inf!("Creating order template: {}", name);
            let template = crate::ib::OrderTemplate::new(
                name.clone(),
                symbol,
                side,
                quantity,
                limit_price,
                stop_price,
                time_in_force,
                model,
            );
            
            match ib_client.lock().await.create_template(template).await {
                Ok(template_id) => {
                    state.send_message_to_ui(UIMessage::StatusMessage(format!("Created template: {}", name)));
                    update_templates(&state, &ib_client).await;
                    let _ = response.send(Ok(template_id));
                }
                Err(e) => {
                    err!("Failed to create template: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(format!("Failed to create template: {}", e)));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::UpdateTemplate { template, response } => {
            inf!("Updating template: {}", template.id);
            match ib_client.lock().await.update_template(template).await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage("Template updated".to_string()));
                    update_templates(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to update template: {}", e);
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::DeleteTemplate { template_id, response } => {
            inf!("Deleting template: {}", template_id);
            match ib_client.lock().await.delete_template(&template_id).await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage("Template deleted".to_string()));
                    update_templates(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to delete template: {}", e);
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::GetTemplate { template_id, response } => {
            let template = ib_client.lock().await.get_template(&template_id).await;
            let _ = response.send(template);
        }
        
        IBMessage::GetAllTemplates { response } => {
            let templates = ib_client.lock().await.get_all_templates().await;
            let _ = response.send(templates);
        }
        
        IBMessage::ActivateTemplate { template_id, response } => {
            inf!("Activating template: {}", template_id);
            match ib_client.lock().await.activate_template(&template_id).await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage(format!("Template {} activated", template_id)));
                    update_templates(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to activate template: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(format!("Failed to activate: {}", e)));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::DeactivateTemplate { template_id, response } => {
            inf!("Deactivating template: {}", template_id);
            match ib_client.lock().await.deactivate_template(&template_id).await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage(format!("Template {} deactivated", template_id)));
                    update_templates(&state, &ib_client).await;
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to deactivate template: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(format!("Failed to deactivate: {}", e)));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::SubscribeMarketData { symbol, response } => {
            inf!("Subscribing to market data for {}", symbol);
            match ib_client.lock().await.subscribe_market_data(&symbol).await {
                Ok(_) => {
                    state.send_message_to_ui(UIMessage::StatusMessage(format!("Subscribed to {}", symbol)));
                    let _ = response.send(Ok(()));
                }
                Err(e) => {
                    err!("Failed to subscribe to market data: {}", e);
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::UnsubscribeMarketData { symbol } => {
            inf!("Unsubscribing from market data for {}", symbol);
            ib_client.lock().await.unsubscribe_market_data(&symbol).await;
            notify_oneshot!(reply_channel, RuntimeOutMessage::Ok);
        }
        
        IBMessage::GetAccountSummary { response } => {
            // TODO: Implement account summary retrieval
            let _ = response.send(Err("Account summary not yet implemented".to_string()));
        }
        
        IBMessage::GetPositions { response } => {
            // TODO: Implement positions retrieval
            let _ = response.send(Err("Positions retrieval not yet implemented".to_string()));
        }
        
        IBMessage::GetHistoricalData { symbol, duration_days, bar_size, response } => {
            inf!("Getting historical data for {} - {} days of {} bars", symbol, duration_days, bar_size);
            match ib_client.lock().await.get_historical_data(&symbol, duration_days, &bar_size).await {
                Ok(historical_data) => {
                    state.send_message_to_ui(UIMessage::StatusMessage(
                        format!("Retrieved {} bars for {}", historical_data.bars.len(), symbol)
                    ));
                    let _ = response.send(Ok(historical_data));
                }
                Err(e) => {
                    err!("Failed to get historical data: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(
                        format!("Failed to get historical data: {}", e)
                    ));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
        
        IBMessage::CalculateFilteredATR { symbol, period_days, method, response } => {
            inf!("Calculating filtered ATR for {} - {} days period", symbol, period_days);
            match ib_client.lock().await.calculate_filtered_atr(&symbol, period_days, method).await {
                Ok(atr_result) => {
                    let msg = format!(
                        "ATR for {}: Filtered {:.2}, Regular {:.2}, Excluded {} bars ({}%)",
                        symbol, atr_result.filtered_atr, atr_result.regular_atr, 
                        atr_result.excluded_bars, (atr_result.exclusion_rate * 100.0) as i32
                    );
                    state.send_message_to_ui(UIMessage::StatusMessage(msg));
                    let _ = response.send(Ok(atr_result));
                }
                Err(e) => {
                    err!("Failed to calculate ATR: {}", e);
                    state.send_message_to_ui(UIMessage::ErrorMessage(
                        format!("Failed to calculate ATR: {}", e)
                    ));
                    let _ = response.send(Err(e.to_string()));
                }
            }
        }
    }
    
    state_local
}

async fn update_connection_status(state: &State, ib_client: &Arc<tokio::sync::Mutex<IBClient>>) {
    let status = ib_client.lock().await.get_connection_status().await;
    state.send_message_to_ui(UIMessage::IBConnectionStatus {
        paper_connected: status.paper_connected,
        live_connected: status.live_connected,
        active_account: status.active_account,
    });
}

async fn update_templates(state: &State, ib_client: &Arc<tokio::sync::Mutex<IBClient>>) {
    let templates = ib_client.lock().await.get_all_templates().await;
    state.send_message_to_ui(UIMessage::IBOrderTemplateUpdate { templates });
}