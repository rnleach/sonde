use std::fs::File;
use std::io::{Read, Write};

use gtk::{Builder, Application, prelude::*};

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

/// Unique Application identifier.
const APP_ID: &str = "weather.profiles.sonde";

pub fn run() -> Result<(), Box<dyn Error>> {
    // Set up data context
    let app = AppContext::initialize();

    // Load the data configuration from last time, if it exists.
    load_last_used_config(&app);

    // Create the GTKApplication
    let gtk_app = gtk::Application::builder().application_id(APP_ID).build();

    {
        let app = app.clone();
        gtk_app.connect_activate(move |gtk_app| {

            //let ui_src = include_str!("./sonde.ui");
            //let gui = Builder::from_string(ui_src);
            let gui = Builder::from_file("src/sonde.ui");

            let window: gtk::Window = gui.object("main_window").unwrap();
            window.set_application(Some(gtk_app));

            app.set_gui(gui);

            gui::initialize(&app).unwrap();

            window.show();
        });
    }

    gtk_app.run();

    // Save the configuration on closing.
    save_config(&app)?;

    Ok(())
}

/*
fn build_ui(app: &Application) {
    //let builder: Builder = Builder::from_string(include_str!("sonde.ui"));
    let builder: Builder = Builder::from_file("src/sonde.ui");

    let window: Window = builder.object("window").unwrap();
    window.set_application(Some(app));

    window.show();
}
*/

const CONFIG_FILE_NAME: &str = "sonde_config.yml";

pub(crate) fn load_config_from_file(
    app: &AppContext,
    config_path: &std::path::Path,
) -> Result<(), Box<dyn Error + 'static>> {
    // Keep the current "last file opened" info
    let last_file = app.config.borrow().last_open_file.clone();

    let config = File::open(config_path)
        .and_then(|mut f| {
            let mut serialized_config = String::new();

            f.read_to_string(&mut serialized_config)
                .map(|_| serialized_config)
        })
        .map_err(Box::new)?;

    let config = serde_yaml::from_str::<app::config::Config>(&config)?;

    *app.config.borrow_mut() = config;

    if last_file.is_some() {
        app.config.borrow_mut().last_open_file = last_file;
    }

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

//
// Make the below public for benchmarking only.
//
pub use crate::analysis::Analysis;
pub use crate::app::load_file::load_file;
