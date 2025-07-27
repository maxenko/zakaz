# Zakaz Type Definitions

## Order Types

### OrderTemplate
```rust
OrderTemplate {
    id: String,                    // UUID
    name: String,                  // User-friendly name
    symbol: String,                // Stock symbol
    side: OrderSide,               // Long or Short
    quantity: f64,                 // Number of shares
    limit_price: f64,              // Entry limit price
    stop_price: f64,               // Stop loss price (always GTC)
    time_in_force: TimeInForce,    // DAY or GTC for main order
    status: OrderTemplateStatus,   // Inactive/Active/etc
    parent_order_id: Option<i32>,  // IB order ID when active
    stop_order_id: Option<i32>,    // IB stop order ID
    created_at: DateTime<Utc>,     // When template was created
    activated_at: Option<DateTime<Utc>>, // When last activated
    notes: Option<String>,         // User notes
    model: TradingModel,           // Trading model/strategy type
}
```

### TradingModel
```rust
enum TradingModel {
    Breakout,      // Price breaks through resistance/support
    FalseBreakout, // Failed breakout, reversal trade
    Bounce,        // Price bounces off support/resistance
    Continuation,  // Trend continuation pattern
}
```

### OrderSide
```rust
enum OrderSide {
    Long,
    Short,
}
```

### TimeInForce
```rust
enum TimeInForce {
    Day,   // Order expires at end of trading day
    GTC,   // Good Till Canceled
}
```

### OrderTemplateStatus
```rust
enum OrderTemplateStatus {
    Inactive,      // Not sent to IB
    Activating,    // Being sent to IB
    Active,        // Live on IB
    Deactivating,  // Being canceled on IB
    Failed,        // Failed to activate/deactivate
}
```

## Market Data Types

### HistoricalData
```rust
HistoricalData {
    symbol: String,              // Stock symbol
    bars: Vec<HistoricalBar>,    // OHLC bars
    bar_size: String,            // "1 day", "1 hour"
    duration: String,            // "14 days"
}
```

### HistoricalBar
```rust
HistoricalBar {
    timestamp: DateTime<Utc>,    // Bar timestamp
    open: f64,                   // Open price
    high: f64,                   // High price
    low: f64,                    // Low price
    close: f64,                  // Close price
    volume: i64,                 // Volume
    wap: f64,                    // Weighted Average Price
    count: i64,                  // Number of trades
}
```

### MarketData
```rust
MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
    volume: i64,
    timestamp: DateTime<Utc>,
}
```

## ATR Calculation Types

### ATRResult
```rust
ATRResult {
    // Basic info
    symbol: String,
    period_days: usize,
    calculation_date: DateTime<Utc>,
    
    // ATR values
    filtered_atr: f64,              // ATR excluding outliers
    regular_atr: f64,               // Standard ATR
    atr_difference: f64,            // Filtered - Regular
    atr_difference_percent: f64,    // Percentage difference
    
    // Statistics
    total_bars: usize,              // Total bars analyzed
    used_bars: usize,               // Bars used in filtered ATR
    excluded_bars: usize,           // Number of outliers
    exclusion_rate: f64,            // Exclusion percentage
    
    // Range statistics
    mean_range: f64,                // Average daily range
    median_range: f64,              // Median daily range
    std_dev_range: f64,             // Standard deviation
    q1_range: f64,                  // 25th percentile
    q3_range: f64,                  // 75th percentile
    iqr: f64,                       // Interquartile range
    lower_bound: f64,               // Outlier lower threshold
    upper_bound: f64,               // Outlier upper threshold
    
    // Details
    method: OutlierMethod,          // IQR, ZScore, or Percentile
    excluded_bars_detail: Vec<ExcludedBar>,  // Excluded bar details
    used_bars_detail: Vec<HistoricalBar>,    // Used bar details
    
    // Validation
    confidence_score: f64,          // 0-100 confidence rating
    is_valid: bool,                 // true if enough bars
}
```

### OutlierMethod
```rust
enum OutlierMethod {
    IQR { multiplier: f64 },           // Default 1.5, robust to extremes
    ZScore { threshold: f64 },         // Default 2.0, more sensitive
    Percentile { low: f64, high: f64 }, // Fixed percentiles (e.g., 10th-90th)
}
```

### ExcludedBar
```rust
ExcludedBar {
    date: DateTime<Utc>,
    range: f64,
    reason: String,
    high: f64,
    low: f64,
}
```

## Account Types

### ConnectionStatus
```rust
ConnectionStatus {
    paper_connected: bool,
    live_connected: bool,
    active_account: Option<AccountType>,
}
```

### AccountSummary
```rust
AccountSummary {
    account_id: String,
    net_liquidation: f64,
    total_cash_value: f64,
    buying_power: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
}
```

### Position
```rust
Position {
    symbol: String,
    position: f64,
    average_cost: f64,
    market_value: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
}
```

## Chart Types

### ChartTheme
```rust
ChartTheme {
    colors: ChartColors {
        background: String,           // Hex color
        candle_bullish_body: String,
        candle_bearish_body: String,
        candle_wick: String,
        volume_bar: String,
        grid_line: String,
        text: String,
        crosshair: String,
    },
    candle_width_ratio: f64,         // 0.0-1.0, default 0.8
    volume_height_ratio: f64,        // 0.0-1.0, default 0.2
    grid_divisions: usize,           // Number of grid lines
    font_size: u32,                  // Label font size
}
```

## IB Message Types

See the full list of IB messages in `src/ib/messages.rs`:

### Connection Management
- `ConnectPaper` - Connect to paper trading account
- `ConnectLive` - Connect to live trading account  
- `Disconnect` - Disconnect from IB
- `SwitchToPaper` - Switch to paper account
- `SwitchToLive` - Switch to live account
- `GetConnectionStatus` - Get current connection status

### Order Templates
- `CreateTemplate` - Create new order template
- `UpdateTemplate` - Update existing template
- `DeleteTemplate` - Delete template
- `GetTemplate` - Get single template
- `GetAllTemplates` - Get all templates
- `ActivateTemplate` - Send template orders to IB
- `DeactivateTemplate` - Cancel template orders

### Market Data
- `SubscribeMarketData` - Subscribe to real-time data
- `UnsubscribeMarketData` - Unsubscribe from data
- `GetHistoricalData` - Fetch historical OHLC bars
- `CalculateFilteredATR` - Calculate ATR with outlier filtering

### Account Info
- `GetAccountSummary` - Get account summary
- `GetPositions` - Get current positions