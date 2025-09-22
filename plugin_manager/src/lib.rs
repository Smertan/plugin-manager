use libloading::{Library, Symbol};
use log;
use serde::Deserialize;
use std::any::Any;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::path::Path;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
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
}

pub fn load_plugin(
    filename: &str,
) -> Result<(Library, Vec<Box<dyn Plugin>>), Box<dyn std::error::Error>> {
    let path = Path::new(filename);
    if !path.exists() {
        let msg = format!("Plugin file does not exist: {}", filename);
        log::debug!("{msg}");
        return Err(msg.into());
    }

    let library = unsafe { Library::new(path)? };
    log::debug!("Library loaded successfully");

    let create_plugin: Symbol<PluginCreate> = unsafe { library.get(b"create_plugins")? };
    log::debug!("Found create_plugin symbol");

    let plugins = unsafe { create_plugin() };
    log::debug!("Plugin created successfully");
    log::debug!("Plugin name: {}", plugins[0].name());

    Ok((library, plugins))
}

pub fn activate_plugins() -> Result<PluginManager, Box<dyn std::error::Error>> {
    let mut plugin_manager = PluginManager::new();
    println!(
        "Current working directory: {}",
        std::env::current_dir()?.display()
    );
    let meta_data = get_plugin_path();
    log::debug!("Plugin metadata: {:?}", meta_data);

    // access all MetaData fields in meta_data and get plugin path
    // println!("Plugin path: {:#?}", plugin_path);
    let plugin_path = "/home/dre/projects/plugin_manager/target/release/libplugin_mods.so";

    // Use ManuallyDrop to ensure the library isn't unloaded prematurely
    match meta_data {
        Metadata {
            plugin_a_path: Some(path),
        } => {
            println!("Plugin A path found: {}", path);
            let (library, plugins) = load_plugin(plugin_path)?;

            for plugin in plugins {
                println!("Registering plugin...");
                let plugin_name = plugin.name();
                plugin_manager.register_plugin(plugin);

                println!("Attempting to execute plugin...");
                plugin_manager.execute_plugin(plugin_name.as_str(), &())?;
                println!("Plugin executed successfully from match statement");
            }
            println!("");
            // _ = library.close();
            let _library = ManuallyDrop::new(library);
        }
        Metadata {
            plugin_a_path: None,
        } => {
            println!("No path found for Plugin A");
            // Handle the case where no path is specified
        }
    }

    Ok(plugin_manager)
}

pub fn get_plugin_path() -> Metadata {
    let plugin_a_path = std::env::var("CARGO_MANIFEST_PATH").unwrap_or_else(|_| ".".to_string());

    let file_string = std::fs::read_to_string(plugin_a_path);
    let manifest = match file_string {
        Ok(manifest) => manifest,
        Err(msg) => {
            eprintln!("Error reading manifest file {}", msg);
            return Metadata {
                plugin_a_path: None,
            };
        }
    };
    let value: toml::Value = toml::from_str(&manifest).unwrap();

    let meta_data = value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.as_table());
    println!("meta_data: {:?}", meta_data);
    match meta_data {
        Some(meta_data) => {
            let meta: Result<Metadata, toml::de::Error> =
                toml::from_str(&toml::to_string(meta_data).unwrap());
            println!("cargo:rustc-env=PLUGIN_A_PATH={:?}", meta.unwrap());
        }
        None => {
            println!("cargo:rustc-env=PLUGIN_A_PATH=not_found");
        }
    }
    let metadata = if let Some(meta_data) = value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.as_table())
    {
        let meta: Result<Metadata, toml::de::Error> =
            toml::from_str(&toml::to_string(meta_data).unwrap());
        meta.unwrap()
    } else {
        Metadata {
            plugin_a_path: None,
        }
    };
    metadata
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub plugin_a_path: Option<String>,
    // plugins: Option<Plugins>,
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
        let metadata = get_plugin_path();
        let plugin_path = metadata.plugin_a_path.clone().unwrap();
        assert_eq!(
            plugin_path,
            "/home/dre/projects/plugin_manager/target/release/libplugin_mods.so"
        );
    }

    #[test]
    fn activate_plugins_test() {
        set_env_var();
        let plugin_manager = activate_plugins().unwrap();
        assert!(plugin_manager.get_plugin("plugin_a").is_some());
    }

    #[test]
    fn load_plugin_test() {
        let (_library, plugins) =
            load_plugin("/home/dre/projects/plugin_manager/target/release/libplugin_mods.so")
                .unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name(), "plugin_a");
    }
}
