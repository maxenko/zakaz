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
│   │   └── macros.rs   # notify_channel macro
│   ├── ui/             # UI layer
│   │   ├── ui_binds.rs # Slint → Runtime
│   │   └── ui_message_handler.rs # Runtime → UI
│   ├── error.rs        # Error types
│   └── main.rs         # Entry point
├── ui/
│   └── main_window.slint # UI definition
└── logs/               # Log files (auto-created)
```

## Dependencies
- **slint** (1.12): UI framework
- **tokio** (1.43): Async runtime
- **mailbox_processor**: Actor-pattern message processing (local)
- **ibapi** (1.2.2): Interactive Brokers TWS API
- **simplelog** (0.12): File + console logging
- **chrono**: Timestamps
- **serde/serde_json**: Serialization
- **thiserror**: Error handling

## Architecture
- **Mailbox Pattern**: All state changes go through typed messages
- **Event System**: UI updates via pub-sub
- **Logging**: Custom macros (inf!, err!, wrn!) → timestamped files in logs/
- **Separation**: UI events → Runtime messages → State changes → UI updates

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