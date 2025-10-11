use libloading::Library;
use plugin_types::{Plugin, PluginInventory};
use std::any::Any;
use std::collections::{HashMap, hash_map};
use std::error::Error;

pub type PluginCreate = unsafe fn() -> Vec<Plugins>;
pub type PluginResult = Result<(Library, Vec<Plugins>), Box<dyn std::error::Error>>;

pub enum Plugins {
    Base,
    Inventory(InventoryPlugins),
}

impl Plugins {
    pub fn name(&self) -> String {
        match self {
            Plugins::Base => String::from("Base"),
            Plugins::Inventory(inventory) => inventory.name(),
        }
    }
}
pub struct InventoryPlugins {
    plugins: HashMap<String, Box<dyn PluginInventory>>,
}

impl InventoryPlugins {
    pub fn new() -> Self {
        InventoryPlugins {
            plugins: HashMap::new(),
        }
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

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn PluginInventory>> {
        self.plugins.get(name)
    }

    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    // pub fn get_plugins(&self) -> &[Box<dyn PluginInventory>] {
    //     &self.plugins.
    // }
}

impl PluginInventory for InventoryPlugins {
    fn load(&self) {
        // Load inventory plugins from a file or database
    }
}

impl Plugin for InventoryPlugins {
    fn name(&self) -> String {
        String::from("Inventory Plugins")
    }

    fn execute(&self, context: &dyn Any) -> Result<(), Box<dyn Error>> {
        // Or iterate through and execute individual plugins:
        for (name, plugin) in &self.plugins {
            log::info!("Executing inventory plugin: {}", name);
            plugin.execute(context)?;
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
