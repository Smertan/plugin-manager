pub mod inventory_a;
use plugin_manager::Plugin;

#[unsafe(no_mangle)]
pub fn create_plugins() -> Vec<Box<dyn Plugin>> {
    let plugins: Vec<Box<dyn Plugin>> = vec![Box::new(inventory_a::InventoryA)];
    plugins
}
