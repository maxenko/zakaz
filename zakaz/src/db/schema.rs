use sqlx::sqlite::SqlitePool;

pub const DATABASE_URL: &str = "sqlite:zakaz.db";

pub async fn create_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Templates table: All order templates
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK (side IN ('Buy', 'Sell')),
            quantity INTEGER NOT NULL,
            limit_price REAL NOT NULL,
            stop_price REAL NOT NULL,
            technical_stop_price REAL,
            time_in_force TEXT NOT NULL DEFAULT 'GTC',
            model TEXT NOT NULL CHECK (model IN ('Breakout', 'FalseBreakout', 'Bounce', 'Continuation')),
            status TEXT NOT NULL CHECK (status IN ('Template', 'Active', 'Filled', 'Cancelled')),
            is_read_only BOOLEAN NOT NULL DEFAULT 0,
            risk_per_trade REAL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#
    )
    .execute(pool)
    .await?;

    // Active orders table: Template ID + IB order ID mapping
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS active_orders (
            template_id TEXT NOT NULL,
            ib_order_id INTEGER NOT NULL,
            ib_stop_order_id INTEGER,
            submitted_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (template_id, ib_order_id),
            FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE CASCADE
        )
        "#
    )
    .execute(pool)
    .await?;

    // Settings table: Risk parameters, ATR settings
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#
    )
    .execute(pool)
    .await?;

    // Positions table: IB positions with optional template association
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS positions (
            ib_position_id TEXT PRIMARY KEY,
            template_id TEXT,
            symbol TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            avg_cost REAL NOT NULL,
            is_read_only BOOLEAN NOT NULL DEFAULT 1,
            synced_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE SET NULL
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes for performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_templates_symbol ON templates(symbol)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_templates_status ON templates(status)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_active_orders_ib_order_id ON active_orders(ib_order_id)")
        .execute(pool)
        .await?;

    // Create triggers to update timestamps
    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS update_templates_timestamp 
        AFTER UPDATE ON templates
        BEGIN
            UPDATE templates SET updated_at = datetime('now') WHERE id = NEW.id;
        END
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS update_settings_timestamp 
        AFTER UPDATE ON settings
        BEGIN
            UPDATE settings SET updated_at = datetime('now') WHERE key = NEW.key;
        END
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Initialize default settings
pub async fn init_default_settings(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Default risk per trade: $100
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('risk_per_trade', '100.0')")
        .execute(pool)
        .await?;
    
    // Default ATR period: 14
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('atr_period', '14')")
        .execute(pool)
        .await?;
    
    // Default ATR multiplier for outlier filtering: 2.5
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('atr_outlier_multiplier', '2.5')")
        .execute(pool)
        .await?;
    
    // Default stop loss percentage of ATR: 10%
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('stop_loss_atr_percentage', '0.10')")
        .execute(pool)
        .await?;
    
    // Maximum technical adjustment percentage of ATR: 15%
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('max_technical_stop_atr_percentage', '0.15')")
        .execute(pool)
        .await?;

    Ok(())
}