use chrono::Utc;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::ib::types::{OrderSide, TradingModel};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbOrderTemplate {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub side: String, // Will be converted to/from OrderSide
    pub quantity: i64,
    pub limit_price: f64,
    pub stop_price: f64,
    pub technical_stop_price: Option<f64>,
    pub time_in_force: String,
    pub model: String, // Will be converted to/from TradingModel
    pub status: String, // Will be converted to/from OrderStatus
    pub is_read_only: bool,
    pub risk_per_trade: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Template,   // Not yet submitted to IB
    Active,     // Submitted to IB
    Filled,     // Order executed
    Cancelled,  // Order cancelled
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Template => "Template",
            OrderStatus::Active => "Active",
            OrderStatus::Filled => "Filled",
            OrderStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Template" => Some(OrderStatus::Template),
            "Active" => Some(OrderStatus::Active),
            "Filled" => Some(OrderStatus::Filled),
            "Cancelled" => Some(OrderStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbActiveOrder {
    pub template_id: String,
    pub ib_order_id: i64,
    pub ib_stop_order_id: Option<i64>,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbPosition {
    pub ib_position_id: String,
    pub template_id: Option<String>,
    pub symbol: String,
    pub quantity: i64,
    pub avg_cost: f64,
    pub is_read_only: bool,
    pub synced_at: String,
}

// Conversion helpers
impl DbOrderTemplate {
    pub fn new(
        name: String,
        symbol: String,
        side: OrderSide,
        quantity: i64,
        limit_price: f64,
        stop_price: f64,
        model: TradingModel,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            symbol,
            side: match side {
                OrderSide::Long => "Buy".to_string(),
                OrderSide::Short => "Sell".to_string(),
            },
            quantity,
            limit_price,
            stop_price,
            technical_stop_price: None,
            time_in_force: "GTC".to_string(),
            model: match model {
                TradingModel::Breakout => "Breakout",
                TradingModel::FalseBreakout => "FalseBreakout",
                TradingModel::Bounce => "Bounce",
                TradingModel::Continuation => "Continuation",
            }.to_string(),
            status: OrderStatus::Template.as_str().to_string(),
            is_read_only: false,
            risk_per_trade: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn get_order_side(&self) -> Option<OrderSide> {
        match self.side.as_str() {
            "Buy" => Some(OrderSide::Long),
            "Sell" => Some(OrderSide::Short),
            _ => None,
        }
    }

    pub fn get_trading_model(&self) -> Option<TradingModel> {
        match self.model.as_str() {
            "Breakout" => Some(TradingModel::Breakout),
            "FalseBreakout" => Some(TradingModel::FalseBreakout),
            "Bounce" => Some(TradingModel::Bounce),
            "Continuation" => Some(TradingModel::Continuation),
            _ => None,
        }
    }

    pub fn get_order_status(&self) -> Option<OrderStatus> {
        OrderStatus::from_str(&self.status)
    }

    pub fn get_stop_loss(&self) -> f64 {
        // Return technical stop if set, otherwise use calculated stop
        self.technical_stop_price.unwrap_or(self.stop_price)
    }
}