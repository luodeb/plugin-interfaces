
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


/// 流式传输错误类型
#[derive(Debug, Clone)]
pub enum StreamError {
    SendFailed,
    InvalidStreamId,
    StreamNotFound,
    StreamAlreadyEnded,
    InvalidState,
    StreamCancelled,
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::SendFailed => write!(f, "Failed to send message to frontend"),
            StreamError::InvalidStreamId => write!(f, "Invalid stream ID"),
            StreamError::StreamNotFound => write!(f, "Stream not found"),
            StreamError::StreamAlreadyEnded => write!(f, "Stream already ended"),
            StreamError::InvalidState => write!(f, "Invalid stream state"),
            StreamError::StreamCancelled => write!(f, "Stream was cancelled by user"),
        }
    }
}

impl std::error::Error for StreamError {}

/// 流状态
#[derive(Debug, Clone, PartialEq)]
pub enum StreamStatus {
    Active,
    Paused,
    Finalizing,
    Completed,
    Error,
    Cancelled,
}

/// 流信息
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub id: String,
    pub plugin_id: String,
    pub message_type: String,
    pub status: StreamStatus,
    pub created_at: u64,
}

/// 流式消息基础结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessageWrapper {
    pub r#type: String,
    pub plugin_id: String,
    pub instance_id: String,
    pub data: StreamMessageData,
    pub timestamp: u64,
}

/// 流式消息数据联合体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StreamMessageData {
    Start(StreamStartData),
    Data(StreamDataData),
    End(StreamEndData),
    Control(StreamControlData),
}

/// 流开始消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStartData {
    pub stream_id: String,
    pub message_type: String,
}

/// 流数据消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDataData {
    pub stream_id: String,
    pub chunk: String,
    pub is_final: bool,
}

/// 流结束消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEndData {
    pub stream_id: String,
    pub success: bool,
    pub error: Option<String>,
}

/// 流控制消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamControlData {
    pub stream_id: String,
}

/// 全局流管理器（插件端，主要用于API兼容性）
/// 实际的流状态管理在后端进行
pub static STREAM_MANAGER: std::sync::LazyLock<Arc<Mutex<HashMap<String, StreamInfo>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));



/// 插件流式消息发送器
/// 重新设计，移除 plugin_ctx 参数依赖
pub trait PluginStreamMessage {
    /// 开始流式传输，返回流ID
    fn send_message_stream_start(&self) -> Result<String, StreamError>;

    /// 发送流式数据块
    fn send_message_stream(
        &self,
        stream_id: &str,
        chunk: &str,
        is_final: bool,
    ) -> Result<(), StreamError>;

    /// 结束流式传输
    fn send_message_stream_end(
        &self,
        stream_id: &str,
        success: bool,
        error_msg: Option<&str>,
    ) -> Result<(), StreamError>;

    /// 暂停流式传输
    fn send_message_stream_pause(&self, stream_id: &str) -> Result<(), StreamError>;

    /// 恢复流式传输
    fn send_message_stream_resume(&self, stream_id: &str) -> Result<(), StreamError>;

    /// 取消流式传输
    fn send_message_stream_cancel(&self, stream_id: &str) -> Result<(), StreamError>;

    /// 获取流状态
    fn get_stream_status(&self, stream_id: &str) -> Option<StreamStatus>;

    /// 列出活跃的流
    fn list_active_streams(&self) -> Vec<String>;

    /// 批量发送流式数据
    fn send_message_stream_batch(
        &self,
        stream_id: &str,
        chunks: &[&str],
    ) -> Result<(), StreamError>;
}


