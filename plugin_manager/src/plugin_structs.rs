use plugin_types::PluginInventory;
use std::collections::{HashMap, hash_map};

pub struct InventoryPlugins {
    plugins: HashMap<String, Box<dyn PluginInventory>>,
}

impl InventoryPlugins {
    pub fn new() -> Self {
        InventoryPlugins { plugins: HashMap::new() }
    }
    pub fn add_plugin(&mut self, name: String, plugin: Box<dyn PluginInventory>) {
      if let hash_map::Entry::Vacant(entry) = self.plugins.entry(name.clone()) {
            entry.insert(plugin);
        } else {
            let msg = format!("Plugin '{}' already registered", &name);
            log::error!("{msg}");
            panic!("{msg}");
        }
        // self.plugins.push(plugin);
    }
    // pub fn get_plugins(&self) -> &[Box<dyn PluginInventory>] {
    //     &self.plugins.
    // }
}

