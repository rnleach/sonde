use std::fs::File;
use std::io::{Read, Write};

// Module for aggregating analysis information
mod analysis;

// Module for maintaining application state
mod app;
use crate::app::AppContext;

// Module for coordinate systems
mod coords;

// Errors
mod errors;
use crate::errors::*;

// GUI module
mod gui;

pub fn run() -> Result<(), Box<dyn Error>> {
    // Set up Gtk+
    gtk::init()?;

    // Set up data context
    let app = AppContext::initialize();

    // Load the data configuration from last time, if it exists.
    dirs::config_dir()
        .map(|path| path.join("sonde_config.yml"))
        .and_then(|path| File::open(path).ok())
        .and_then(|mut f| {
            let mut serialized_config = String::new();

            match f.read_to_string(&mut serialized_config) {
                Ok(_) => Some(serialized_config),
                Err(_) => None,
            }
        })
        .and_then(|serialized_config| {
            serde_yaml::from_str::<app::config::Config>(&serialized_config).ok()
        })
        .and_then(|deserialized_config| {
            *app.config.borrow_mut() = deserialized_config;
            Some(())
        });

    // Build the GUI
    gui::initialize(&app)?;

    // Initialize the main loop.
    gtk::main();

    // Save the configuration on closing.
    let serialized_config = serde_yaml::to_string(&app.config)?;
    if let Some(config_path) = dirs::config_dir().map(|path| path.join("sonde_config.yml")) {
        File::create(config_path).and_then(|mut f| f.write_all(serialized_config.as_bytes()))?;
    }

    Ok(())
}
