# Zakaz Trading Application - Goals and Architecture

## Application Purpose

Zakaz is a streamlined trading interface for Interactive Brokers (IB) that simplifies order placement for rule-based trading strategies. The application bypasses IB's complex interface to provide a focused, efficient workflow for traders following strict trading models.

## Core Concepts

### IB Order States
- **Submitted Orders**: Live orders in the market awaiting execution
- **Inactive Orders**: Created but not yet submitted to the market

### Trading Models
- **Breakout**: Price breaks through resistance/support levels
- **FalseBreakout**: Failed breakout patterns for reversal trades  
- **Bounce**: Price bounces off support/resistance levels
- **Continuation**: Trend continuation patterns

## Key Features

### 1. Order Creation and Sizing
- **Position Sizing Formula**: `ORDER SIZE (SHARES) = RISK PER TRADE / STOP LOSS`
- **Stop Loss Calculation**:
  - Base: 10% of ATR (using enhanced ATR algorithm with outlier filtering)
  - Technical adjustment: User can specify manual stop loss from (0.01 below entry for long, above entry for short) 
    and up to 15% of ATR (i.e., 5% larger than base stop)


### 2. Order Lifecycle Management

#### Template Orders → IB Orders
1. Create order templates locally
2. Submit to IB as either:
   - Active (submitted) orders
   - Inactive (non-submitted) orders
3. Maintain bidirectional association between template and IB order

#### Order States and Collections
- **Order Templates**: Local orders not yet sent to IB
- **Active Orders**: Templates with associated IB orders
- **Order History**: Completed/cancelled orders archive

### 3. Order Actions
- **Submit**: Send template to IB as active order
- **Deactivate**: Convert active IB order to inactive state
- **Cancel**: 
  - Non-executed orders: Complete cancellation
  - Executed positions: Not supported (manual closure required)

### 4. Data Persistence
- **Storage**: SQLite database in application directory
- **Resilience**: Continue operation if historical orders fail to load

### 5. Position Synchronization
On startup:
1. Query existing IB positions
2. Match positions with Active Orders by order ID
3. Create read-only templates for unmatched positions
    - Through a flag indicating read-only position
4. Display all positions with appropriate UI indicators

## Architecture Principles

### Data Flow
```
User Input → Template Order → IB Submission → Active Order → Order History
                    ↓                              ↑
                SQLite DB                     IB Position Query
```

### State Management
- Template orders are the source of truth
- IB orders maintain reference to originating template
- Historical orders preserve full template state
- Positions without templates get synthetic read-only entries

### Error Handling
- Log all errors to application log
- Continue operation on non-critical failures
- Preserve data integrity with transactional updates

## Technical Implementation Notes

### Database Schema
- Templates table: All order templates
- Active orders table: Template ID + IB order ID mapping
- Settings table: Risk parameters, ATR settings

### File Structure
```
zakaz.exe
zakaz.db
logs/
  └── zakaz-YYYY-MM-DD.log
```

### Integration Points
- IB TWS API for order management
- Real-time position updates
- Historical data for ATR calculations
- Market data for current prices