use std::time::{SystemTime, UNIX_EPOCH};

use crate::PluginHandler;
use serde_json::json;

/// 生成唯一的流ID
fn generate_message_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("message_{}", timestamp)
}

/// 发送消息到前端（新协议）
pub fn send_message_to_frontend(_plugin_id: &str, _instance_id: &str, _content: &str) -> bool {
    let _payload = json!({
        "message_type": "plugin_message",
        "plugin_id": _plugin_id,
        "instance_id": _instance_id,
        "message_id": generate_message_id(),
        "content": _content,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    });

    // TODO: 需要通过上下文传递来发送消息
    false
}

/// 插件消息发送器
/// 提供向前端发送消息的便捷方法，使用上下文传递模式
pub trait PluginMessage {
    /// 向前端发送消息，需要传入插件实例上下文
    fn send_message_to_frontend(&self, content: &str, plugin_ctx: &crate::metadata::PluginInstanceContext) -> bool;
}

impl<T: PluginHandler> PluginMessage for T {
    fn send_message_to_frontend(&self, content: &str, plugin_ctx: &crate::metadata::PluginInstanceContext) -> bool {
        // 使用上下文中的信息发送消息
        let plugin_id = &plugin_ctx.metadata.id;
        let instance_id = plugin_ctx.metadata.instance_id.as_ref().unwrap_or(&plugin_ctx.metadata.id);

        // 构建消息载荷
        let payload = serde_json::json!({
            "message_type": "plugin_message",
            "plugin_id": plugin_id,
            "instance_id": instance_id,
            "message_id": generate_message_id(),
            "content": content,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        }).to_string();

        // 通过上下文发送消息到前端
        plugin_ctx.send_to_frontend("plugin-message", &payload)
    }
}
