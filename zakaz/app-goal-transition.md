# Architecture Transition Plan

## Phase 1: Database Layer Foundation
1. **Add SQLite dependency** and create database module
2. **Design schema**:
   - `templates` table: id, name, symbol, side, quantity, limit_price, stop_price, technical_stop, time_in_force, model, status, created_at, updated_at
   - `active_orders` table: template_id, ib_order_id, ib_stop_order_id, submitted_at
   - `settings` table: key, value (for risk_per_trade, atr_settings, etc.)
   - `positions` table: ib_position_id, template_id (nullable), symbol, quantity, avg_cost, is_read_only

3. **Create database access layer** with async SQLite operations

## Phase 2: Refactor Order Types
1. **Update OrderTemplate struct**:
   - Add `technical_stop_price: Option<f64>` field
   - Add `is_read_only: bool` flag
   - Add `risk_per_trade: f64` field
   - Implement `get_stop_loss()` method that returns technical or calculated stop

2. **Create position sizing module**:
   - Implement `calculate_position_size(risk_per_trade, stop_loss) -> shares`
   - Add stop loss validation (0.01 below/above entry to 15% ATR max)

## Phase 3: State Management Refactor
1. **Replace monolithic State with domain-specific managers**:
   - `OrderManager`: Handle template CRUD, IB order association
   - `PositionManager`: Track IB positions, sync on startup
   - `SettingsManager`: Risk parameters, ATR settings

2. **Update message types** to be domain-specific instead of generic RuntimeInMessage

## Phase 4: IB Integration Updates
1. **Enhance IBClient**:
   - Add methods for inactive order submission
   - Implement position querying on startup
   - Add order deactivation (active → inactive conversion)

2. **Create position sync logic**:
   - Query all IB positions on startup
   - Match with active_orders by IB order ID
   - Create read-only templates for unmatched positions

## Phase 5: Migration & Testing
1. **Data migration**: Convert existing in-memory templates to SQLite
2. **Integration testing**: Full order lifecycle testing
3. **Error handling**: Ensure resilience per requirements
4. **Performance optimization**: Async DB operations, caching

This transition maintains app functionality while systematically moving to the new architecture focused on SQLite persistence and proper order lifecycle management.