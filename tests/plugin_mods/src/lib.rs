pub mod plugin_a;
pub mod plugin_b;
use plugin_types::Plugin;

#[unsafe(no_mangle)]
pub fn create_plugins() -> Vec<Box<dyn Plugin>> {
    let plugins: Vec<Box<dyn Plugin>> =
        vec![Box::new(plugin_a::PluginA), Box::new(plugin_b::PluginB)];
    plugins
}
