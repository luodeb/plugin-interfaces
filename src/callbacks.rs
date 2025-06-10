use std::ffi::c_char;
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

/// 主程序提供给插件的回调函数集合
/// 这些函数指针在插件加载时由主程序传递给插件
#[repr(C)]
#[derive(Clone)]
pub struct HostCallbacks {
    /// 向前端发送消息
    pub send_to_frontend: extern "C" fn(*const c_char, *const c_char) -> bool,

    /// 获取应用配置
    pub get_app_config: extern "C" fn(*const c_char) -> *const c_char,

    /// 调用其他插件
    pub call_other_plugin: extern "C" fn(*const c_char, *const c_char) -> *const c_char,
}

impl std::fmt::Debug for HostCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostCallbacks")
            .field("send_to_frontend", &"<function pointer>")
            .field("get_app_config", &"<function pointer>")
            .field("call_other_plugin", &"<function pointer>")
            .finish()
    }
}

/// 实例级别的回调函数存储
/// 每个插件实例都有自己独立的回调函数集合
static INSTANCE_CALLBACKS: OnceLock<Arc<Mutex<HashMap<String, HostCallbacks>>>> = OnceLock::new();

/// 初始化实例回调函数存储
fn init_instance_callbacks() -> &'static Arc<Mutex<HashMap<String, HostCallbacks>>> {
    INSTANCE_CALLBACKS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

/// 设置指定实例的主程序回调函数（由主程序调用）
pub fn set_host_callbacks(instance_id: &str, callbacks: HostCallbacks) -> Result<(), String> {
    let storage = init_instance_callbacks();
    let mut map = storage.lock().map_err(|_| "Failed to lock callbacks storage")?;
    map.insert(instance_id.to_string(), callbacks);
    Ok(())
}

/// 获取指定实例的主程序回调函数（由插件调用）
pub fn get_host_callbacks(instance_id: &str) -> Option<HostCallbacks> {
    let storage = init_instance_callbacks();
    let map = storage.lock().ok()?;
    map.get(instance_id).cloned()
}

/// 清理指定实例的回调函数
/// 在插件卸载时调用
pub fn clear_host_callbacks(instance_id: &str) -> bool {
    let storage = init_instance_callbacks();
    if let Ok(mut map) = storage.lock() {
        map.remove(instance_id).is_some()
    } else {
        false
    }
}
