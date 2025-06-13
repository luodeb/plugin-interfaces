use crate::log_error;
use crate::message::{
    PluginStreamMessage, StreamControlData, StreamDataData, StreamEndData, StreamError, StreamInfo,
    StreamMessageData, StreamStartData, StreamStatus, STREAM_MANAGER,
};
use serde::{Deserialize, Serialize};
use std::os::raw::c_char;
use std::time::{SystemTime, UNIX_EPOCH};

/// 插件元数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub disabled: bool,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub library_path: Option<String>, // 动态库文件路径
    pub config_path: String,          // 配置文件路径
    pub instance_id: Option<String>,  // 插件实例ID，用于多实例支持
    pub require_history: bool,        // 是否需要接收历史记录
}

/// FFI安全的插件元数据结构
/// 使用C风格的字符串指针
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PluginMetadataFFI {
    pub id: *const c_char,
    pub disabled: bool,
    pub name: *const c_char,
    pub description: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,       // 如果为null表示None
    pub library_path: *const c_char, // 如果为null表示None
    pub config_path: *const c_char,
    pub instance_id: *const c_char, // 如果为null表示None
    pub require_history: bool,      // 是否需要接收历史记录
}

impl PluginMetadata {
    /// 转换为FFI安全的结构
    /// 注意：调用者需要负责释放返回的字符串内存
    pub fn to_ffi(&self) -> PluginMetadataFFI {
        use std::ffi::CString;

        let id = CString::new(self.id.clone()).unwrap().into_raw();
        let name = CString::new(self.name.clone()).unwrap().into_raw();
        let description = CString::new(self.description.clone()).unwrap().into_raw();
        let version = CString::new(self.version.clone()).unwrap().into_raw();
        let config_path = CString::new(self.config_path.clone()).unwrap().into_raw();
        let instance_id = if let Some(ref id) = self.instance_id {
            CString::new(id.clone()).unwrap().into_raw()
        } else {
            std::ptr::null()
        };

        let author = if let Some(ref author) = self.author {
            CString::new(author.clone()).unwrap().into_raw()
        } else {
            std::ptr::null()
        };

        let library_path = if let Some(ref path) = self.library_path {
            CString::new(path.clone()).unwrap().into_raw()
        } else {
            std::ptr::null()
        };

        PluginMetadataFFI {
            id,
            disabled: self.disabled,
            name,
            description,
            version,
            author,
            library_path,
            config_path,
            instance_id,
            require_history: self.require_history,
        }
    }
}

/// 释放FFI元数据结构中的字符串内存
/// 必须在不再使用PluginMetadataFFI时调用
pub unsafe fn free_plugin_metadata_ffi(metadata: PluginMetadataFFI) {
    use std::ffi::CString;

    if !metadata.id.is_null() {
        let _ = CString::from_raw(metadata.id as *mut c_char);
    }
    if !metadata.name.is_null() {
        let _ = CString::from_raw(metadata.name as *mut c_char);
    }
    if !metadata.description.is_null() {
        let _ = CString::from_raw(metadata.description as *mut c_char);
    }
    if !metadata.version.is_null() {
        let _ = CString::from_raw(metadata.version as *mut c_char);
    }
    if !metadata.config_path.is_null() {
        let _ = CString::from_raw(metadata.config_path as *mut c_char);
    }
    if !metadata.author.is_null() {
        let _ = CString::from_raw(metadata.author as *mut c_char);
    }
    if !metadata.library_path.is_null() {
        let _ = CString::from_raw(metadata.library_path as *mut c_char);
    }
}

/// 历史消息结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String, // 'normal' | 'streaming'
    pub status: String, // 'completed' | 'active' | 'paused' | 'error' | 'cancelled'
    pub content: String,
    #[serde(rename = "pluginId")]
    pub plugin_id: String,
    pub role: String, // 'user' | 'plugin' | 'system'
    #[serde(rename = "createdAt")]
    pub created_at: String, // ISO 8601 时间字符串
}

/// 插件实例上下文
/// 包含插件实例的所有状态信息
#[derive(Debug, Clone)]
pub struct PluginInstanceContext {
    pub instance_id: String,
    pub metadata: PluginMetadata,
    pub callbacks: Option<crate::callbacks::HostCallbacks>,
    pub history: Option<Vec<HistoryMessage>>, // 当前会话的历史记录
}

impl PluginInstanceContext {
    /// 创建新的插件实例上下文
    pub fn new(instance_id: String, metadata: PluginMetadata) -> Self {
        Self {
            instance_id,
            metadata,
            callbacks: None,
            history: None,
        }
    }

    /// 设置回调函数
    pub fn set_callbacks(&mut self, callbacks: crate::callbacks::HostCallbacks) {
        self.callbacks = Some(callbacks);
    }

    /// 获取实例ID
    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    /// 获取元数据
    pub fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// 获取回调函数
    pub fn get_callbacks(&self) -> Option<&crate::callbacks::HostCallbacks> {
        self.callbacks.as_ref()
    }

    /// 设置历史记录
    pub fn set_history(&mut self, history: Vec<HistoryMessage>) {
        self.history = Some(history);
    }

    /// 获取历史记录
    pub fn get_history(&self) -> Option<&Vec<HistoryMessage>> {
        // 如果调用了 get_history 方法，但插件没有设置 require_history，
        // 则给出报错提示
        if !self.metadata.require_history {
            log_error!(
                "Plugin '{}' does not set require_history config, but get_history was called.",
                self.metadata.id
            );
        }
        self.history.as_ref()
    }

    /// 清除历史记录
    pub fn clear_history(&mut self) {
        self.history = None;
    }

    /// 向前端发送消息
    pub fn send_to_frontend(&self, event: &str, payload: &str) -> bool {
        if let Some(callbacks) = &self.callbacks {
            use std::ffi::CString;
            if let (Ok(event_str), Ok(payload_str)) = (CString::new(event), CString::new(payload)) {
                return (callbacks.send_to_frontend)(event_str.as_ptr(), payload_str.as_ptr());
            }
        }
        false
    }

    /// 获取应用配置
    pub fn get_app_config(&self, key: &str) -> Option<String> {
        if let Some(callbacks) = &self.callbacks {
            use std::ffi::CString;
            if let Ok(key_str) = CString::new(key) {
                let result_ptr = (callbacks.get_app_config)(key_str.as_ptr());
                if !result_ptr.is_null() {
                    unsafe {
                        let c_str = std::ffi::CStr::from_ptr(result_ptr);
                        return c_str.to_str().ok().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    /// 调用其他插件
    pub fn call_other_plugin(&self, plugin_id: &str, message: &str) -> Option<String> {
        if let Some(callbacks) = &self.callbacks {
            use std::ffi::CString;
            if let (Ok(id_str), Ok(msg_str)) = (CString::new(plugin_id), CString::new(message)) {
                let result_ptr = (callbacks.call_other_plugin)(id_str.as_ptr(), msg_str.as_ptr());
                if !result_ptr.is_null() {
                    unsafe {
                        let c_str = std::ffi::CStr::from_ptr(result_ptr);
                        return c_str.to_str().ok().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    /// 向前端发送消息
    pub fn send_message_to_frontend(&self, content: &str) -> bool {
        // 使用上下文中的信息发送消息
        let plugin_id = &self.metadata.id;
        let instance_id = self
            .metadata
            .instance_id
            .as_ref()
            .unwrap_or(&self.metadata.id);

        // 构建消息载荷
        let payload = serde_json::json!({
            "message_type": "plugin_message",
            "plugin_id": plugin_id,
            "instance_id": instance_id,
            "message_id": self.generate_message_id(),
            "content": content,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        })
        .to_string();

        // 通过上下文发送消息到前端
        self.send_to_frontend("plugin-message", &payload)
    }

    /// 生成唯一的消息ID
    fn generate_message_id(&self) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("message_{}", timestamp)
    }

    /// 刷新UI
    pub fn refresh_ui(&self) -> bool {
        // 使用上下文中的信息发送UI刷新事件
        let plugin_id = &self.metadata.id;
        let instance_id = self
            .metadata
            .instance_id
            .as_ref()
            .unwrap_or(&self.metadata.id);

        // 构建刷新事件的载荷
        let payload = serde_json::json!({
            "plugin": plugin_id,
            "instance": instance_id
        })
        .to_string();

        // 通过上下文发送消息到前端
        self.send_to_frontend("plugin-ui-refreshed", &payload)
    }

    /// 请求前端断开连接
    pub fn call_disconnect(&self) -> bool {
        let plugin_id = &self.metadata.id;
        let instance_id = self
            .metadata
            .instance_id
            .as_ref()
            .unwrap_or(&self.metadata.id);

        // 构建断开连接请求事件的载荷
        let payload = serde_json::json!({
            "plugin_id": plugin_id,
            "instance_id": instance_id,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        })
        .to_string();

        // 通过上下文发送断开连接请求到前端
        self.send_to_frontend("plugin-disconnect-request", &payload)
    }

    /// 生成唯一的流ID
    fn generate_stream_id(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("stream_{}", timestamp)
    }

    /// 发送流式消息到前端
    fn send_stream_message_to_frontend(&self, message_type: &str, data: StreamMessageData) -> bool {
        let plugin_id = &self.metadata.id;
        let instance_id = self
            .metadata
            .instance_id
            .as_ref()
            .unwrap_or(&self.metadata.id);

        let wrapper = crate::message::StreamMessageWrapper {
            r#type: message_type.to_string(),
            plugin_id: plugin_id.clone(),
            instance_id: instance_id.clone(),
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        match serde_json::to_string(&wrapper) {
            Ok(payload) => self.send_to_frontend("plugin-stream", &payload),
            Err(_) => false,
        }
    }
}

/// 将 FFI 元数据转换为 Rust 元数据
pub unsafe fn convert_ffi_to_metadata(metadata_ffi: PluginMetadataFFI) -> PluginMetadata {
    use std::ffi::CStr;

    let id = if !metadata_ffi.id.is_null() {
        CStr::from_ptr(metadata_ffi.id)
            .to_string_lossy()
            .to_string()
    } else {
        String::new()
    };

    let name = if !metadata_ffi.name.is_null() {
        CStr::from_ptr(metadata_ffi.name)
            .to_string_lossy()
            .to_string()
    } else {
        String::new()
    };

    let description = if !metadata_ffi.description.is_null() {
        CStr::from_ptr(metadata_ffi.description)
            .to_string_lossy()
            .to_string()
    } else {
        String::new()
    };

    let version = if !metadata_ffi.version.is_null() {
        CStr::from_ptr(metadata_ffi.version)
            .to_string_lossy()
            .to_string()
    } else {
        String::new()
    };

    let author = if !metadata_ffi.author.is_null() {
        Some(
            CStr::from_ptr(metadata_ffi.author)
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };

    let library_path = if !metadata_ffi.library_path.is_null() {
        Some(
            CStr::from_ptr(metadata_ffi.library_path)
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };

    let config_path = if !metadata_ffi.config_path.is_null() {
        CStr::from_ptr(metadata_ffi.config_path)
            .to_string_lossy()
            .to_string()
    } else {
        String::new()
    };

    let instance_id = if !metadata_ffi.instance_id.is_null() {
        Some(
            CStr::from_ptr(metadata_ffi.instance_id)
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };

    PluginMetadata {
        id,
        disabled: metadata_ffi.disabled,
        name,
        description,
        version,
        author,
        library_path,
        config_path,
        instance_id,
        require_history: metadata_ffi.require_history, // FFI 转换时默认为 false，实际值从配置文件读取
    }
}

/// 为 PluginInstanceContext 实现 PluginStreamMessage trait
impl PluginStreamMessage for PluginInstanceContext {
    fn send_message_stream_start(&self) -> Result<String, StreamError> {
        let stream_id = self.generate_stream_id();
        let plugin_id = &self.metadata.id;

        let data = StreamMessageData::Start(StreamStartData {
            stream_id: stream_id.clone(),
            message_type: "stream_start".to_string(),
        });

        if self.send_stream_message_to_frontend("stream_start", data) {
            // 记录流信息
            if let Ok(mut manager) = STREAM_MANAGER.lock() {
                let stream_info = StreamInfo {
                    id: stream_id.clone(),
                    plugin_id: plugin_id.clone(),
                    message_type: "plugin_stream".to_string(),
                    status: StreamStatus::Active,
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                manager.insert(stream_id.clone(), stream_info);
            }
            Ok(stream_id)
        } else {
            Err(StreamError::SendFailed)
        }
    }

    fn send_message_stream(
        &self,
        stream_id: &str,
        chunk: &str,
        is_final: bool,
    ) -> Result<(), StreamError> {
        // 检查流是否存在
        {
            let manager = STREAM_MANAGER
                .lock()
                .map_err(|_| StreamError::InvalidState)?;
            if !manager.contains_key(stream_id) {
                return Err(StreamError::StreamNotFound);
            }
        }

        let data = StreamMessageData::Data(StreamDataData {
            stream_id: stream_id.to_string(),
            chunk: chunk.to_string(),
            is_final,
        });

        if self.send_stream_message_to_frontend("stream_data", data) {
            // 更新流状态
            if is_final {
                if let Ok(mut manager) = STREAM_MANAGER.lock() {
                    if let Some(stream_info) = manager.get_mut(stream_id) {
                        stream_info.status = StreamStatus::Finalizing;
                    }
                }
            }
            Ok(())
        } else {
            Err(StreamError::StreamCancelled)
        }
    }

    fn send_message_stream_end(
        &self,
        stream_id: &str,
        success: bool,
        error_msg: Option<&str>,
    ) -> Result<(), StreamError> {
        // 检查流是否存在
        {
            let manager = STREAM_MANAGER
                .lock()
                .map_err(|_| StreamError::InvalidState)?;
            if !manager.contains_key(stream_id) {
                return Err(StreamError::StreamNotFound);
            }
        }

        let data = StreamMessageData::End(StreamEndData {
            stream_id: stream_id.to_string(),
            success,
            error: error_msg.map(|s| s.to_string()),
        });

        if self.send_stream_message_to_frontend("stream_end", data) {
            // 更新流状态
            if let Ok(mut manager) = STREAM_MANAGER.lock() {
                if let Some(stream_info) = manager.get_mut(stream_id) {
                    stream_info.status = if success {
                        StreamStatus::Completed
                    } else {
                        StreamStatus::Error
                    };
                }
            }
            Ok(())
        } else {
            Err(StreamError::SendFailed)
        }
    }

    fn send_message_stream_pause(&self, stream_id: &str) -> Result<(), StreamError> {
        let mut manager = STREAM_MANAGER
            .lock()
            .map_err(|_| StreamError::InvalidState)?;
        match manager.get_mut(stream_id) {
            Some(stream_info) => {
                if stream_info.status == StreamStatus::Active {
                    stream_info.status = StreamStatus::Paused;
                    let data = StreamMessageData::Control(StreamControlData {
                        stream_id: stream_id.to_string(),
                    });
                    if self.send_stream_message_to_frontend("stream_pause", data) {
                        Ok(())
                    } else {
                        Err(StreamError::SendFailed)
                    }
                } else {
                    Err(StreamError::InvalidState)
                }
            }
            None => Err(StreamError::StreamNotFound),
        }
    }

    fn send_message_stream_resume(&self, stream_id: &str) -> Result<(), StreamError> {
        let mut manager = STREAM_MANAGER
            .lock()
            .map_err(|_| StreamError::InvalidState)?;
        match manager.get_mut(stream_id) {
            Some(stream_info) => {
                if stream_info.status == StreamStatus::Paused {
                    stream_info.status = StreamStatus::Active;
                    let data = StreamMessageData::Control(StreamControlData {
                        stream_id: stream_id.to_string(),
                    });
                    if self.send_stream_message_to_frontend("stream_resume", data) {
                        Ok(())
                    } else {
                        Err(StreamError::SendFailed)
                    }
                } else {
                    Err(StreamError::InvalidState)
                }
            }
            None => Err(StreamError::StreamNotFound),
        }
    }

    fn send_message_stream_cancel(&self, stream_id: &str) -> Result<(), StreamError> {
        let mut manager = STREAM_MANAGER
            .lock()
            .map_err(|_| StreamError::InvalidState)?;
        match manager.get_mut(stream_id) {
            Some(stream_info) => match stream_info.status {
                StreamStatus::Active | StreamStatus::Paused | StreamStatus::Finalizing => {
                    stream_info.status = StreamStatus::Cancelled;
                    let data = StreamMessageData::Control(StreamControlData {
                        stream_id: stream_id.to_string(),
                    });
                    if self.send_stream_message_to_frontend("stream_cancel", data) {
                        Ok(())
                    } else {
                        Err(StreamError::SendFailed)
                    }
                }
                _ => Err(StreamError::StreamAlreadyEnded),
            },
            None => Err(StreamError::StreamNotFound),
        }
    }

    fn get_stream_status(&self, stream_id: &str) -> Option<StreamStatus> {
        if let Ok(manager) = STREAM_MANAGER.lock() {
            manager.get(stream_id).map(|info| info.status.clone())
        } else {
            None
        }
    }

    fn list_active_streams(&self) -> Vec<String> {
        if let Ok(manager) = STREAM_MANAGER.lock() {
            manager
                .iter()
                .filter(|(_, info)| {
                    matches!(
                        info.status,
                        StreamStatus::Active | StreamStatus::Paused | StreamStatus::Finalizing
                    )
                })
                .map(|(id, _)| id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn send_message_stream_batch(
        &self,
        stream_id: &str,
        chunks: &[&str],
    ) -> Result<(), StreamError> {
        // 检查流是否存在且状态有效
        {
            let manager = STREAM_MANAGER
                .lock()
                .map_err(|_| StreamError::InvalidState)?;
            match manager.get(stream_id) {
                Some(stream_info) => match stream_info.status {
                    StreamStatus::Active | StreamStatus::Finalizing => {}
                    StreamStatus::Paused => return Err(StreamError::InvalidState),
                    StreamStatus::Completed | StreamStatus::Error | StreamStatus::Cancelled => {
                        return Err(StreamError::StreamAlreadyEnded);
                    }
                },
                None => return Err(StreamError::StreamNotFound),
            }
        }

        for (i, chunk) in chunks.iter().enumerate() {
            let is_final = i == chunks.len() - 1;
            let data = StreamMessageData::Data(StreamDataData {
                stream_id: stream_id.to_string(),
                chunk: chunk.to_string(),
                is_final,
            });

            if !self.send_stream_message_to_frontend("stream_data", data) {
                return Err(StreamError::SendFailed);
            }
        }

        // 更新流状态
        if !chunks.is_empty() {
            if let Ok(mut manager) = STREAM_MANAGER.lock() {
                if let Some(stream_info) = manager.get_mut(stream_id) {
                    stream_info.status = StreamStatus::Finalizing;
                }
            }
        }

        Ok(())
    }
}
