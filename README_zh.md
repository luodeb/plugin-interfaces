# Plugin Interfaces (插件接口)

一个全面的 Rust crate，为聊天客户端插件系统提供核心接口定义和抽象。此 crate 作为构建可扩展聊天客户端应用程序功能的动态、可加载插件的基础。

## 概述 (Overview)

`plugin-interfaces` crate 定义了一个标准化的插件架构，支持：

*   **动态插件加载 (Dynamic Plugin Loading)**: 在运行时作为共享库 (.dll/.so/.dylib) 加载插件
*   **FFI 安全通信 (FFI-Safe Communication)**: 使用 C 风格接口实现跨语言兼容性
*   **事件驱动 UI (Event-Driven UI)**: 受 egui 启发的即时模式 UI 框架
*   **消息流式传输 (Message Streaming)**: 插件与前端之间的实时双向通信
*   **插件生命周期管理 (Plugin Lifecycle Management)**: 结构化的初始化、挂载和清理过程

## 架构 (Architecture)

### 核心组件 (Core Components)

#### 1. 插件处理器 (`handler.rs`)
所有插件必须实现的主要 trait：

```rust
pub trait PluginHandler {
    fn init_ui(&mut self, ctx: &CreationContext, ui: &mut Ui); // 初始化 UI
    fn update_ui(&mut self, ctx: &Context, ui: &mut Ui); // 更新 UI
    fn on_mount(&mut self, metadata: &PluginMetadata) -> Result<(), Box<dyn std::error::Error>>; // 挂载时
    fn on_dispose(&mut self) -> Result<(), Box<dyn std::error::Error>>; // 销毁时
    fn on_connect(&mut self) -> Result<(), Box<dyn std::error::Error>>; // 连接时
    fn on_disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>>; // 断开连接时
    fn handle_message(&self, message: &str) -> Result<String, Box<dyn std::error::Error>>; // 处理消息
    fn get_metadata(&self) -> PluginMetadata; // 获取元数据
}
```

#### 2. FFI 接口 (`symbols.rs`)
提供用于跨语言插件加载的 C 兼容函数指针：

*   `PluginInterface`: 包含函数指针的 FFI 安全结构体
*   `CreatePluginFn` / `DestroyPluginFn`: 插件生命周期管理
*   符号导出: `create_plugin` 和 `destroy_plugin`

#### 3. 插件 UI 框架 (`pluginui/`)
一个即时模式 UI 框架，提供：

*   **组件 (Components)**: 文本输入框、按钮、组合框、标签
*   **上下文管理 (Context Management)**: 创建和运行时上下文
*   **事件处理 (Event Handling)**: 点击事件和用户交互
*   **响应系统 (Response System)**: UI 组件状态管理

#### 4. 消息系统 (`message/`)
全面的消息基础设施：

*   **PluginMessage**: 标准消息类型 (普通、成功、警告、错误、信息)
*   **StreamMessage**: 具有开始/数据/结束生命周期的实时流式传输
*   **前端通信 (Frontend Communication)**: 双向消息传递

#### 5. 宿主回调 (`callbacks.rs`)
插件与宿主应用程序通信的接口：

*   `send_to_frontend`: 发送消息到前端
*   `get_app_config`: 访问应用程序配置
*   `call_other_plugin`: 插件间通信

#### 6. 配置 (`config.rs`)
插件配置管理：

*   基于 TOML 的配置文件
*   元数据提取 (id、名称、版本、作者等)
*   运行时配置加载

#### 7. 元数据 (`metadata.rs`)
插件元数据结构：

*   `PluginMetadata`: Rust 原生元数据结构
*   `PluginMetadataFFI`: 用于跨语言兼容性的 FFI 安全元数据

#### 8. 日志 (`logging/`)
为插件设计的结构化日志系统，具有不同的日志级别和格式化。

#### 9. API (`api.rs`)
用于常见插件操作的高级 API 函数：

*   `send_to_frontend`: 简化的前端通信
*   `host_send_to_frontend`: 直接的宿主通信

## 插件生命周期 (Plugin Lifecycle)

1.  **加载 (Loading)**: 宿主加载插件共享库并调用 `create_plugin`
2.  **初始化 (Initialization)**: 插件通过 `initialize` 接收宿主回调
3.  **UI 设置 (UI Setup)**: 调用 `init_ui` 以设置初始用户界面
4.  **挂载 (Mounting)**: 使用插件元数据调用 `on_mount`
5.  **运行时 (Runtime)**:
    *   调用 `update_ui` 进行 UI 更新和事件处理
    *   `handle_message` 处理传入的消息
    *   `on_connect`/`on_disconnect` 处理连接状态变化
6.  **清理 (Cleanup)**: 在关闭期间调用 `on_dispose` 和 `destroy`

## 使用示例 (Usage Example)

```rust
use plugin_interface::*; // 使用插件接口

struct MyPlugin {
    name: String,
    // ... 其他字段
}

impl PluginHandler for MyPlugin {
    fn init_ui(&mut self, ctx: &CreationContext, ui: &mut Ui) {
        ui.text_edit_singleline(&mut self.name); // 单行文本编辑
        ui.button("Click me"); // 按钮
    }

    fn update_ui(&mut self, ctx: &Context, ui: &mut Ui) {
        if ui.button("Click me").clicked() { // 按钮被点击
            self.send_message_to_frontend("Button clicked!"); // 发送消息到前端
        }
    }

    // ... 实现其他必需的方法
}

// 导出插件创建函数
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut PluginInterface {
    let plugin = Box::new(MyPlugin { name: String::new() }); // 创建插件实例
    create_plugin_interface(plugin) // 创建插件接口
}
```

## 特性 (Features)

*   **类型安全 (Type Safety)**: 利用 Rust 类型系统的强类型接口
*   **内存安全 (Memory Safety)**: 带有适当清理的安全内存管理
*   **跨平台 (Cross-Platform)**: 在 Windows, macOS 和 Linux 上工作
*   **异步支持 (Async Support)**: 内置支持异步操作
*   **流式传输 (Streaming)**: 实时数据流传输能力
*   **配置 (Configuration)**: 灵活的基于 TOML 的配置系统
*   **日志 (Logging)**: 全面的日志基础设施
*   **UI 框架 (UI Framework)**: 带有事件处理的即时模式 UI

## 依赖项 (Dependencies)

*   `serde`: 序列化/反序列化
*   `serde_json`: 用于消息传递的 JSON 支持
*   `uuid`: 唯一标识符生成
*   `toml`: 配置文件解析

## 集成 (Integration)

此 crate 设计用于：

1.  **插件开发者 (Plugin Developers)**: 实现 `PluginHandler` trait 来创建新插件
2.  **宿主应用程序 (Host Application)**: 使用 FFI 接口加载和管理插件
3.  **前端 (Frontend)**: 接收来自插件的消息和 UI 更新

该插件系统同时支持 Rust 原生插件和能够导出 C 兼容函数的其他语言编写的插件。
