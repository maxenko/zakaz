use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::inf;
use super::schema::{create_schema, init_default_settings, DATABASE_URL};
use super::models::{DbOrderTemplate, DbActiveOrder, DbPosition, OrderStatus};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Arc<Mutex<Self>>, sqlx::Error> {
        inf!("Initializing database connection");
        
        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(DATABASE_URL)
            .await?;
        
        // Create schema if needed
        create_schema(&pool).await?;
        
        // Initialize default settings
        init_default_settings(&pool).await?;
        
        inf!("Database initialized successfully");
        
        Ok(Arc::new(Mutex::new(Self { pool })))
    }

    // Template operations
    pub async fn create_template(&self, template: DbOrderTemplate) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO templates (
                id, name, symbol, side, quantity, limit_price, stop_price, 
                technical_stop_price, time_in_force, model, status, is_read_only, 
                risk_per_trade, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(&template.symbol)
        .bind(&template.side)
        .bind(template.quantity)
        .bind(template.limit_price)
        .bind(template.stop_price)
        .bind(template.technical_stop_price)
        .bind(&template.time_in_force)
        .bind(&template.model)
        .bind(&template.status)
        .bind(template.is_read_only)
        .bind(template.risk_per_trade)
        .bind(&template.created_at)
        .bind(&template.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_template(&self, id: &str) -> Result<Option<DbOrderTemplate>, sqlx::Error> {
        let template = sqlx::query_as::<_, DbOrderTemplate>(
            "SELECT * FROM templates WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(template)
    }

    pub async fn get_all_templates(&self) -> Result<Vec<DbOrderTemplate>, sqlx::Error> {
        let templates = sqlx::query_as::<_, DbOrderTemplate>(
            "SELECT * FROM templates ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(templates)
    }

    pub async fn get_templates_by_status(&self, status: OrderStatus) -> Result<Vec<DbOrderTemplate>, sqlx::Error> {
        let templates = sqlx::query_as::<_, DbOrderTemplate>(
            "SELECT * FROM templates WHERE status = ? ORDER BY created_at DESC"
        )
        .bind(status.as_str())
        .fetch_all(&self.pool)
        .await?;
        
        Ok(templates)
    }

    pub async fn update_template_status(&self, id: &str, status: OrderStatus) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE templates SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    pub async fn delete_template(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM templates WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    // Active order operations
    pub async fn create_active_order(&self, active_order: DbActiveOrder) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO active_orders (template_id, ib_order_id, ib_stop_order_id, submitted_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(&active_order.template_id)
        .bind(active_order.ib_order_id)
        .bind(active_order.ib_stop_order_id)
        .bind(&active_order.submitted_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_active_order_by_ib_id(&self, ib_order_id: i64) -> Result<Option<DbActiveOrder>, sqlx::Error> {
        let order = sqlx::query_as::<_, DbActiveOrder>(
            "SELECT * FROM active_orders WHERE ib_order_id = ?"
        )
        .bind(ib_order_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(order)
    }

    pub async fn delete_active_order(&self, template_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM active_orders WHERE template_id = ?")
            .bind(template_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    // Settings operations
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result.map(|(value,)| value))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_risk_per_trade(&self) -> Result<f64, sqlx::Error> {
        let value = self.get_setting("risk_per_trade").await?
            .unwrap_or_else(|| "100.0".to_string());
        
        Ok(value.parse::<f64>().unwrap_or(100.0))
    }

    // Position operations
    pub async fn sync_position(&self, position: DbPosition) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO positions (
                ib_position_id, template_id, symbol, quantity, avg_cost, is_read_only, synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&position.ib_position_id)
        .bind(&position.template_id)
        .bind(&position.symbol)
        .bind(position.quantity)
        .bind(position.avg_cost)
        .bind(position.is_read_only)
        .bind(&position.synced_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_all_positions(&self) -> Result<Vec<DbPosition>, sqlx::Error> {
        let positions = sqlx::query_as::<_, DbPosition>(
            "SELECT * FROM positions ORDER BY synced_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(positions)
    }

    pub async fn clear_positions(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM positions")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    // Transaction support
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>, sqlx::Error> {
        self.pool.begin().await
    }
}