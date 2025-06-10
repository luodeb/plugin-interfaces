use crate::callbacks::HostCallbacks;
use crate::metadata::{PluginMetadataFFI, PluginInstanceContext};
use std::os::raw::c_char;

/// 插件包装器，包含处理器和上下文
pub struct PluginWrapper {
    pub handler: Box<dyn crate::handler::PluginHandler>,
    pub context: Option<PluginInstanceContext>,
}

/// FFI安全的插件接口
/// 使用C风格的函数指针而不是trait对象
#[repr(C)]
pub struct PluginInterface {
    pub plugin_ptr: *mut std::ffi::c_void,
    pub initialize:
        unsafe extern "C" fn(*mut std::ffi::c_void, HostCallbacks, PluginMetadataFFI) -> i32,
    pub update_ui: unsafe extern "C" fn(
        *mut std::ffi::c_void,
        *const std::ffi::c_void,
        *mut std::ffi::c_void,
    ) -> i32,
    pub on_mount: unsafe extern "C" fn(*mut std::ffi::c_void) -> i32,
    pub on_dispose: unsafe extern "C" fn(*mut std::ffi::c_void) -> i32,
    pub on_connect: unsafe extern "C" fn(*mut std::ffi::c_void) -> i32,
    pub on_disconnect: unsafe extern "C" fn(*mut std::ffi::c_void) -> i32,
    pub handle_message:
        unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *mut *mut c_char) -> i32,
    pub get_metadata: unsafe extern "C" fn(*mut std::ffi::c_void) -> PluginMetadataFFI,
    pub destroy: unsafe extern "C" fn(*mut std::ffi::c_void),
}

/// 插件创建函数类型
/// 返回FFI安全的插件接口
pub type CreatePluginFn = unsafe extern "C" fn() -> *mut PluginInterface;

/// 插件销毁函数类型
/// 销毁插件接口
pub type DestroyPluginFn = unsafe extern "C" fn(*mut PluginInterface);

/// 插件导出符号名称
pub const CREATE_PLUGIN_SYMBOL: &[u8] = b"create_plugin";
pub const DESTROY_PLUGIN_SYMBOL: &[u8] = b"destroy_plugin";

/// 从PluginHandler trait对象创建FFI安全的插件接口
/// 这个函数帮助插件开发者将trait对象转换为FFI安全的接口
pub fn create_plugin_interface_from_handler(
    handler: Box<dyn crate::handler::PluginHandler>,
) -> *mut PluginInterface {
    use std::ffi::{CStr, CString};

    let wrapper = PluginWrapper {
        handler,
        context: None,
    };
    let wrapper_ptr = Box::into_raw(Box::new(wrapper)) as *mut std::ffi::c_void;

    // 定义FFI安全的函数包装器
    unsafe extern "C" fn initialize_wrapper(
        ptr: *mut std::ffi::c_void,
        callbacks: HostCallbacks,
        metadata_ffi: PluginMetadataFFI,
    ) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);

        // 将 FFI 元数据转换为 Rust 元数据
        let metadata = crate::metadata::convert_ffi_to_metadata(metadata_ffi);

        match wrapper.handler.initialize(callbacks, metadata) {
            Ok(context) => {
                wrapper.context = Some(context);
                0
            }
            Err(_) => -1,
        }
    }

    unsafe extern "C" fn update_ui_wrapper(
        ptr: *mut std::ffi::c_void,
        ctx_ptr: *const std::ffi::c_void,
        ui_ptr: *mut std::ffi::c_void,
    ) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);
        let ctx = &*(ctx_ptr as *const crate::pluginui::Context);
        let ui = &mut *(ui_ptr as *mut crate::pluginui::Ui);

        if let Some(plugin_context) = &wrapper.context {
            wrapper.handler.update_ui(ctx, ui, plugin_context);
        }
        0
    }

    unsafe extern "C" fn on_mount_wrapper(ptr: *mut std::ffi::c_void) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);

        if let Some(plugin_context) = &wrapper.context {
            match wrapper.handler.on_mount(plugin_context) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }

    unsafe extern "C" fn on_dispose_wrapper(ptr: *mut std::ffi::c_void) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);
        if let Some(plugin_context) = &wrapper.context {
            match wrapper.handler.on_dispose(plugin_context) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }

    unsafe extern "C" fn on_connect_wrapper(ptr: *mut std::ffi::c_void) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);
        if let Some(plugin_context) = &wrapper.context {
            match wrapper.handler.on_connect(plugin_context) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }

    unsafe extern "C" fn on_disconnect_wrapper(ptr: *mut std::ffi::c_void) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);
        if let Some(plugin_context) = &wrapper.context {
            match wrapper.handler.on_disconnect(plugin_context) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }

    unsafe extern "C" fn handle_message_wrapper(
        ptr: *mut std::ffi::c_void,
        message: *const c_char,
        result: *mut *mut c_char,
    ) -> i32 {
        let wrapper = &mut *(ptr as *mut PluginWrapper);
        let message_str = CStr::from_ptr(message).to_string_lossy();

        if let Some(plugin_context) = &wrapper.context {
            match wrapper.handler.handle_message(&message_str, plugin_context) {
                Ok(response) => {
                    let response_cstring = CString::new(response).unwrap();
                    *result = response_cstring.into_raw();
                    0
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    }

    unsafe extern "C" fn get_metadata_wrapper(ptr: *mut std::ffi::c_void) -> PluginMetadataFFI {
        let wrapper = &*(ptr as *mut PluginWrapper);
        if let Some(plugin_context) = &wrapper.context {
            let metadata = wrapper.handler.get_metadata(plugin_context);
            metadata.to_ffi()
        } else {
            // 返回一个默认的空元数据
            PluginMetadataFFI {
                id: std::ptr::null(),
                disabled: false,
                name: std::ptr::null(),
                description: std::ptr::null(),
                version: std::ptr::null(),
                author: std::ptr::null(),
                library_path: std::ptr::null(),
                config_path: std::ptr::null(),
                instance_id: std::ptr::null(),
            }
        }
    }

    unsafe extern "C" fn destroy_wrapper(ptr: *mut std::ffi::c_void) {
        let _ = Box::from_raw(ptr as *mut PluginWrapper);
    }

    let interface = PluginInterface {
        plugin_ptr: wrapper_ptr,
        initialize: initialize_wrapper,
        update_ui: update_ui_wrapper,
        on_mount: on_mount_wrapper,
        on_dispose: on_dispose_wrapper,
        on_connect: on_connect_wrapper,
        on_disconnect: on_disconnect_wrapper,
        handle_message: handle_message_wrapper,
        get_metadata: get_metadata_wrapper,
        destroy: destroy_wrapper,
    };

    Box::into_raw(Box::new(interface))
}
