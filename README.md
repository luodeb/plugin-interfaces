# Plugin Interfaces

A comprehensive Rust crate that provides the core interface definitions and abstractions for the chat-client plugin system. This crate serves as the foundation for building dynamic, loadable plugins that can extend the functionality of the chat-client application.

## Overview

The `plugin-interfaces` crate defines a standardized plugin architecture that enables:

- **Dynamic Plugin Loading**: Load plugins at runtime as shared libraries (.dll/.so/.dylib)
- **FFI-Safe Communication**: Cross-language compatibility using C-style interfaces
- **Event-Driven UI**: Immediate-mode UI framework inspired by egui
- **Message Streaming**: Real-time bidirectional communication between plugins and frontend
- **Plugin Lifecycle Management**: Structured initialization, mounting, and cleanup processes

## Architecture

### Core Components

#### 1. Plugin Handler (`handler.rs`)
The main trait that all plugins must implement:

```rust
pub trait PluginHandler {
    fn init_ui(&mut self, ctx: &CreationContext, ui: &mut Ui);
    fn update_ui(&mut self, ctx: &Context, ui: &mut Ui);
    fn on_mount(&mut self, metadata: &PluginMetadata) -> Result<(), Box<dyn std::error::Error>>;
    fn on_dispose(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn on_connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn on_disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn handle_message(&self, message: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn get_metadata(&self) -> PluginMetadata;
}
```

#### 2. FFI Interface (`symbols.rs`)
Provides C-compatible function pointers for cross-language plugin loading:

- `PluginInterface`: FFI-safe struct containing function pointers
- `CreatePluginFn` / `DestroyPluginFn`: Plugin lifecycle management
- Symbol exports: `create_plugin` and `destroy_plugin`

#### 3. Plugin UI Framework (`pluginui/`)
An immediate-mode UI framework that provides:

- **Components**: Text inputs, buttons, combo boxes, labels
- **Context Management**: Creation and runtime contexts
- **Event Handling**: Click events and user interactions
- **Response System**: UI component state management

#### 4. Message System (`message/`)
Comprehensive messaging infrastructure:

- **PluginMessage**: Standard message types (Normal, Success, Warning, Error, Info)
- **StreamMessage**: Real-time streaming with start/data/end lifecycle
- **Frontend Communication**: Bidirectional message passing

#### 5. Host Callbacks (`callbacks.rs`)
Interface for plugins to communicate with the host application:

- `send_to_frontend`: Send messages to the frontend
- `get_app_config`: Access application configuration
- `call_other_plugin`: Inter-plugin communication

#### 6. Configuration (`config.rs`)
Plugin configuration management:

- TOML-based configuration files
- Metadata extraction (id, name, version, author, etc.)
- Runtime configuration loading

#### 7. Metadata (`metadata.rs`)
Plugin metadata structures:

- `PluginMetadata`: Rust-native metadata structure
- `PluginMetadataFFI`: FFI-safe metadata for cross-language compatibility

#### 8. Logging (`logging/`)
Structured logging system for plugins with different log levels and formatting.

#### 9. API (`api.rs`)
High-level API functions for common plugin operations:

- `send_to_frontend`: Simplified frontend communication
- `host_send_to_frontend`: Direct host communication

## Plugin Lifecycle

1. **Loading**: Host loads plugin shared library and calls `create_plugin`
2. **Initialization**: Plugin receives host callbacks via `initialize`
3. **UI Setup**: `init_ui` called to set up initial user interface
4. **Mounting**: `on_mount` called with plugin metadata
5. **Runtime**: 
   - `update_ui` called for UI updates and event handling
   - `handle_message` processes incoming messages
   - `on_connect`/`on_disconnect` handle connection state changes
6. **Cleanup**: `on_dispose` and `destroy` called during shutdown

## Usage Example

```rust
use plugin_interface::*;

struct MyPlugin {
    name: String,
    // ... other fields
}

impl PluginHandler for MyPlugin {
    fn init_ui(&mut self, ctx: &CreationContext, ui: &mut Ui) {
        ui.text_edit_singleline(&mut self.name);
        ui.button("Click me");
    }

    fn update_ui(&mut self, ctx: &Context, ui: &mut Ui) {
        if ui.button("Click me").clicked() {
            self.send_message_to_frontend("Button clicked!");
        }
    }

    // ... implement other required methods
}

// Export plugin creation function
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut PluginInterface {
    let plugin = Box::new(MyPlugin { name: String::new() });
    create_plugin_interface(plugin)
}
```

## Features

- **Type Safety**: Strongly typed interfaces with Rust's type system
- **Memory Safety**: Safe memory management with proper cleanup
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Async Support**: Built-in support for asynchronous operations
- **Streaming**: Real-time data streaming capabilities
- **Configuration**: Flexible TOML-based configuration system
- **Logging**: Comprehensive logging infrastructure
- **UI Framework**: Immediate-mode UI with event handling

## Dependencies

- `serde`: Serialization/deserialization
- `serde_json`: JSON support for message passing
- `uuid`: Unique identifier generation
- `toml`: Configuration file parsing

## Integration

This crate is designed to be used by:

1. **Plugin Developers**: Implement the `PluginHandler` trait to create new plugins
2. **Host Application**: Load and manage plugins using the FFI interface
3. **Frontend**: Receive messages and UI updates from plugins

The plugin system supports both Rust-native plugins and plugins written in other languages that can export C-compatible functions.
