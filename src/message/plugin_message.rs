use std::time::{SystemTime, UNIX_EPOCH};

use crate::{send_to_frontend, PluginHandler};
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
pub fn send_message_to_frontend(plugin_id: &str, instance_id: &str, content: &str) -> bool {
    let payload = json!({
        "message_type": "plugin_message",
        "plugin_id": plugin_id,
        "instance_id": instance_id,
        "message_id": generate_message_id(),
        "content": content,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    });

    send_to_frontend("plugin-message", &payload.to_string())
}

/// 插件消息发送器
pub trait PluginMessage {
    /// 向前端发送消息（新协议）
    fn send_message_to_frontend(&self, content: &str) -> bool;
}

impl<T: PluginHandler> PluginMessage for T {
    fn send_message_to_frontend(&self, content: &str) -> bool {
        let metadata = self.get_metadata();
        let plugin_id = &metadata.id;
        let instance_id = metadata.instance_id.as_ref().unwrap_or(&metadata.id);
        send_message_to_frontend(plugin_id, instance_id, content)
    }
}
