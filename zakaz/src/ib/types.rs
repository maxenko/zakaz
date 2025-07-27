use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Long,
    Short,
}

impl OrderSide {
    pub fn to_action(&self) -> ibapi::orders::Action {
        match self {
            OrderSide::Long => ibapi::orders::Action::Buy,
            OrderSide::Short => ibapi::orders::Action::Sell,
        }
    }
    
    pub fn stop_action(&self) -> ibapi::orders::Action {
        match self {
            OrderSide::Long => ibapi::orders::Action::Sell,
            OrderSide::Short => ibapi::orders::Action::Buy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    Day,
    GTC,
}

impl TimeInForce {
    pub fn to_string(&self) -> String {
        match self {
            TimeInForce::Day => "DAY".to_string(),
            TimeInForce::GTC => "GTC".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderTemplateStatus {
    Inactive,      // Not sent to IB
    Activating,    // Being sent to IB
    Active,        // Live on IB
    Deactivating,  // Being canceled on IB
    Failed,        // Failed to activate/deactivate
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingModel {
    Breakout,      // Price breaks through resistance/support
    FalseBreakout, // Failed breakout, reversal trade
    Bounce,        // Price bounces off support/resistance
    Continuation,  // Trend continuation pattern
}

impl Default for TradingModel {
    fn default() -> Self {
        TradingModel::Breakout
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTemplate {
    pub id: String,                    // Local template ID
    pub name: String,                  // User-friendly name
    pub symbol: String,                // Stock symbol
    pub side: OrderSide,               // Long or Short
    pub quantity: f64,                 // Number of shares
    pub limit_price: f64,              // Entry limit price
    pub stop_price: f64,               // Stop loss price (calculated)
    pub technical_stop_price: Option<f64>, // Technical adjustment stop
    pub time_in_force: TimeInForce,   // DAY or GTC for main order
    pub status: OrderTemplateStatus,   // Current status
    pub parent_order_id: Option<i32>,  // IB order ID when active
    pub stop_order_id: Option<i32>,    // IB stop order ID when active
    pub created_at: DateTime<Utc>,     // When template was created
    pub activated_at: Option<DateTime<Utc>>, // When last activated
    pub notes: Option<String>,         // User notes
    pub model: TradingModel,           // Trading model/strategy type
    pub is_read_only: bool,            // For IB positions without templates
    pub risk_per_trade: f64,           // Risk amount for position sizing
}

impl OrderTemplate {
    pub fn new(
        name: String,
        symbol: String,
        side: OrderSide,
        quantity: f64,
        limit_price: f64,
        stop_price: f64,
        time_in_force: TimeInForce,
        model: TradingModel,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            symbol,
            side,
            quantity,
            limit_price,
            stop_price,
            technical_stop_price: None,
            time_in_force,
            status: OrderTemplateStatus::Inactive,
            parent_order_id: None,
            stop_order_id: None,
            created_at: Utc::now(),
            activated_at: None,
            notes: None,
            model,
            is_read_only: false,
            risk_per_trade: 100.0, // Default risk per trade
        }
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderTemplateStatus::Active)
    }
    
    pub fn can_activate(&self) -> bool {
        matches!(self.status, OrderTemplateStatus::Inactive | OrderTemplateStatus::Failed)
    }
    
    pub fn can_deactivate(&self) -> bool {
        matches!(self.status, OrderTemplateStatus::Active)
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.quantity <= 0.0 {
            return Err("Quantity must be positive".to_string());
        }
        
        if self.limit_price <= 0.0 {
            return Err("Limit price must be positive".to_string());
        }
        
        if self.stop_price <= 0.0 {
            return Err("Stop price must be positive".to_string());
        }
        
        // Validate stop placement relative to side
        match self.side {
            OrderSide::Long => {
                if self.stop_price >= self.limit_price {
                    return Err("For long orders, stop price must be below limit price".to_string());
                }
            }
            OrderSide::Short => {
                if self.stop_price <= self.limit_price {
                    return Err("For short orders, stop price must be above limit price".to_string());
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_stop_loss(&self) -> f64 {
        // Return technical stop if set, otherwise use calculated stop
        self.technical_stop_price.unwrap_or(self.stop_price)
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutlierMethod {
    IQR { multiplier: f64 },      // Default 1.5
    ZScore { threshold: f64 },    // Default 2.0
    Percentile { low: f64, high: f64 }, // Default 10th-90th
}

impl Default for OutlierMethod {
    fn default() -> Self {
        OutlierMethod::IQR { multiplier: 1.5 }
    }
}

#[derive(Debug, Clone)]
pub struct ExcludedBar {
    pub date: chrono::DateTime<chrono::Utc>,
    pub range: f64,
    pub reason: String,
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone)]
pub struct ATRResult {
    pub symbol: String,
    pub period_days: usize,
    pub calculation_date: chrono::DateTime<chrono::Utc>,
    
    // ATR values
    pub filtered_atr: f64,
    pub regular_atr: f64,
    pub atr_difference: f64,
    pub atr_difference_percent: f64,
    
    // Statistics
    pub total_bars: usize,
    pub used_bars: usize,
    pub excluded_bars: usize,
    pub exclusion_rate: f64,
    
    // Range statistics
    pub mean_range: f64,
    pub median_range: f64,
    pub std_dev_range: f64,
    pub q1_range: f64,
    pub q3_range: f64,
    pub iqr: f64,
    pub lower_bound: f64,  // Q1 - 1.5*IQR
    pub upper_bound: f64,  // Q3 + 1.5*IQR
    
    // Details
    pub method: OutlierMethod,
    pub excluded_bars_detail: Vec<ExcludedBar>,
    pub used_bars_detail: Vec<HistoricalBar>,
    
    // Confidence metrics
    pub confidence_score: f64,  // 0-100, based on sample size and consistency
    pub is_valid: bool,         // true if we have minimum required bars
}

impl ATRResult {
    pub fn new(symbol: String, period_days: usize, method: OutlierMethod) -> Self {
        Self {
            symbol,
            period_days,
            calculation_date: chrono::Utc::now(),
            filtered_atr: 0.0,
            regular_atr: 0.0,
            atr_difference: 0.0,
            atr_difference_percent: 0.0,
            total_bars: 0,
            used_bars: 0,
            excluded_bars: 0,
            exclusion_rate: 0.0,
            mean_range: 0.0,
            median_range: 0.0,
            std_dev_range: 0.0,
            q1_range: 0.0,
            q3_range: 0.0,
            iqr: 0.0,
            lower_bound: 0.0,
            upper_bound: 0.0,
            method,
            excluded_bars_detail: Vec::new(),
            used_bars_detail: Vec::new(),
            confidence_score: 0.0,
            is_valid: false,
        }
    }
    
    pub fn calculate_confidence(&mut self) {
        // Confidence based on:
        // 1. Sample size (more bars = higher confidence)
        // 2. Exclusion rate (moderate exclusion = good, too high = bad)
        // 3. Consistency (lower std dev = higher confidence)
        
        let sample_score = (self.used_bars.min(14) as f64 / 14.0) * 40.0; // Max 40 points
        
        let exclusion_score = if self.exclusion_rate < 0.1 {
            30.0 // Very few exclusions, might not be filtering enough
        } else if self.exclusion_rate < 0.3 {
            40.0 // Good range of exclusions
        } else if self.exclusion_rate < 0.5 {
            20.0 // High exclusions
        } else {
            10.0 // Too many exclusions
        };
        
        let consistency_score = if self.mean_range > 0.0 {
            let cv = self.std_dev_range / self.mean_range; // Coefficient of variation
            ((1.0 - cv.min(1.0)) * 20.0).max(0.0)
        } else {
            0.0
        };
        
        self.confidence_score = sample_score + exclusion_score + consistency_score;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalBar {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub wap: f64,  // Weighted Average Price
    pub count: i64, // Number of trades
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    pub symbol: String,
    pub bars: Vec<HistoricalBar>,
    pub bar_size: String,  // e.g., "1 day", "1 hour"
    pub duration: String,  // e.g., "14 D", "1 M"
}

impl HistoricalData {
    pub fn new(symbol: String, bar_size: String, duration: String) -> Self {
        Self {
            symbol,
            bars: Vec::new(),
            bar_size,
            duration,
        }
    }
    
    pub fn add_bar(&mut self, bar: HistoricalBar) {
        self.bars.push(bar);
    }
    
    pub fn sort_by_time(&mut self) {
        self.bars.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_template_validation() {
        let mut template = OrderTemplate::new(
            "Test Long".to_string(),
            "AAPL".to_string(),
            OrderSide::Long,
            100.0,
            150.0,
            145.0,
            TimeInForce::Day,
            TradingModel::Breakout,
        );
        
        assert!(template.validate().is_ok());
        
        // Test invalid stop for long
        template.stop_price = 155.0;
        assert!(template.validate().is_err());
        
        // Test short order
        let mut short_template = OrderTemplate::new(
            "Test Short".to_string(),
            "AAPL".to_string(),
            OrderSide::Short,
            100.0,
            150.0,
            155.0,
            TimeInForce::GTC,
            TradingModel::Bounce,
        );
        
        assert!(short_template.validate().is_ok());
        
        // Test invalid stop for short
        short_template.stop_price = 145.0;
        assert!(short_template.validate().is_err());
    }
}