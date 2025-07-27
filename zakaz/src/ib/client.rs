use std::collections::HashMap;
use std::sync::Arc;

use ibapi::{contracts::Contract, orders, Client};
use ibapi::prelude::{HistoricalBarSize, HistoricalWhatToShow};
use tokio::sync::{Mutex, RwLock};

use crate::error::AppError;
use crate::{err, inf, wrn};
use super::messages::{ConnectionStatus, MarketData};
use super::types::{ATRResult, ExcludedBar, HistoricalBar, HistoricalData, OrderTemplate, OrderTemplateStatus, OutlierMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Paper,
    Live,
}

pub struct IBClient {
    paper_client: Option<Arc<Mutex<Client>>>,
    live_client: Option<Arc<Mutex<Client>>>,
    active_account: Arc<RwLock<Option<AccountType>>>,
    order_templates: Arc<RwLock<HashMap<String, OrderTemplate>>>,
    active_orders: Arc<Mutex<HashMap<i32, String>>>, // order_id -> template_id
    market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    next_order_id: Arc<Mutex<i32>>,
}

impl std::fmt::Debug for IBClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IBClient")
            .field("paper_connected", &self.paper_client.is_some())
            .field("live_connected", &self.live_client.is_some())
            .field("active_account", &"<async>")
            .field("templates_count", &"<async>")
            .field("active_orders_count", &"<async>")
            .finish()
    }
}

impl IBClient {
    pub fn new() -> Self {
        Self {
            paper_client: None,
            live_client: None,
            active_account: Arc::new(RwLock::new(None)),
            order_templates: Arc::new(RwLock::new(HashMap::new())),
            active_orders: Arc::new(Mutex::new(HashMap::new())),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            next_order_id: Arc::new(Mutex::new(1000)),
        }
    }
    
    pub async fn connect_paper(&mut self) -> Result<(), AppError> {
        let paper_url = "127.0.0.1:7497"; // Default TWS paper trading port
        let client_id = 101;
        
        // Run sync connection in blocking task
        let result = tokio::task::spawn_blocking(move || {
            Client::connect(paper_url, client_id)
        }).await
        .map_err(|e| AppError::IBConnection(format!("Task join error: {}", e)))?;
        
        match result {
            Ok(client) => {
                self.paper_client = Some(Arc::new(Mutex::new(client)));
                // Automatically set as active account
                *self.active_account.write().await = Some(AccountType::Paper);
                inf!("Connected to paper trading account and set as active");
                Ok(())
            }
            Err(e) => {
                err!("Failed to connect to paper account: {}", e);
                Err(AppError::IBConnection(format!("Paper connection failed: {}", e)))
            }
        }
    }
    
    pub async fn connect_live(&mut self) -> Result<(), AppError> {
        let live_url = "127.0.0.1:7496"; // Default TWS live trading port
        let client_id = 102;
        
        // Run sync connection in blocking task
        let result = tokio::task::spawn_blocking(move || {
            Client::connect(live_url, client_id)
        }).await
        .map_err(|e| AppError::IBConnection(format!("Task join error: {}", e)))?;
        
        match result {
            Ok(client) => {
                self.live_client = Some(Arc::new(Mutex::new(client)));
                // Automatically set as active account
                *self.active_account.write().await = Some(AccountType::Live);
                wrn!("Connected to LIVE trading account and set as active");
                Ok(())
            }
            Err(e) => {
                err!("Failed to connect to live account: {}", e);
                Err(AppError::IBConnection(format!("Live connection failed: {}", e)))
            }
        }
    }
    
    pub async fn disconnect(&mut self) {
        self.paper_client = None;
        self.live_client = None;
        *self.active_account.write().await = None;
        inf!("Disconnected from IB");
    }
    
    pub async fn switch_to_paper(&self) -> Result<(), AppError> {
        if self.paper_client.is_some() {
            *self.active_account.write().await = Some(AccountType::Paper);
            inf!("Switched to paper trading account");
            Ok(())
        } else {
            Err(AppError::IBConnection("Paper account not connected".to_string()))
        }
    }
    
    pub async fn switch_to_live(&self) -> Result<(), AppError> {
        if self.live_client.is_some() {
            *self.active_account.write().await = Some(AccountType::Live);
            wrn!("Switched to LIVE trading account");
            Ok(())
        } else {
            Err(AppError::IBConnection("Live account not connected".to_string()))
        }
    }
    
    pub async fn get_connection_status(&self) -> ConnectionStatus {
        ConnectionStatus {
            paper_connected: self.paper_client.is_some(),
            live_connected: self.live_client.is_some(),
            active_account: *self.active_account.read().await,
        }
    }
    
    async fn get_active_client(&self) -> Result<Arc<Mutex<Client>>, AppError> {
        let account_type = self.active_account.read().await;
        match *account_type {
            Some(AccountType::Paper) => {
                self.paper_client.clone()
                    .ok_or(AppError::IBConnection("Paper client not connected".to_string()))
            }
            Some(AccountType::Live) => {
                self.live_client.clone()
                    .ok_or(AppError::IBConnection("Live client not connected".to_string()))
            }
            None => Err(AppError::IBConnection("No active account selected".to_string()))
        }
    }
    
    async fn get_next_order_id(&self) -> i32 {
        let mut id = self.next_order_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }
    
    // Order template management
    pub async fn create_template(&self, template: OrderTemplate) -> Result<String, AppError> {
        template.validate()
            .map_err(|e| AppError::Validation(e))?;
        
        let template_id = template.id.clone();
        self.order_templates.write().await.insert(template_id.clone(), template);
        inf!("Created order template: {}", template_id);
        Ok(template_id)
    }
    
    pub async fn update_template(&self, template: OrderTemplate) -> Result<(), AppError> {
        template.validate()
            .map_err(|e| AppError::Validation(e))?;
        
        let mut templates = self.order_templates.write().await;
        if templates.contains_key(&template.id) {
            let template_id = template.id.clone();
            templates.insert(template_id.clone(), template);
            inf!("Updated order template: {}", template_id);
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Template {} not found", template.id)))
        }
    }
    
    pub async fn delete_template(&self, template_id: &str) -> Result<(), AppError> {
        let mut templates = self.order_templates.write().await;
        if let Some(template) = templates.get(template_id) {
            if template.is_active() {
                return Err(AppError::Validation("Cannot delete active template".to_string()));
            }
            templates.remove(template_id);
            inf!("Deleted order template: {}", template_id);
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Template {} not found", template_id)))
        }
    }
    
    pub async fn get_template(&self, template_id: &str) -> Option<OrderTemplate> {
        self.order_templates.read().await.get(template_id).cloned()
    }
    
    pub async fn get_all_templates(&self) -> Vec<OrderTemplate> {
        self.order_templates.read().await.values().cloned().collect()
    }
    
    // Order activation
    pub async fn activate_template(&self, template_id: &str) -> Result<(), AppError> {
        let client = self.get_active_client().await?;
        
        let mut templates = self.order_templates.write().await;
        let template = templates.get_mut(template_id)
            .ok_or(AppError::NotFound(format!("Template {} not found", template_id)))?;
        
        if !template.can_activate() {
            return Err(AppError::Validation("Template cannot be activated in current state".to_string()));
        }
        
        // Create contract
        let contract = Contract::stock(&template.symbol);
        
        // Get order IDs
        let parent_order_id = self.get_next_order_id().await;
        let stop_order_id = parent_order_id + 1;
        
        // Create parent limit order
        let mut parent_order = orders::Order::default();
        parent_order.action = template.side.to_action();
        parent_order.order_type = "LMT".to_string();
        parent_order.total_quantity = template.quantity;
        parent_order.limit_price = Some(template.limit_price);
        parent_order.tif = template.time_in_force.to_string();
        parent_order.transmit = false; // Don't transmit until stop is attached
        
        // Create attached stop order
        let mut stop_order = orders::Order::default();
        stop_order.action = template.side.stop_action();
        stop_order.order_type = "STP".to_string();
        stop_order.total_quantity = template.quantity;
        stop_order.aux_price = Some(template.stop_price);
        stop_order.parent_id = parent_order_id;
        stop_order.tif = "GTC".to_string(); // Stop is always GTC
        stop_order.transmit = true; // This will transmit both orders
        
        // Update template status
        template.status = OrderTemplateStatus::Activating;
        template.parent_order_id = Some(parent_order_id);
        template.stop_order_id = Some(stop_order_id);
        
        let template_id_clone = template_id.to_string();
        let active_orders = self.active_orders.clone();
        
        // Place orders in a blocking task
        let client_clone = client.clone();
        let contract_clone = contract.clone();
        let parent_order_clone = parent_order.clone();
        let stop_order_clone = stop_order.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            let client_guard = futures::executor::block_on(client_clone.lock());
            
            // Place parent order
            let parent_result = client_guard.place_order(parent_order_id, &contract_clone, &parent_order_clone);
            if parent_result.is_err() {
                return Err(parent_result.unwrap_err());
            }
            
            // Place stop order
            let stop_result = client_guard.place_order(stop_order_id, &contract_clone, &stop_order_clone);
            if stop_result.is_err() {
                // Try to cancel parent order if stop fails
                let _ = client_guard.cancel_order(parent_order_id, "");
                return Err(stop_result.unwrap_err());
            }
            
            Ok(())
        }).await
        .map_err(|e| AppError::IBConnection(format!("Task join error: {}", e)))?;
        
        match result {
            Ok(()) => {
                // Track orders
                active_orders.lock().await.insert(parent_order_id, template_id_clone.clone());
                active_orders.lock().await.insert(stop_order_id, template_id_clone.clone());
                
                // Update template status
                template.status = OrderTemplateStatus::Active;
                template.activated_at = Some(chrono::Utc::now());
                
                inf!("Activated template {} with orders {} and {}", template_id, parent_order_id, stop_order_id);
                Ok(())
            }
            Err(e) => {
                err!("Failed to place orders: {}", e);
                template.status = OrderTemplateStatus::Failed;
                template.parent_order_id = None;
                template.stop_order_id = None;
                Err(AppError::IBConnection(format!("Failed to place orders: {}", e)))
            }
        }
    }
    
    pub async fn deactivate_template(&self, template_id: &str) -> Result<(), AppError> {
        let client = self.get_active_client().await?;
        
        let mut templates = self.order_templates.write().await;
        let template = templates.get_mut(template_id)
            .ok_or(AppError::NotFound(format!("Template {} not found", template_id)))?;
        
        if !template.can_deactivate() {
            return Err(AppError::Validation("Template cannot be deactivated in current state".to_string()));
        }
        
        template.status = OrderTemplateStatus::Deactivating;
        
        // Cancel both orders in blocking task
        let client_clone = client.clone();
        let parent_id = template.parent_order_id;
        let stop_id = template.stop_order_id;
        
        let result = tokio::task::spawn_blocking(move || {
            let client_guard = futures::executor::block_on(client_clone.lock());
            let mut errors = Vec::new();
            
            if let Some(parent_id) = parent_id {
                if let Err(e) = client_guard.cancel_order(parent_id, "") {
                    errors.push(format!("Failed to cancel parent order {}: {}", parent_id, e));
                }
            }
            
            if let Some(stop_id) = stop_id {
                if let Err(e) = client_guard.cancel_order(stop_id, "") {
                    errors.push(format!("Failed to cancel stop order {}: {}", stop_id, e));
                }
            }
            
            errors
        }).await
        .map_err(|e| AppError::IBConnection(format!("Task join error: {}", e)))?;
        
        let errors = result;
        
        // Update active orders
        if let Some(parent_id) = template.parent_order_id {
            if !errors.iter().any(|e| e.contains(&format!("parent order {}", parent_id))) {
                self.active_orders.lock().await.remove(&parent_id);
            }
        }
        if let Some(stop_id) = template.stop_order_id {
            if !errors.iter().any(|e| e.contains(&format!("stop order {}", stop_id))) {
                self.active_orders.lock().await.remove(&stop_id);
            }
        }
        
        if errors.is_empty() {
            template.status = OrderTemplateStatus::Inactive;
            template.parent_order_id = None;
            template.stop_order_id = None;
            inf!("Deactivated template {}", template_id);
            Ok(())
        } else {
            template.status = OrderTemplateStatus::Failed;
            Err(AppError::IBConnection(errors.join(", ")))
        }
    }
    
    // Market data
    pub async fn subscribe_market_data(&self, symbol: &str) -> Result<(), AppError> {
        // TODO: Implement market data subscription with sync API
        // For now, just log the request
        inf!("Market data subscription requested for {} (not yet implemented)", symbol);
        Ok(())
    }
    
    pub async fn unsubscribe_market_data(&self, symbol: &str) {
        self.market_data.write().await.remove(symbol);
        inf!("Unsubscribed from market data for {}", symbol);
    }
    
    pub async fn get_market_data(&self, symbol: &str) -> Option<MarketData> {
        self.market_data.read().await.get(symbol).cloned()
    }
    
    // Historical data
    pub async fn get_historical_data(
        &self, 
        symbol: &str, 
        duration_days: u32,
        bar_size: &str,  // e.g., "1 day", "1 hour"
    ) -> Result<HistoricalData, AppError> {
        let client = self.get_active_client().await?;
        let contract = Contract::stock(symbol);
        
        inf!("Fetching historical data for {} - {} days of {} bars", symbol, duration_days, bar_size);
        
        // Convert bar size string to enum
        // Note: Check ibapi docs for all available bar sizes
        let bar_size_enum = match bar_size {
            "1 day" => HistoricalBarSize::Day,
            "1 hour" => HistoricalBarSize::Hour,
            _ => {
                return Err(AppError::Validation(format!("Unsupported bar size: {}. Currently only '1 day' and '1 hour' are supported.", bar_size)));
            }
        };
        
        let symbol_clone = symbol.to_string();
        let bar_size_clone = bar_size.to_string();
        let duration_str = format!("{} days", duration_days);
        
        // Run in blocking task
        let client_clone = client.clone();
        let contract_clone = contract.clone();
        let result = tokio::task::spawn_blocking(move || {
            use ibapi::market_data::historical::Duration;
            
            let client_guard = futures::executor::block_on(client_clone.lock());
            let duration = Duration::days(duration_days as i32);
            
            // Request historical data
            client_guard.historical_data(
                &contract_clone,
                None, // end date time (None = now)
                duration,
                bar_size_enum,
                HistoricalWhatToShow::Trades,
                true, // use RTH (regular trading hours)
            )
        }).await
        .map_err(|e| AppError::IBConnection(format!("Task join error: {}", e)))?;
        
        match result {
            Ok(hist_data) => {
                let mut historical_data = HistoricalData::new(
                    symbol_clone,
                    bar_size_clone,
                    duration_str,
                );
                
                // Convert IB bars to our HistoricalBar format
                for bar in hist_data.bars {
                    // bar.date is an OffsetDateTime from the time crate
                    // Convert it to chrono DateTime
                    let timestamp = chrono::DateTime::from_timestamp(
                        bar.date.unix_timestamp(),
                        bar.date.nanosecond(),
                    ).unwrap_or_else(|| chrono::Utc::now());
                    
                    let hist_bar = HistoricalBar {
                        timestamp,
                        open: bar.open,
                        high: bar.high,
                        low: bar.low,
                        close: bar.close,
                        volume: bar.volume as i64,
                        wap: bar.wap,
                        count: bar.count as i64,
                    };
                    historical_data.add_bar(hist_bar);
                }
                
                inf!("Received {} historical bars for {}", historical_data.bars.len(), symbol);
                historical_data.sort_by_time();
                Ok(historical_data)
            }
            Err(e) => {
                err!("Failed to fetch historical data: {}", e);
                Err(AppError::IBConnection(format!("Historical data request failed: {}", e)))
            }
        }
    }
    
    // Calculate ATR with outlier filtering
    pub async fn calculate_filtered_atr(
        &self,
        symbol: &str,
        period_days: usize,
        method: OutlierMethod,
    ) -> Result<ATRResult, AppError> {
        // Fetch more days to ensure we have enough after filtering
        let fetch_days = (period_days * 3).max(30).min(60) as u32;
        
        inf!("Calculating filtered ATR for {} - {} days period", symbol, period_days);
        
        // Get historical data
        let historical_data = self.get_historical_data(symbol, fetch_days, "1 day").await?;
        
        if historical_data.bars.is_empty() {
            return Err(AppError::Validation("No historical data available".to_string()));
        }
        
        let mut result = ATRResult::new(symbol.to_string(), period_days, method);
        result.total_bars = historical_data.bars.len();
        
        // Calculate ranges for all bars
        let mut ranges: Vec<(usize, f64)> = historical_data.bars
            .iter()
            .enumerate()
            .map(|(idx, bar)| (idx, bar.high - bar.low))
            .collect();
        
        // Sort ranges for percentile calculations
        let mut sorted_ranges: Vec<f64> = ranges.iter().map(|(_, r)| *r).collect();
        sorted_ranges.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        // Calculate statistics
        let n = sorted_ranges.len();
        result.mean_range = sorted_ranges.iter().sum::<f64>() / n as f64;
        result.median_range = if n % 2 == 0 {
            (sorted_ranges[n/2 - 1] + sorted_ranges[n/2]) / 2.0
        } else {
            sorted_ranges[n/2]
        };
        
        // Calculate standard deviation
        let variance = sorted_ranges.iter()
            .map(|r| (r - result.mean_range).powi(2))
            .sum::<f64>() / n as f64;
        result.std_dev_range = variance.sqrt();
        
        // Calculate quartiles
        result.q1_range = sorted_ranges[n / 4];
        result.q3_range = sorted_ranges[3 * n / 4];
        result.iqr = result.q3_range - result.q1_range;
        
        // Determine outlier bounds based on method
        let (lower_bound, upper_bound) = match method {
            OutlierMethod::IQR { multiplier } => {
                let lb = result.q1_range - multiplier * result.iqr;
                let ub = result.q3_range + multiplier * result.iqr;
                (lb.max(0.0), ub)
            }
            OutlierMethod::ZScore { threshold } => {
                let lb = result.mean_range - threshold * result.std_dev_range;
                let ub = result.mean_range + threshold * result.std_dev_range;
                (lb.max(0.0), ub)
            }
            OutlierMethod::Percentile { low, high } => {
                let low_idx = ((low / 100.0) * n as f64) as usize;
                let high_idx = ((high / 100.0) * n as f64) as usize;
                (sorted_ranges[low_idx], sorted_ranges[high_idx.min(n-1)])
            }
        };
        
        result.lower_bound = lower_bound;
        result.upper_bound = upper_bound;
        
        // Filter bars and collect details
        let mut filtered_bars = Vec::new();
        let mut excluded_bars = Vec::new();
        
        for (idx, range) in ranges.iter().rev().take(fetch_days as usize) {
            let bar = &historical_data.bars[*idx];
            
            if *range < lower_bound || *range > upper_bound {
                // This bar is an outlier
                let reason = if *range < lower_bound {
                    format!("Range {:.2} below lower bound {:.2}", range, lower_bound)
                } else {
                    format!("Range {:.2} above upper bound {:.2}", range, upper_bound)
                };
                
                excluded_bars.push(ExcludedBar {
                    date: bar.timestamp,
                    range: *range,
                    reason,
                    high: bar.high,
                    low: bar.low,
                });
            } else {
                // This bar is normal
                filtered_bars.push(bar.clone());
                
                // Stop if we have enough bars for the requested period
                if filtered_bars.len() >= period_days {
                    break;
                }
            }
        }
        
        // Update result with filtering details
        result.used_bars = filtered_bars.len();
        result.excluded_bars = excluded_bars.len();
        result.exclusion_rate = if result.total_bars > 0 {
            excluded_bars.len() as f64 / result.total_bars as f64
        } else {
            0.0
        };
        
        result.excluded_bars_detail = excluded_bars;
        result.used_bars_detail = filtered_bars.clone();
        
        // Check if we have enough bars
        result.is_valid = result.used_bars >= period_days;
        
        if !result.is_valid {
            wrn!("Not enough valid bars for ATR calculation. Got {} valid bars, need {}", 
                result.used_bars, period_days);
        }
        
        // Calculate filtered ATR (simple average of ranges for now)
        if result.used_bars > 0 {
            let filtered_ranges: Vec<f64> = filtered_bars.iter()
                .take(period_days)
                .map(|bar| bar.high - bar.low)
                .collect();
            
            result.filtered_atr = filtered_ranges.iter().sum::<f64>() / filtered_ranges.len() as f64;
        }
        
        // Calculate regular ATR for comparison (using all bars)
        let regular_ranges: Vec<f64> = historical_data.bars.iter()
            .rev()
            .take(period_days)
            .map(|bar| bar.high - bar.low)
            .collect();
        
        if !regular_ranges.is_empty() {
            result.regular_atr = regular_ranges.iter().sum::<f64>() / regular_ranges.len() as f64;
        }
        
        // Calculate differences
        if result.regular_atr > 0.0 {
            result.atr_difference = result.filtered_atr - result.regular_atr;
            result.atr_difference_percent = (result.atr_difference / result.regular_atr) * 100.0;
        }
        
        // Calculate confidence score
        result.calculate_confidence();
        
        inf!("ATR calculation complete. Filtered: {:.2}, Regular: {:.2}, Excluded {} bars", 
            result.filtered_atr, result.regular_atr, result.excluded_bars);
        
        Ok(result)
    }
}