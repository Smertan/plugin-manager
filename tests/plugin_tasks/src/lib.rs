pub mod task_a;
use plugin_types::Plugin;

#[unsafe(no_mangle)]
pub fn create_plugins() -> Vec<Box<dyn Plugin>> {
    let plugins: Vec<Box<dyn Plugin>> = vec![Box::new(task_a::TaskA)];
    plugins
}
