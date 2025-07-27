use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::AppError;
use crate::{err, inf};
use super::types::OrderTemplate;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderTemplateStorage {
    pub templates: Vec<OrderTemplate>,
}

impl OrderTemplateStorage {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }
    
    pub async fn load_from_file(path: &PathBuf) -> Result<Self, AppError> {
        match fs::read_to_string(path).await {
            Ok(content) => {
                let storage: OrderTemplateStorage = serde_json::from_str(&content)
                    .map_err(|e| AppError::Serialization(format!("Failed to parse templates: {}", e)))?;
                inf!("Loaded {} order templates from file", storage.templates.len());
                Ok(storage)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                inf!("No existing order templates file found, creating new");
                Ok(Self::new())
            }
            Err(e) => {
                err!("Failed to read order templates file: {}", e);
                Err(AppError::Io(e))
            }
        }
    }
    
    pub async fn save_to_file(&self, path: &PathBuf) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize templates: {}", e)))?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AppError::Io(e))?;
        }
        
        fs::write(path, json).await
            .map_err(|e| AppError::Io(e))?;
        
        inf!("Saved {} order templates to file", self.templates.len());
        Ok(())
    }
    
    pub fn add_template(&mut self, template: OrderTemplate) {
        self.templates.push(template);
    }
    
    pub fn update_template(&mut self, template: OrderTemplate) -> Result<(), AppError> {
        if let Some(existing) = self.templates.iter_mut().find(|t| t.id == template.id) {
            *existing = template;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Template {} not found", template.id)))
        }
    }
    
    pub fn remove_template(&mut self, template_id: &str) -> Result<(), AppError> {
        let initial_len = self.templates.len();
        self.templates.retain(|t| t.id != template_id);
        
        if self.templates.len() < initial_len {
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Template {} not found", template_id)))
        }
    }
    
    pub fn get_template(&self, template_id: &str) -> Option<&OrderTemplate> {
        self.templates.iter().find(|t| t.id == template_id)
    }
    
    pub fn get_all_templates(&self) -> &[OrderTemplate] {
        &self.templates
    }
}

// Helper functions for order calculations
pub mod calculations {
    use super::*;
    
    pub fn calculate_risk(template: &OrderTemplate) -> f64 {
        let price_diff = (template.limit_price - template.stop_price).abs();
        price_diff * template.quantity
    }
    
    pub fn calculate_reward_risk_ratio(template: &OrderTemplate, target_price: f64) -> f64 {
        let risk = (template.limit_price - template.stop_price).abs();
        let reward = (target_price - template.limit_price).abs();
        
        if risk > 0.0 {
            reward / risk
        } else {
            0.0
        }
    }
    
    pub fn calculate_position_value(template: &OrderTemplate) -> f64 {
        template.limit_price * template.quantity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_order_template_storage() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("templates.json");
        
        // Create and save templates
        let mut storage = OrderTemplateStorage::new();
        let template = OrderTemplate::new(
            "Test Template".to_string(),
            "AAPL".to_string(),
            crate::ib::types::OrderSide::Long,
            100.0,
            150.0,
            145.0,
            crate::ib::types::TimeInForce::Day,
            crate::ib::types::TradingModel::default(),
        );
        
        storage.add_template(template.clone());
        storage.save_to_file(&file_path).await.unwrap();
        
        // Load and verify
        let loaded_storage = OrderTemplateStorage::load_from_file(&file_path).await.unwrap();
        assert_eq!(loaded_storage.templates.len(), 1);
        assert_eq!(loaded_storage.templates[0].id, template.id);
    }
    
    #[test]
    fn test_risk_calculations() {
        let template = OrderTemplate::new(
            "Test".to_string(),
            "AAPL".to_string(),
            crate::ib::types::OrderSide::Long,
            100.0,
            150.0,
            145.0,
            crate::ib::types::TimeInForce::Day,
            crate::ib::types::TradingModel::default(),
        );
        
        assert_eq!(calculations::calculate_risk(&template), 500.0); // (150-145) * 100
        assert_eq!(calculations::calculate_position_value(&template), 15000.0); // 150 * 100
        
        let rr_ratio = calculations::calculate_reward_risk_ratio(&template, 160.0);
        assert_eq!(rr_ratio, 2.0); // reward: 10, risk: 5
    }
}