# Zakaz - Trading Application

## Structure
```
zakaz/
├── src/
│   ├── system/         # Core business logic
│   │   ├── runtime.rs  # Mailbox processor runtime
│   │   ├── mailbox.rs  # Message handler
│   │   ├── state.rs    # App state management
│   │   ├── types.rs    # Message types
│   │   ├── event.rs    # Event pub-sub
│   │   ├── log.rs      # File logging
│   │   ├── macros.rs   # notify_channel macro
│   │   └── ib_handler.rs # IB message handler
│   ├── ib/             # Interactive Brokers integration
│   │   ├── client.rs   # IB client with account switching
│   │   ├── types.rs    # Order templates & trading types
│   │   ├── orders.rs   # Order management & storage
│   │   └── messages.rs # IB-specific messages
│   ├── ui/             # UI layer
│   │   ├── ui_binds.rs # Slint → Runtime
│   │   └── ui_message_handler.rs # Runtime → UI
│   ├── error.rs        # Error types
│   └── main.rs         # Entry point
├── ui/
│   ├── main_window.slint # Main UI
│   └── components/     # UI components
│       ├── z-tabs.slint # Tab component
│       └── z-tabs-bottom.slint # Bottom tabs
├── docs/
│   └── types.md        # Type definitions & examples
└── logs/               # Log files (auto-created)
```

## Architecture
- **Mailbox Pattern**: All state changes go through typed messages
- **Event System**: UI updates via pub-sub
- **Logging**: Custom macros (inf!, err!, wrn!) → timestamped files in logs/
- **Separation**: UI events → Runtime messages → State changes → UI updates
- **IB Integration**: Synchronous ibapi wrapped in async handlers
- **Charting**: Plotters-based candlestick charts with pan/zoom support

## IB Trading Features
- **Order Templates**: Local storage of limit orders with attached stops
- **Account Switching**: Runtime switching between paper/live accounts
- **Order Types**: Always limit orders with GTC stop-loss
- **Order Management**: Activate/deactivate templates without deletion
- **Connection**: TWS paper (7497) and live (7496) ports
- **Historical Data**: Fetch OHLC data for charting (daily/hourly bars)
- **ATR Calculation**: Calculate ATR with outlier filtering

## Key Commands
```bash
cargo build
cargo run
cargo check
```

## Log Macros
- `inf!("message")` - Info logging
- `err!("message")` - Error logging  
- `wrn!("message")` - Warning logging
- `notify_channel!(channel, message)` - Send mailbox response

## Type Definitions
All type definitions, message structures, and usage examples are documented in [docs/types.md](docs/types.md).

## Notes
- TWS or IB Gateway must be running and configured
- API connections must be enabled in TWS/Gateway settings
- Paper account uses port 7497, live uses 7496
- All times are in UTC
- Historical data limited by IB subscription level
- ATR calculation fetches 3x requested period to ensure enough valid bars after filtering
- Charts render at 800x600 by default, customizable in CandlestickChart::new()

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.