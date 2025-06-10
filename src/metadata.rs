use serde::{Deserialize, Serialize};
use std::os::raw::c_char;
use std::sync::OnceLock;

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

/// 全局插件元数据存储
/// 用于在插件运行时存储当前插件的元数据信息
static PLUGIN_METADATA: OnceLock<PluginMetadata> = OnceLock::new();

/// 设置全局插件元数据
/// 在插件挂载时由主程序调用
pub fn set_plugin_metadata(metadata: PluginMetadata) -> Result<(), PluginMetadata> {
    PLUGIN_METADATA.set(metadata)
}

/// 获取全局插件元数据
/// 供插件在运行时获取自己的元数据信息
pub fn get_plugin_metadata() -> Option<&'static PluginMetadata> {
    PLUGIN_METADATA.get()
}

/// 清理全局插件元数据
/// 在插件卸载时调用，为下次插件加载做准备
/// 注意：OnceLock 不支持重置，所以这个函数目前是空实现
/// 实际的清理会在插件进程结束时自动完成
pub fn clear_plugin_metadata() {
    // OnceLock 不支持重置，但在插件卸载时进程会结束
    // 所以全局变量会自动清理
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
    }
}
