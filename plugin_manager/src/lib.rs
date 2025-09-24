use libloading::{Library, Symbol};
use log;
use serde::Deserialize;
use std::any::Any;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::path::Path;

type PathString = String;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub plugins: Option<HashMap<String, PluginEntry>>,
}

/// Information about a plugin entry. This can either be a single plugin
/// or a group of plugins.
#[derive(Deserialize, Debug)]
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

type PluginCreate = unsafe fn() -> Vec<Box<dyn Plugin>>;

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    /// Registers each plugin by the name returned by the plugin's `name` method.
    /// It allows for plugins to be grouped together for easier management within
    /// a single crated if there share similar traits.
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>, group: Option<String>) {
        log::info!("Registering plugin: {:?}", plugin.name());
        let name = plugin.name().to_string();
        let plugin_info = PluginInfo { plugin, group };
        // check if the plugin is already registered
        if self.plugins.contains_key(&name) {
            let msg = format!("Plugin '{}' already registered", name);
            log::error!("{msg}");
            panic!("{msg}");
            // return;
        } else {
            self.plugins.insert(name, plugin_info);
        }
    
    }

    /// Gets the plugin with the given name.
    pub fn get_plugin(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.get(name)
    }

    pub fn get_plugins_by_group(&self, group: &str) -> Vec<&PluginInfo> {
        self.plugins
            .values()
            .filter(|plugin_info| plugin_info.group.as_deref() == Some(group))
            .collect()
    }

    pub fn execute_plugin(
        &self,
        name: &str,
        context: &dyn Any,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(plugin_info) = self.get_plugin(name) {
            plugin_info.plugin.execute(context)
        } else {
            let msg = format!("Plugin '{}' not found", name);
            log::error!("{msg}");
            Err(msg.into())
        }
    }

    /// Utility method to downcast a plugin to a specific type
    ///
    /// It allows you to safely access the plugin's fields and methods,
    /// not found in the `Plugin` trait.
    pub fn with_any<P: 'static>(&self, name: &str) -> Result<&P, Box<dyn std::error::Error>> {
        if let Some(plugin_info) = self.get_plugin(name) {
            match plugin_info.plugin.as_any().downcast_ref::<P>() {
                Some(plugin) => Ok(plugin),
                None => Err(format!("Failed to downcast plugin to type P: {}", name).into()),
            }
        } else {
            Err(format!("Plugin '{}' not found", name).into())
        }
    }

    /// Loops over the plugins and registers them to the plugin manager
    fn register_plugins_vec(&mut self, plugins: Vec<Box<dyn Plugin>>, group: Option<String>) {
        for plugin in plugins {
            self.register_plugin(plugin, group.clone());
        }
    }

    pub fn activate_plugins(mut self) -> Result<PluginManager, Box<dyn std::error::Error>> {
        let meta_data = self.get_plugin_metadata();
        log::debug!("Plugin metadata: {:?}", meta_data);

        if let Some(plugin_config) = meta_data.plugins {
            for (group_or_name, entry) in plugin_config {
                match entry {
                    PluginEntry::Individual(path) => {
                        log::debug!("Loading individual plugin: {group_or_name} {path}");
                        let (library, plugins) = self.load_plugin(&path)?;
                        self.register_plugins_vec(plugins, None);
                        let _library = ManuallyDrop::new(library);
                    }
                    PluginEntry::Group(group_plugins) => {
                        group_plugins.iter().for_each(|(name, path)| {
                            log::debug!("Loading plugin group: {group_or_name}, {name} {path}");
                            let (library, plugins) = self.load_plugin(path).unwrap();
                            self.register_plugins_vec(plugins, Some(group_or_name.clone()));
                            let _library = ManuallyDrop::new(library);
                        });
                    }
                }
            }
            // }
        } else {
            log::error!("No plugin metadata found in manifest");
            return Err("No plugin metadata found in manifest".into());
        }

        Ok(self)
    }

    /// Loads a plugin from a shared object file and registers it to the plugin manager.
    pub fn load_plugin(
        &self,
        filename: &str,
    ) -> Result<(Library, Vec<Box<dyn Plugin>>), Box<dyn std::error::Error>> {
        let path = Path::new(filename);
        if !path.exists() {
            let msg = format!("Plugin file does not exist: {}", filename);
            log::error!("{msg}");
            return Err(msg.into());
        } else {
            log::debug!("Attempting to load plugin: {}", filename);
        }

        let library = unsafe { Library::new(path)? };
        log::debug!("Library loaded successfully");

        let create_plugin: Symbol<PluginCreate> = unsafe { library.get(b"create_plugins")? };
        log::debug!("Found create_plugins symbol");

        let plugins = unsafe { create_plugin() };
        log::debug!("Plugin created successfully");

        Ok((library, plugins))
    }

    /// Retrieves the environment variable CARGO_MANIFEST_PATH containing the
    /// path to  manifest file. The file should contain the plugin metadata
    /// in TOML format which contains the following structure:
    ///
    /// ```toml
    /// [package.metadata.plugins]
    /// plugin_a = "/path/to/plugin_a.so"
    ///
    /// [package.metadata.plugins.inventory]
    /// inventory_plugin = "/path/to/inventory_plugin.so"
    /// ```
    pub fn get_plugin_metadata(&self) -> Metadata {
        let plugin_a_path =
            std::env::var("CARGO_MANIFEST_PATH").unwrap_or_else(|_| ".".to_string());

        let file_string = std::fs::read_to_string(plugin_a_path);
        let manifest = match file_string {
            Ok(manifest) => manifest,
            Err(msg) => {
                eprintln!("Error reading manifest file {}", msg);
                return Metadata { plugins: None };
            }
        };
        let value: toml::Value = toml::from_str(&manifest).unwrap();
        let metadata = if let Some(meta_data) = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.as_table())
        {
            let meta: Result<Metadata, toml::de::Error> =
                toml::from_str(&toml::to_string(meta_data).unwrap());
            meta.unwrap()
        } else {
            Metadata { plugins: None }
        };
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_env_var() {
        unsafe {
            std::env::set_var("CARGO_MANIFEST_PATH", "../tests/plugin_mods/Cargo.toml");
        }
    }

    #[test]
    fn get_plugin_path_test() {
        set_env_var();
        let plugin_manager = PluginManager::new();
        let metadata = plugin_manager.get_plugin_metadata();
        let plugins = metadata.plugins;
        match plugins {
            Some(plug_entry) => {
                for (group, entry) in plug_entry {
                    match entry {
                        PluginEntry::Individual(path) => {
                            assert_eq!(path, "../tests/target/release/libplugin_mods.so");
                        }
                        PluginEntry::Group(path) => {
                            path.iter().for_each(|(metadata_name, path)| {
                                assert_eq!(path, "../tests/target/release/libplugin_inventory.so");
                                assert_eq!(metadata_name, "inventory_a");
                                assert_eq!(group, "inventory");
                            });
                        }
                    }
                }
            }
            None => {
                panic!("No plugins found in metadata");
            }
        }
    }

    #[test]
    fn activate_plugins_test() {
        set_env_var();
        let mut plugin_manager = PluginManager::new();
        plugin_manager = plugin_manager.activate_plugins().unwrap();
        assert!(plugin_manager.get_plugin("plugin_a").is_some());
        assert_eq!(plugin_manager.plugins.len(), 3);
    }

    #[test]
    #[should_panic]
    /// Test for duplicate activation of plugins.
    fn activate_plugins_and_panic_test() {
        set_env_var();
        let mut plugin_manager = PluginManager::new();
        plugin_manager = plugin_manager.activate_plugins().unwrap();
        _ = plugin_manager.activate_plugins().unwrap();
    }

    #[test]
    fn load_plugin_test() {
        let plugin_manager = PluginManager::new();
        let (_library, plugins) = plugin_manager
            .load_plugin("../tests/target/release/libplugin_mods.so")
            .unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name(), "plugin_a");
    }

    #[test]
    fn load_plugin_and_panic_test() {
        let plugin_manager = PluginManager::new();
        let (_library, _) = plugin_manager
            .load_plugin("../tests/target/release/libplugin_mods.so")
            .unwrap();
        let (_library, plugins) = plugin_manager
            .load_plugin("../tests/target/release/libplugin_mods.so")
            .unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name(), "plugin_a");
    }

    #[test]
    fn activate_plugins_with_groups_test() {
        set_env_var();
        let plugin_manager = PluginManager::new().activate_plugins().unwrap();

        // Check individual plugin
        let plugin_a = plugin_manager.get_plugin("plugin_a").unwrap();
        assert_eq!(plugin_a.group, None);

        // Check grouped plugin
        let inventory_plugin = plugin_manager.get_plugin("inventory_a").unwrap();
        assert_eq!(inventory_plugin.group, Some("inventory".to_string()));

        // Get all plugins in the "inventory" group
        let inventory_plugins = plugin_manager.get_plugins_by_group("inventory");
        assert_eq!(inventory_plugins.len(), 1);
        assert_eq!(inventory_plugins[0].plugin.name(), "inventory_a");

        assert_eq!(plugin_manager.plugins.len(), 3);
    }
}
