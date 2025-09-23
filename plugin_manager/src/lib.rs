use libloading::{Library, Symbol};
use log;
use serde::Deserialize;
use std::any::Any;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::path::Path;

pub struct PluginManager {
    pub plugins: HashMap<String, Box<dyn Plugin>>,
}

pub trait Plugin: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn name(&self) -> String;
    fn execute(&self, context: &dyn Any) -> Result<(), Box<dyn std::error::Error>>;
}

type PluginCreate = unsafe fn() -> Vec<Box<dyn Plugin>>;

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        log::debug!("Registering plugin: {:?}", plugin.name());
        // println!("Registering plugin: {:?}", plugin.name());
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }

    pub fn execute_plugin(
        &self,
        name: &str,
        context: &dyn Any,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(plugin) = self.get_plugin(name) {
            plugin.execute(context)
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
        if let Some(plugin) = self.get_plugin(name) {
            match plugin.as_any().downcast_ref::<P>() {
                Some(plugin) => Ok(plugin),
                None => Err(format!("Failed to downcast plugin to type P: {}", name).into()),
            }
        } else {
            Err(format!("Plugin '{}' not found", name).into())
        }
    }

    /// Loops over the plugins and registers them to the plugin manager
    fn register_plugins_vec(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        for plugin in plugins {
            self.register_plugin(plugin);
        }
    }
    pub fn activate_plugins(mut self) -> Result<PluginManager, Box<dyn std::error::Error>> {
        let meta_data = self.get_plugin_metadata();
        log::debug!("Plugin metadata: {:?}", meta_data);

        // Use ManuallyDrop to ensure the library isn't unloaded prematurely
        match meta_data {
            Metadata {
                plugins: Some(path),
            } => {
                for (key, path) in path.as_object().unwrap() {
                    log::debug!("loading plugin: {}", key);
                    if path.is_object() {
                        for (key, path) in path.as_object().unwrap() {
                            log::debug!("loading plugin: {}", key);
                            let (library, plugins) = self.load_plugin(path.as_str().unwrap())?;
                            self.register_plugins_vec(plugins);
                            let _library = ManuallyDrop::new(library);
                        }
                    } else {
                        let (library, plugins) = self.load_plugin(path.as_str().unwrap())?;
                        self.register_plugins_vec(plugins);
                        let _library = ManuallyDrop::new(library);
                    }
                }
            }
            Metadata { plugins: None } => {
                log::error!("No plugin metadata found in manifest");
                return Err(format!("No plugin metadata found in manifest").into());
            }
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
#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub plugins: Option<serde_json::Value>,
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
        let plugins = metadata.plugins.clone().unwrap();
        assert_eq!(
            plugins.get("plugin_a_path").unwrap(),
            "../tests/target/release/libplugin_mods.so"
        );
    }

    // #[test]
    // fn get_plugin_inventory_test() {
    //     set_env_var();
    //     let metadata = get_plugin_path();
    //     let plugins = metadata.plugins.clone().unwrap();
    //     assert_eq!(
    //         plugins
    //             .get("inventory")
    //             .unwrap()
    //             .get("inventory_plugin")
    //             .unwrap(),
    //         "my_inventory_plugin"
    //     );
    // }

    #[test]
    fn activate_plugins_test() {
        set_env_var();
        let mut plugin_manager = PluginManager::new();
        plugin_manager = plugin_manager.activate_plugins().unwrap();
        assert!(plugin_manager.get_plugin("plugin_a").is_some());
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
}
