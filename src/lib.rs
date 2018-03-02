// `error_chain! can recurse deeply
#![recursion_limit = "1024"]

extern crate chrono;
extern crate failure;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate lazy_static;

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

    // FIXME: Dead code? Does nothing?
    // Clear the cache every time through the event loop.
    gtk::idle_add(move || gtk::Continue(true));

    // Build the GUI
    let gui = gui::Gui::new(&app);

    app.set_gui(gui.clone());

    // Initialize the main loop.
    gtk::main();

    Ok(())
}
