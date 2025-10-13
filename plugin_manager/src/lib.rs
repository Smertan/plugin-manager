//! # Plugin Manager
//!
//! A flexible and easy-to-use plugin management system for Rust applications.
//!
//! This module provides a `PluginManager` that allows dynamic loading, registration,
//! and management of plugins at runtime. It supports individual plugins and grouped plugins,
//! making it suitable for various application architectures.
//!
//! ## Features
//!
//! - Dynamic loading of plugins from shared object files (.so)
//! - Support for individual and grouped plugins
//! - Plugin registration and deregistration
//! - Execution of plugin functionality
//! - Metadata-driven plugin configuration
//!
//!
//! ## Creating Plugins
//!
//! To create a plugin, implement the `Plugin` trait and export a `create_plugins` function:
//!
//! The `as_any` method is required to allow access to the methods not
//! mentioned in the `Plugin` trait, and needs to be set up to return self.
//!
//! ```rust
//! use plugin_types::Plugin;
//! use std::any::Any;
//!
//! #[derive(Debug)]
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> String {
//!         "my_plugin".to_string()
//!     }
//!
//!     fn execute(&self, _context: &dyn Any) -> Result<(), Box<dyn std::error::Error>> {
//!         println!("Executing MyPlugin");
//!         Ok(())
//!     }
//!
//!     fn as_any(&self) -> &dyn Any {
//!         self
//!     }
//! }
//!
//! #[unsafe(no_mangle)]
//! pub fn create_plugins() -> Vec<Box<dyn Plugin>> {
//!     vec![Box::new(MyPlugin)]
//! }
//! ```
//!
//! ## Setting up Cargo.toml for Plugins
//!
//! When creating a plugin, you need to set up your `Cargo.toml` file correctly:
//!
//! 1. Add the `plugin_manager` as a dependency:
//!
//! ```toml
//! [dependencies]
//! plugin_manager = "0.1.0"
//! ```
//!
//! 2. Configure the library to be both a Rust library and a dynamic library:
//!
//! ```toml
//! [lib]
//! name = "your_plugin_name"
//! crate-type = ["lib", "cdylib"]
//! ```
//!
//! This configuration allows your plugin to be compiled as both a Rust library
//! and a dynamic library, which is necessary for the PluginManager to load it at runtime.
//!
//! ## Building the Plugin
//!
//! To build your plugin for use with the main project:
//!
//! 1. Navigate to your plugin's directory.
//! 2. Run the following command to build the plugin as a dynamic library:
//!
//!    ```bash
//!    cargo build --release
//!    ```
//!
//! 3. The compiled dynamic library will be in the `target/release` directory with a name like
//!    `libyour_plugin_name.so` (on Linux), `libyour_plugin_name.dylib` (on macOS),
//!    or `your_plugin_name.dll` (on Windows).
//!
//! ## Differences between Cargo.toml Files
//!
//! Both the main project using plugins and the individual plugin projects are end users of the plugin_manager.
//!
//! 1. Main Project Cargo.toml:
//!    - Located in the root of the project that will use plugins.
//!    - Includes `plugin_manager` as a dependency.
//!    - Does not need the `crate-type` specification.
//!    - Does not contain any metadata for plugin configuration.
//!    - The loaded plugins are dependant on the plugins specified in the `End-User's` project Cargo.toml.
//!
//!    Example:
//!    ```toml
//!    [package]
//!    name = "main_project"
//!    version = "0.1.0"
//!    edition = "2024"
//!
//!    [dependencies]
//!    plugin_manager = "0.1.0"
//!    ```
//!
//! 2. Plugin Project Cargo.toml:
//!    - Located in a separate project directory for each plugin.
//!    - Includes `plugin_manager` as a dependency.
//!    - Specifies `crate-type = ["lib", "cdylib"]` to build as both a Rust library and a dynamic library.
//!    - Does not contain plugin metadata configuration.
//!
//!    Example:
//!    ```toml
//!    [package]
//!    name = "my_plugin"
//!    version = "0.1.0"
//!    edition = "2024"
//!
//!    [dependencies]
//!    plugin_manager = "0.1.0"
//!
//!    [lib]
//!    name = "my_plugin"
//!    crate-type = ["lib", "cdylib"]
//!    ```
//!
//! 3. End-User Project Cargo.toml:
//!    - Includes the main project as dependencies.
//!    - Contains metadata for plugin configuration.
//!
//!    Example:
//!    ```toml
//!    [package]
//!    name = "my_application"
//!    version = "0.1.0"
//!    edition = "2024"
//!
//!    [dependencies]
//!    main_project = "0.1.0"
//!
//!    [package.metadata.plugins]
//!    my_plugin = "/path/to/libmy_plugin.so"
//!    ```
//!
//! The main differences between these Cargo.toml files are:
//!
//! 1. The Main Project Cargo.toml sets up the core project that will use plugins:
//!    - It includes the plugin_manager as a dependency.
//!    - It doesn't specify crate-type or contain plugin metadata.
//!    - The plugins it can load are determined by the End-User's project configuration.
//!
//! 2. The Plugin Project Cargo.toml configures individual plugin projects:
//!    - It includes the plugin_manager as a dependency.
//!    - It specifies crate-type as both "lib" and "cdylib" to produce a dynamic library.
//!    - It doesn't contain any plugin metadata configuration.
//!
//! 3. The End-User Project Cargo.toml configures the application that will use the main project and its plugins:
//!    - It includes the main project (not the plugin_manager directly) as a dependency.
//!    - It contains the metadata for plugin configuration, specifying which plugins to load and how to group them.
//!
//! ## Plugin Configuration
//!
//! Plugins are configured in the `Cargo.toml` file of the end-user project:
//!
//! ```toml
//! [package.metadata.plugins]
//! plugin_a = "/path/to/plugin_a.so"
//!
//! [package.metadata.plugins.group_name]
//! plugin_b = "/path/to/plugin_b.so"
//! plugin_c = "/path/to/plugin_c.so"
//! ```
//!
//! ## Usage
//!
//! Here's a basic example of how to use the `PluginManager`:
//!
//! ```rust
//! # unsafe {
//! #     std::env::set_var("CARGO_MANIFEST_PATH", "../tests/plugin_mods/Cargo.toml");
//! # }
//! use plugin_manager::PluginManager;
//!
//! # fn doc_test() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new PluginManager
//! let mut plugin_manager = PluginManager::new();
//!
//! // Activate plugins based on metadata in Cargo.toml
//! plugin_manager = plugin_manager.activate_plugins()?;
//!
//! // Execute a specific plugin
//! plugin_manager.execute_plugin("plugin_a", &())?;
//!
//! // Deregister a plugin
//! let deregistered = plugin_manager.deregister_plugin("plugin_b");
//! print!("Deregistered plugin: {:?}", deregistered);
//!
//! // Deregister all plugins
//! let deregistered = plugin_manager.deregister_all_plugins();
//! println!("Deregistered plugins: {:?}", deregistered);
//!
//! #    Ok(())
//! # }
//! ```
//!
//!
//! This module provides a robust foundation for building plugin-based architectures
//! in Rust applications, offering flexibility and ease of use.

pub mod plugin_structs;

use libloading::{Library, Symbol};
use plugin_structs::{PluginCreate as PluginCreateNew, PluginResult as PluginResultNew};
use plugin_types::{
    GroupOrName, Plugin, PluginCreate, PluginEntry, PluginInfo, PluginInventory, PluginResult,
    Plugins,
};
use serde::Deserialize;
use std::any::Any;
use std::collections::{HashMap, hash_map};
use std::io::{Error, ErrorKind};
use std::mem::ManuallyDrop;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub plugins: Option<HashMap<GroupOrName, PluginEntry>>,
}

pub struct PluginManagerNew {
    pub plugins: HashMap<String, Plugins>,
    plugin_path: Vec<HashMap<GroupOrName, PluginEntry>>,
    libraries: Vec<libloading::Library>, // Add this to keep libraries alive
}

impl Default for PluginManagerNew {
    fn default() -> Self {
        Self::new()
    }
}
macro_rules! get_plugins_by_variant {
    ($self:expr, $variant:path, $trait_type:ty) => {
        $self
            .plugins
            .iter()
            .filter_map(|(name, plugin)| match plugin {
                $variant(inner) => Some((name, inner as $trait_type)),
                _ => None,
            })
            .collect()
    };
}

impl PluginManagerNew {
    pub fn new() -> Self {
        PluginManagerNew {
            plugins: HashMap::new(),
            plugin_path: Vec::new(),
            libraries: Vec::new(),
        }
    }

    pub fn activate_plugins(mut self) -> Result<PluginManagerNew, Box<dyn std::error::Error>> {
        let meta_data = self.get_plugin_metadata();
        log::debug!("Plugin metadata: {:?}", meta_data);
        let mut registrations = Vec::new();
        if let Some(plugin_config) = meta_data.plugins {
            for (group_or_name, plugin_entry) in plugin_config {
                registrations.push((group_or_name, plugin_entry));
            }
        } else {
            log::error!("No plugin metadata found in manifest");
            return Err("No plugin metadata found in manifest".into());
        }
        if !self.plugin_path.is_empty() {
            for entry in &self.plugin_path {
                for (group_or_name, plugin_entry) in entry {
                    registrations.push((group_or_name.clone(), plugin_entry.clone()));
                }
            }
        }
        for (group_or_name, plugin_entry) in registrations {
            self.activation_registration(group_or_name.clone(), &plugin_entry)?;
        }
        Ok(self)
    }

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
        // let metadata = if let Some(meta_data) = value
        if let Some(meta_data) = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.as_table())
        {
            let meta: Result<Metadata, toml::de::Error> =
                toml::from_str(&toml::to_string(meta_data).unwrap());
            meta.unwrap()
        } else {
            Metadata { plugins: None }
        }
        // metadata
    }

    fn activation_registration(
        &mut self,
        group_or_name: String,
        plugin_entry: &PluginEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match plugin_entry {
            PluginEntry::Individual(path) => {
                log::debug!("Loading individual plugin: {group_or_name} {path}");
                let (library, plugins) = self.load_plugin(path)?;
                self.libraries.push(library);
                for plugin in plugins {
                    self.register_plugin(plugin);
                }
            }
            PluginEntry::Group(group_plugins) => {
                group_plugins.iter().for_each(|(name, path)| {
                    log::debug!("Loading plugin group: {group_or_name}, {name} {path}");
                    let (library, plugins) = self.load_plugin(path).unwrap();
                    self.libraries.push(library);
                    for plugin in plugins {
                        self.register_plugin(plugin);
                    }
                });
            }
        }
        Ok(())
    }

    pub fn load_plugin(&self, filename: &str) -> PluginResultNew {
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

        let create_plugin: Symbol<PluginCreateNew> = unsafe { library.get(b"create_plugins")? };
        log::debug!("Found create_plugins symbol");

        let plugins = unsafe { create_plugin() };
        log::debug!("Plugin created successfully");

        Ok((library, plugins))
    }

    pub fn register_plugin(&mut self, plugin: Plugins) {
        let name = plugin.name();
        log::info!("Registering plugin: {:?}", name);

        println!("Registering plugin: {}", name);
        if let hash_map::Entry::Vacant(entry) = self.plugins.entry(name.clone()) {
            entry.insert(plugin);
        } else {
            let msg = format!("Plugin '{}' already registered", &name);
            log::error!("{msg}");
            panic!("{msg}");
        }
    }

    /// Gets a plugin as a trait object based on its type
    pub fn get_plugin(&self, name: &str) -> Option<&Plugins> {
        self.plugins.get(name)
    }

    /// Gets an inventory plugin, returns None if the plugin is not a Base variant
    #[allow(clippy::borrowed_box)]
    pub fn get_base_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name).and_then(|plugin| match plugin {
            Plugins::Base(base) => Some(base),
            _ => None,
        })
    }

    #[allow(clippy::borrowed_box)]
    /// Gets an inventory plugin, returns None if the plugin is not an Inventory variant
    pub fn get_inventory_plugin(&self, name: &str) -> Option<&Box<dyn PluginInventory>> {
        self.plugins.get(name).and_then(|plugin| match plugin {
            Plugins::Inventory(inventory) => Some(inventory),
            _ => None,
        })
    }

    // pub fn get_plugins_by_group(&self, group: Plugins) -> Vec<&PluginInfo> {
    //     self.plugins
    //         .values()
    //         .filter(|plugin_info| plugin_info.group.as_deref() == Some(group))
    //         .collect()
    // }

    /// Generic method to get plugins by variant type with a mapper function
    pub fn get_plugins_by_variant<'a, T>(
        &'a self,
        mapper: impl Fn(&'a Plugins) -> Option<T>,
    ) -> Vec<(&'a String, T)> {
        self.plugins
            .iter()
            .filter_map(|(name, plugin)| mapper(plugin).map(|p| (name, p)))
            .collect()
    }

    /// Gets all Base plugins with their trait objects
    pub fn get_plugins_by_type_base(&self) -> Vec<(&String, &Box<dyn Plugin>)> {
        get_plugins_by_variant!(self, Plugins::Base, &Box<dyn Plugin>)
    }

    /// Gets all Inventory plugins with their trait objects
    pub fn get_plugins_by_type_inventory(&self) -> Vec<(&String, &Box<dyn PluginInventory>)> {
        get_plugins_by_variant!(self, Plugins::Inventory, &Box<dyn PluginInventory>)
    }
}

pub struct PluginManager {
    pub plugins: HashMap<String, PluginInfo>,
    plugin_path: Vec<HashMap<GroupOrName, PluginEntry>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
            plugin_path: Vec::new(),
        }
    }

    pub fn activate_plugins(mut self) -> Result<PluginManager, Box<dyn std::error::Error>> {
        let meta_data = self.get_plugin_metadata();
        log::debug!("Plugin metadata: {:?}", meta_data);
        let mut registrations = Vec::new();
        if let Some(plugin_config) = meta_data.plugins {
            for (group_or_name, plugin_entry) in plugin_config {
                registrations.push((group_or_name, plugin_entry));
            }
        } else {
            log::error!("No plugin metadata found in manifest");
            return Err("No plugin metadata found in manifest".into());
        }
        if !self.plugin_path.is_empty() {
            for entry in &self.plugin_path {
                for (group_or_name, plugin_entry) in entry {
                    registrations.push((group_or_name.clone(), plugin_entry.clone()));
                }
            }
        }
        for (group_or_name, plugin_entry) in registrations {
            self.activation_registration(group_or_name.clone(), &plugin_entry)?;
        }
        Ok(self)
    }

    fn activation_registration(
        &mut self,
        group_or_name: String,
        plugin_entry: &PluginEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match plugin_entry {
            PluginEntry::Individual(path) => {
                log::debug!("Loading individual plugin: {group_or_name} {path}");
                let (library, plugins) = self.load_plugin(path)?;
                self.register_plugins_vec(plugins, None);
                let _library = ManuallyDrop::new(library);
            }
            PluginEntry::Group(group_plugins) => {
                group_plugins.iter().for_each(|(name, path)| {
                    // TODO: Implement a match PluginGroups

                    log::debug!("Loading plugin group: {group_or_name}, {name} {path}");
                    let (library, plugins) = self.load_plugin(path).unwrap();
                    self.register_plugins_vec(plugins, Some(group_or_name.clone()));
                    let _library = ManuallyDrop::new(library);
                });
            }
        }
        Ok(())
    }

    /// Registers each plugin by the name returned by the plugin's `name` method.
    /// It allows for plugins to be grouped together for easier management within
    /// a single crated if there share similar traits.
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>, group: Option<String>) {
        log::info!("Registering plugin: {:?}", plugin.name());
        let name = plugin.name().to_string();
        let plugin_info = PluginInfo { plugin, group };

        if let hash_map::Entry::Vacant(entry) = self.plugins.entry(name.clone()) {
            entry.insert(plugin_info);
        } else {
            let msg = format!("Plugin '{}' already registered", &name);
            log::error!("{msg}");
            panic!("{msg}");
        }
    }

    /// Deregisters the plugin with the given name.
    pub fn deregister_plugin(&mut self, name: &str) -> Option<String> {
        log::info!("De-registering plugin: {}", name);
        let plugin = self.plugins.remove(name);

        match plugin {
            None => None,
            Some(plugin_info) => Some(plugin_info.plugin.name()),
        }
    }

    pub fn deregister_all_plugins(&mut self) -> Vec<String> {
        let names: Vec<String> = self.plugins.drain().map(|(name, _)| name).collect();
        names.iter().for_each(|name| {
            log::info!("De-registered plugin: {}", name);
        });
        names
    }
    /// Loops over the plugins and registers them to the plugin manager
    fn register_plugins_vec(&mut self, plugins: Vec<Box<dyn Plugin>>, group: Option<String>) {
        for plugin in plugins {
            self.register_plugin(plugin, group.clone());
        }
    }

    /// Loads a plugin from a shared object file and registers it to the plugin manager.
    pub fn load_plugin(&self, filename: &str) -> PluginResult {
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
        // let metadata = if let Some(meta_data) = value
        if let Some(meta_data) = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.as_table())
        {
            let meta: Result<Metadata, toml::de::Error> =
                toml::from_str(&toml::to_string(meta_data).unwrap());
            meta.unwrap()
        } else {
            Metadata { plugins: None }
        }
        // metadata
    }

    pub fn with_path(mut self, path: &str, group: Option<&str>) -> Result<Self, Error> {
        let path = Path::new(&path);
        if path.exists() {
            let path_string = if let Some(path_str) = path.to_str() {
                path_str.to_string()
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Path contains invalid Unicode",
                ));
            };
            if let Some(group_string) = group {
                let group_info = HashMap::from([(
                    group_string.to_string(),
                    PluginEntry::Group(HashMap::from([(path_string, group_string.to_string())])),
                )]);
                self.plugin_path.push(group_info);
            } else {
                todo!()
                // TODO: implement individual plugin registration
                // let individual_info = PluginEntry::Individual(path_string);
                // self.plugin_path.push(PluginEntry::Individual(path_string));
            };
            Ok(self)
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                format!("FileNotFoundError: {:?}", path.as_os_str()),
            ))
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

    /// Gets all the **names** of the registered plugins.
    pub fn get_all_plugin_names(&self) -> Vec<&String> {
        self.plugins.keys().collect()
    }

    /// Gets all the **names** and **groups** of the registered plugins.
    pub fn get_all_plugin_names_and_groups(&self) -> Vec<(String, Option<String>)> {
        self.plugins
            .iter()
            .map(|(name, plugin_info)| (name.clone(), plugin_info.group.clone()))
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

    // /// Utility method to downcast a plugin to a specific type
    // ///
    // /// It allows you to safely access the plugin's fields and methods,
    // /// not found in the `Plugin` trait.
    // pub fn with_any<P: 'static>(&self, name: &str) -> Result<&P, Box<dyn std::error::Error>> {
    //     if let Some(plugin_info) = self.get_plugin(name) {
    //         match plugin_info.plugin.as_any().downcast_ref::<P>() {
    //             Some(plugin) => Ok(plugin),
    //             None => Err(format!("Failed to downcast plugin to type P: {}", name).into()),
    //         }
    //     } else {
    //         Err(format!("Plugin '{}' not found", name).into())
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    // use plugin_mods::plugin_a::PluginA;

    use super::*;

    fn set_env_var() {
        let file_name = match std::env::consts::OS {
            "linux" => "Cargo.toml",
            "windows" => "Cargo-windows.toml",
            "macos" => "Cargo-macos.toml",
            _ => "Cargo.toml",
        };
        let file = format!("../tests/plugin_mods/{}", file_name);
        unsafe {
            std::env::set_var("CARGO_MANIFEST_PATH", file);
        }
    }

    fn make_file_path(module_name: &str) -> String {
        let mut path_name = PathBuf::new();
        let mut module_name_prefix = String::from(std::env::consts::DLL_PREFIX);
        module_name_prefix.push_str(module_name);
        path_name.push("..");
        path_name.push("target");
        path_name.push("release");
        path_name.push(module_name_prefix);
        path_name.set_extension(std::env::consts::DLL_EXTENSION);
        path_name.to_string_lossy().to_string()
    }

    #[test]
    fn get_plugin_path_test() {
        set_env_var();
        let plugin_manager = PluginManagerNew::new();
        let metadata = plugin_manager.get_plugin_metadata();
        let plugins = metadata.plugins;
        match plugins {
            Some(plug_entry) => {
                for (group, entry) in plug_entry {
                    match entry {
                        PluginEntry::Individual(path) => {
                            assert_eq!(path, make_file_path("plugin_mods"));
                        }
                        PluginEntry::Group(path) => {
                            path.iter().for_each(|(metadata_name, path)| {
                                assert_eq!(path, &make_file_path("plugin_inventory"));
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
        let mut plugin_manager = PluginManagerNew::new();
        plugin_manager = plugin_manager.activate_plugins().unwrap();
        assert!(plugin_manager.get_plugin("plugin_a").is_some());
        assert_eq!(plugin_manager.plugins.len(), 3);
    }

    #[test]
    #[should_panic]
    /// Test for duplicate activation of plugins.
    fn activate_plugins_and_panic_test() {
        set_env_var();
        let mut plugin_manager = PluginManagerNew::new();
        plugin_manager = plugin_manager.activate_plugins().unwrap();
        _ = plugin_manager.activate_plugins().unwrap();
    }

    #[test]
    fn load_plugin_test() {
        let plugin_manager = PluginManagerNew::new();
        let filename = make_file_path("plugin_mods");
        let (_library, plugins) = plugin_manager.load_plugin(&filename).unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name(), "plugin_a");
    }

    #[test]
    fn load_plugin_and_panic_test() {
        let plugin_manager = PluginManagerNew::new();
        let filename = make_file_path("plugin_mods");
        let (_library, _) = plugin_manager.load_plugin(&filename).unwrap();
        let filename = make_file_path("plugin_mods");
        let (_library, plugins) = plugin_manager.load_plugin(&filename).unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name(), "plugin_a");
    }

    #[test]
    fn activate_plugins_with_groups_test() {
        set_env_var();
        let plugin_manager = PluginManagerNew::new().activate_plugins().unwrap();

        // Get all plugins in the "base" group
        let inventory_plugins = plugin_manager.get_plugins_by_type_base();
        assert_eq!(inventory_plugins.len(), 2);

        // Get all plugins in the "inventory" group
        let inventory_plugins = plugin_manager.get_plugins_by_type_inventory();
        assert_eq!(inventory_plugins.len(), 1);
        assert_eq!(inventory_plugins[0].1.name(), "inventory_a");

        assert_eq!(plugin_manager.plugins.len(), 3);
    }

    // #[test]
    // fn get_all_plugin_names_and_groups_test() {
    //     set_env_var();
    //     let plugin_manager = PluginManager::new().activate_plugins().unwrap();
    //     let all_plugins = plugin_manager.get_all_plugin_names_and_groups();
    //     assert_eq!(all_plugins.len(), 3);
    //     all_plugins
    //         .iter()
    //         .for_each(|(name, group)| match name.as_str() {
    //             "plugin_a" => assert_eq!(group.as_deref(), None),
    //             "plugin_b" => assert_eq!(group.as_deref(), None),
    //             "inventory_a" => assert_eq!(group.as_deref(), Some("inventory")),
    //             _ => panic!("Unexpected plugin name"),
    //         });
    // }

    // #[test]
    // fn deregister_plugin_test() {
    //     set_env_var();
    //     let mut plugin_manager = PluginManager::new().activate_plugins().unwrap();
    //     assert_eq!(plugin_manager.plugins.len(), 3);

    //     // Deregister individual plugin
    //     let plugin_name = plugin_manager.deregister_plugin("plugin_a");
    //     if let Some(plugin) = plugin_name {
    //         assert_eq!(plugin, "plugin_a");
    //         assert_eq!(plugin_manager.plugins.len(), 2);
    //     }

    //     // Deregister grouped plugin
    //     let plugin_name = plugin_manager.deregister_plugin("inventory_a");
    //     if let Some(plugin) = plugin_name {
    //         assert_eq!(plugin, "inventory_a");
    //         assert_eq!(plugin_manager.plugins.len(), 1);
    //     }

    //     // Deregister non-existent plugin
    //     let plugin_name = plugin_manager.deregister_plugin("non_existent_plugin");
    //     assert_eq!(plugin_name, None);
    // }

    // #[test]
    // fn deregister_all_plugins_test() {
    //     set_env_var();
    //     let mut plugin_manager = PluginManager::new().activate_plugins().unwrap();
    //     assert_eq!(plugin_manager.plugins.len(), 3);

    //     // Deregister all plugins
    //     let num_plugins_deregistered = plugin_manager.deregister_all_plugins();
    //     assert_eq!(num_plugins_deregistered.len(), 3);
    //     assert_eq!(plugin_manager.plugins.len(), 0);
    // }

    #[test]
    fn plugin_manager_new_test() {
        set_env_var();
        let mut plugin_manager = PluginManagerNew::new();
        assert_eq!(plugin_manager.plugins.len(), 0);
        plugin_manager = plugin_manager.activate_plugins().unwrap();
        assert_eq!(plugin_manager.plugins.len(), 3);
    }
    // #[test]
    // fn with_any_test() {

    //     let plugin_manager = PluginManager::new().activate_plugins().unwrap();
    //     let plugin_a = plugin_manager.get_plugin("plugin_a");
    //     let plugina =  match plugin_a {
    //         Some(plugin_info) => {
    //             let as_is = plugin_info.plugin.as_any().downcast_ref::<PluginA>();
    //             assert_eq!(
    //                 as_is.unwrap().name(),
    //                 "plugin_a",
    //                 "PluginManager::as_any() should return a reference to PluginA"
    //             );
    //             // :<PluginA>("plugin_a");
    //             as_is.unwrap().other_method();
    //             as_is
    //         },
    //         None => panic!("Plugin 'plugin_a' not found"),
    //     };
    //     // :<PluginA>("plugin_a");
    //     assert_eq!(plugin_manager.plugins.len(), 3);
    // }
    // TODO: write a test for PluginManager::execute_plugin
    // TODO: write a test for PluginManager::get_plugin_metadata
    // TODO: write a test for PluginManager::get_plugins_by_group
}
