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
    load_last_used_config(&app);

    // Build the GUI
    gui::initialize(&app)?;

    // Initialize the main loop.
    gtk::main();

    // Save the configuration on closing.
    save_config(&app)?;

    Ok(())
}

const CONFIG_FILE_NAME: &str = "sonde_config.yml";

pub(crate) fn load_config_from_file(
    app: &AppContext,
    config_path: &std::path::Path,
) -> Result<(), Box<dyn Error + 'static>> {
    let config = File::open(config_path)
        .and_then(|mut f| {
            let mut serialized_config = String::new();

            f.read_to_string(&mut serialized_config)
                .map(|_| serialized_config)
        })
        .map_err(|err| Box::new(err))?;

    let config = serde_yaml::from_str::<app::config::Config>(&config)?;

    *app.config.borrow_mut() = config;
    app.mark_background_dirty();
    app.mark_data_dirty();
    app.mark_data_dirty();

    Ok(())
}

fn load_last_used_config(app: &AppContext) {
    let path = match dirs::config_dir().map(|path| path.join(CONFIG_FILE_NAME)) {
        Some(p) => p,
        None => return,
    };

    let _ = load_config_from_file(app, &path);
}

pub(crate) fn save_config(app: &AppContext) -> Result<(), Box<dyn Error>> {
    if let Some(ref config_path) = dirs::config_dir().map(|path| path.join(CONFIG_FILE_NAME)) {
        save_config_with_file_name(app, config_path)?;
    } else {
        eprintln!("Error creating path to save config.");
    }

    Ok(())
}

pub(crate) fn save_config_with_file_name(
    app: &AppContext,
    config_path: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    let serialized_config = serde_yaml::to_string(&app.config)?;

    match File::create(config_path).and_then(|mut f| f.write_all(serialized_config.as_bytes())) {
        ok @ Ok(_) => ok,
        Err(err) => {
            eprintln!("Error saving configuration: \n{}", &err);
            Err(err)
        }
    }?;

    Ok(())
}
