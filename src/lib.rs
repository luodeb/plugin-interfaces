// 模块声明
pub mod api;
pub mod callbacks;
pub mod config;
pub mod handler;
pub mod logging;
pub mod message;
pub mod metadata;
pub mod pluginui;
pub mod symbols;

// 重新导出所有公共接口
pub use api::*;
pub use callbacks::*;
pub use config::*;
pub use handler::*;
pub use logging::*;
pub use message::*;
pub use metadata::*;
pub use pluginui::{Context, CreationContext, PluginUiOption, Ui};
pub use symbols::*;

// 导出插件实例上下文
pub use metadata::PluginInstanceContext;
