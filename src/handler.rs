use crate::callbacks::HostCallbacks;
use crate::metadata::{PluginInstanceContext, PluginMetadata};
use crate::pluginui::{Context, Ui};

/// 插件处理器 trait
/// 定义了插件的生命周期方法，使用上下文传递模式
pub trait PluginHandler: Send + Sync {
    /// 插件初始化时调用（在挂载之前，用于创建插件上下文）
    /// 返回插件实例上下文，包含所有实例相关的状态
    fn initialize(
        &mut self,
        callbacks: HostCallbacks,
        metadata: PluginMetadata,
    ) -> Result<PluginInstanceContext, Box<dyn std::error::Error>> {
        // 创建插件实例上下文
        let instance_id = metadata
            .instance_id
            .as_ref()
            .ok_or("Instance ID is required for plugin initialization")?
            .clone();

        let mut context = PluginInstanceContext::new(instance_id, metadata);
        context.set_callbacks(callbacks);

        Ok(context)
    }

    /// 更新UI（事件驱动）
    /// 当前端用户交互或需要更新UI时调用
    fn update_ui(&mut self, ctx: &Context, ui: &mut Ui, plugin_ctx: &PluginInstanceContext);

    /// 插件挂载时调用
    fn on_mount(
        &mut self,
        plugin_ctx: &PluginInstanceContext,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// 插件卸载时调用
    fn on_dispose(
        &mut self,
        plugin_ctx: &PluginInstanceContext,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// 连接时调用
    fn on_connect(
        &mut self,
        plugin_ctx: &PluginInstanceContext,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// 断开连接时调用
    fn on_disconnect(
        &mut self,
        plugin_ctx: &PluginInstanceContext,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// 处理消息
    fn handle_message(
        &mut self,
        message: &str,
        plugin_ctx: &PluginInstanceContext,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// 获取插件元数据
    fn get_metadata<'a>(&self, plugin_ctx: &'a PluginInstanceContext) -> &'a PluginMetadata {
        plugin_ctx.get_metadata()
    }
}
