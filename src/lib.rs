extern crate chrono;
extern crate failure;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

// GUI crates
extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;

// Library with non-gui related code
extern crate metfor;
extern crate sounding_analysis;
extern crate sounding_base;
extern crate sounding_bufkit;

use std::fs::File;
use std::io::Write;
use std::io::Read;

// Module for maintaining application state
mod app;
use app::AppContext;

// Module for coordinate systems
mod coords;

// Errors
mod errors;
use errors::*;

// GUI module
mod gui;

pub fn run() -> Result<(), Error> {
    // Set up Gtk+
    gtk::init()?;

    // Set up data context
    let app = AppContext::new();

    // Load the data configuration from last time, if it exists.
    File::open("config.yml")
        .ok()
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
    let gui = gui::Gui::new(&app);

    app.set_gui(gui.clone());

    // Initialize the main loop.
    gtk::main();

    // Save the configuration on closing.
    let serialized_config = serde_yaml::to_string(&app.config)?;
    File::create("config.yml").and_then(|mut f| f.write_all(serialized_config.as_bytes()))?;

    Ok(())
}
