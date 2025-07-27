use super::types::{OrderTemplate, OrderSide, TimeInForce, ATRResult, OutlierMethod, TradingModel};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum IBMessage {
    // Connection management
    ConnectPaper {
        response: oneshot::Sender<Result<(), String>>,
    },
    ConnectLive {
        response: oneshot::Sender<Result<(), String>>,
    },
    Disconnect,
    SwitchToPaper {
        response: oneshot::Sender<Result<(), String>>,
    },
    SwitchToLive {
        response: oneshot::Sender<Result<(), String>>,
    },
    GetConnectionStatus {
        response: oneshot::Sender<ConnectionStatus>,
    },
    
    // Order template management
    CreateTemplate {
        name: String,
        symbol: String,
        side: OrderSide,
        quantity: f64,
        limit_price: f64,
        stop_price: f64,
        time_in_force: TimeInForce,
        model: TradingModel,
        response: oneshot::Sender<Result<String, String>>, // Returns template ID
    },
    UpdateTemplate {
        template: OrderTemplate,
        response: oneshot::Sender<Result<(), String>>,
    },
    DeleteTemplate {
        template_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    GetTemplate {
        template_id: String,
        response: oneshot::Sender<Option<OrderTemplate>>,
    },
    GetAllTemplates {
        response: oneshot::Sender<Vec<OrderTemplate>>,
    },
    
    // Order activation/deactivation
    ActivateTemplate {
        template_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    DeactivateTemplate {
        template_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    
    // Market data
    SubscribeMarketData {
        symbol: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    UnsubscribeMarketData {
        symbol: String,
    },
    
    // Account info
    GetAccountSummary {
        response: oneshot::Sender<Result<AccountSummary, String>>,
    },
    GetPositions {
        response: oneshot::Sender<Result<Vec<Position>, String>>,
    },
    
    // Historical data
    GetHistoricalData {
        symbol: String,
        duration_days: u32,
        bar_size: String,
        response: oneshot::Sender<Result<super::types::HistoricalData, String>>,
    },
    
    // ATR calculation
    CalculateFilteredATR {
        symbol: String,
        period_days: usize,
        method: OutlierMethod,
        response: oneshot::Sender<Result<ATRResult, String>>,
    },
}

#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub paper_connected: bool,
    pub live_connected: bool,
    pub active_account: Option<super::AccountType>,
}

#[derive(Debug, Clone)]
pub struct AccountSummary {
    pub account_id: String,
    pub net_liquidation: f64,
    pub total_cash_value: f64,
    pub buying_power: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub position: f64,
    pub average_cost: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}