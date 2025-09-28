pub mod task_a;
use plugin_manager::Plugin;

#[unsafe(no_mangle)]
pub fn create_plugins() -> Vec<Box<dyn Plugin>> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
    plugins.push(Box::new(task_a::TaskA));
    plugins
}
