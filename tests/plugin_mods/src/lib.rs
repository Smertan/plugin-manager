pub mod plugin_a;
pub mod plugin_b;
use plugin_manager::Plugin;

#[unsafe(no_mangle)]
pub fn create_plugins() -> Vec<Box<dyn Plugin>> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
    plugins.push(Box::new(plugin_a::PluginA));
    plugins.push(Box::new(plugin_b::PluginB));
    plugins
}
