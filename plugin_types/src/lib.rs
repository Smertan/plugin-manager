use libloading::Library;
use serde::Deserialize;
use std::any::Any;
use std::collections::HashMap;

pub type PathString = String;
pub type GroupOrName = String;
pub type PluginResult = Result<(Library, Vec<Box<dyn Plugin>>), Box<dyn std::error::Error>>;
pub type PluginCreate = unsafe fn() -> Vec<Box<dyn Plugin>>;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PluginEntry {
    Individual(PathString),
    Group(HashMap<String, PathString>),
}

/// Information about a loaded plugin, including the plugin itself and its group.
pub struct PluginInfo {
    pub plugin: Box<dyn Plugin>,
    pub group: Option<String>,
}

/// Manages the lifecycle of loaded plugins.
pub struct PluginManager {
    pub plugins: HashMap<String, PluginInfo>,
    // plugin_path: Vec<String>
    pub plugin_path: Vec<HashMap<GroupOrName, PluginEntry>>,
}

pub trait Plugin: Send + Sync + Any {
    /// The `as_any` method allows for dynamic access to methods which
    /// are not covered in the `Plugin` trait.
    fn as_any(&self) -> &dyn Any;

    /// The name of the plugin. This is used to identify the plugin and
    /// to associate it with the context.
    fn name(&self) -> String;

    /// Executes a single function with the provided context.
    ///
    /// If the plugin has other methods, they can be accessed through
    /// the `as_any` method.
    fn execute(&self, context: &dyn Any) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait PluginInventory: Plugin {
    // loads the inventory
    fn load(&self);
}

// #[derive(Debug, Clone)]
pub enum Plugins {
    Base(Box<dyn Plugin>),
    Inventory(Box<dyn PluginInventory>),
}

impl Plugins {
    pub fn name(&self) -> String {
        match self {
            Plugins::Base(base) => base.name(),
            Plugins::Inventory(inventory) => inventory.name(),
        }
    }

    pub fn group_name(&self) -> String {
        match self {
            Plugins::Base(_) => String::from("Base"),
            Plugins::Inventory(_) => String::from("Inventory"),
        }
    }
}
