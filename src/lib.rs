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
    load_config(&app);

    // Build the GUI
    gui::initialize(&app)?;

    // Initialize the main loop.
    gtk::main();

    // Save the configuration on closing.
    save_config(&app)?;

    Ok(())
}

const CONFIG_FILE_NAME: &str = "sonde_config.yml";

fn load_config(app: &AppContext) {
    dirs::config_dir()
        .map(|path| path.join(CONFIG_FILE_NAME))
        .and_then(|path| match File::open(path) {
            Ok(f) => Some(f),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        })
        .and_then(|mut f| {
            let mut serialized_config = String::new();

            match f.read_to_string(&mut serialized_config) {
                Ok(_) => Some(serialized_config),
                Err(err) => {
                    eprintln!("{}", err);
                    None
                }
            }
        })
        .and_then(|serialized_config| {
            match serde_yaml::from_str::<app::config::Config>(&serialized_config) {
                Ok(cfg) => Some(cfg),
                Err(err) => {
                    eprintln!("{}", err);
                    None
                }
            }
        })
        .map(|deserialized_config| {
            *app.config.borrow_mut() = deserialized_config;
        });
}

fn save_config(app: &AppContext) -> Result<(), Box<dyn Error>> {
    let serialized_config = serde_yaml::to_string(&app.config)?;
    if let Some(config_path) = dirs::config_dir().map(|path| path.join(CONFIG_FILE_NAME)) {
        match File::create(config_path).and_then(|mut f| f.write_all(serialized_config.as_bytes()))
        {
            ok @ Ok(_) => ok,
            Err(err) => {
                eprintln!("Error saving configuration: \n{}", &err);
                Err(err)
            }
        }?
    } else {
        eprintln!("Error creating path to save config.");
    }

    Ok(())
}
