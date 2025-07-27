use crate::ib::types::OrderSide;

/// Calculate position size based on risk per trade and stop loss distance
/// Formula: ORDER SIZE (SHARES) = RISK PER TRADE / STOP LOSS
pub fn calculate_position_size(
    risk_per_trade: f64,
    entry_price: f64,
    stop_price: f64,
    side: OrderSide,
) -> Result<i64, String> {
    // Calculate stop loss distance
    let stop_loss_distance = match side {
        OrderSide::Long => {
            if stop_price >= entry_price {
                return Err("For long orders, stop price must be below entry price".to_string());
            }
            entry_price - stop_price
        }
        OrderSide::Short => {
            if stop_price <= entry_price {
                return Err("For short orders, stop price must be above entry price".to_string());
            }
            stop_price - entry_price
        }
    };
    
    if stop_loss_distance <= 0.0 {
        return Err("Invalid stop loss distance".to_string());
    }
    
    // Calculate position size (shares)
    let position_size = risk_per_trade / stop_loss_distance;
    
    // Round down to nearest whole share
    let shares = position_size.floor() as i64;
    
    if shares <= 0 {
        return Err("Calculated position size is too small (less than 1 share)".to_string());
    }
    
    Ok(shares)
}

/// Validate stop loss placement relative to ATR
/// Stop loss must be between 0.01 below/above entry and 15% of ATR
pub fn validate_stop_loss(
    entry_price: f64,
    stop_price: f64,
    side: OrderSide,
    atr: f64,
) -> Result<(), String> {
    let min_distance = 0.01;
    let max_atr_percentage = 0.15; // 15% of ATR
    let max_distance = atr * max_atr_percentage;
    
    let stop_distance = match side {
        OrderSide::Long => {
            if stop_price >= entry_price {
                return Err("For long orders, stop price must be below entry price".to_string());
            }
            entry_price - stop_price
        }
        OrderSide::Short => {
            if stop_price <= entry_price {
                return Err("For short orders, stop price must be above entry price".to_string());
            }
            stop_price - entry_price
        }
    };
    
    if stop_distance < min_distance {
        return Err(format!(
            "Stop loss too close to entry. Minimum distance is ${:.2}",
            min_distance
        ));
    }
    
    if stop_distance > max_distance {
        return Err(format!(
            "Stop loss too far from entry. Maximum distance is ${:.2} (15% of ATR ${:.2})",
            max_distance, atr
        ));
    }
    
    Ok(())
}

/// Calculate default stop loss based on ATR (10% of ATR)
pub fn calculate_default_stop_loss(
    entry_price: f64,
    side: OrderSide,
    atr: f64,
) -> f64 {
    let stop_distance = atr * 0.10; // 10% of ATR
    
    match side {
        OrderSide::Long => entry_price - stop_distance,
        OrderSide::Short => entry_price + stop_distance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_size_long() {
        let risk = 100.0;
        let entry = 50.0;
        let stop = 48.0;
        let size = calculate_position_size(risk, entry, stop, OrderSide::Long).unwrap();
        assert_eq!(size, 50); // $100 risk / $2 stop = 50 shares
    }

    #[test]
    fn test_position_size_short() {
        let risk = 100.0;
        let entry = 50.0;
        let stop = 52.0;
        let size = calculate_position_size(risk, entry, stop, OrderSide::Short).unwrap();
        assert_eq!(size, 50); // $100 risk / $2 stop = 50 shares
    }

    #[test]
    fn test_invalid_stop_long() {
        let risk = 100.0;
        let entry = 50.0;
        let stop = 51.0; // Above entry for long
        let result = calculate_position_size(risk, entry, stop, OrderSide::Long);
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_validation() {
        let entry = 100.0;
        let atr = 2.0;
        
        // Valid stop for long
        let stop = 99.8; // 0.2 below entry, within 15% ATR (0.3)
        assert!(validate_stop_loss(entry, stop, OrderSide::Long, atr).is_ok());
        
        // Too close
        let stop = 99.995;
        assert!(validate_stop_loss(entry, stop, OrderSide::Long, atr).is_err());
        
        // Too far
        let stop = 99.0; // 1.0 below, exceeds 15% ATR (0.3)
        assert!(validate_stop_loss(entry, stop, OrderSide::Long, atr).is_err());
    }

    #[test]
    fn test_default_stop_calculation() {
        let entry = 100.0;
        let atr = 2.0;
        
        // Long stop
        let stop = calculate_default_stop_loss(entry, OrderSide::Long, atr);
        assert_eq!(stop, 99.8); // 100 - (2 * 0.1)
        
        // Short stop
        let stop = calculate_default_stop_loss(entry, OrderSide::Short, atr);
        assert_eq!(stop, 100.2); // 100 + (2 * 0.1)
    }
}