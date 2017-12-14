// `error_chain! can recurse deeply
#![recursion_limit = "1024"]

extern crate chrono;
#[macro_use]
extern crate error_chain;
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
extern crate sounding_analysis;
extern crate sounding_base;
extern crate sounding_bufkit;

use std::rc::Rc;

// Module for maintaining application state
mod app;
use app::AppContext;

// Module for coordinate systems
mod coords;

// Errors
mod errors;
use errors::*;

// meteorological formulas
mod formula;

// GUI module
mod gui;

pub fn run() -> Result<()> {

    // Set up Gtk+
    gtk::init().chain_err(|| "Error intializing Gtk+3")?;

    // Set up data context
    let app = AppContext::new();

    // Clear the cache every time through the event loop.
    let ac = Rc::clone(&app);
    gtk::idle_add(move || {
        ac.drawing_cache.reset();
        gtk::Continue(true)
    });

    // Build the GUI
    let gui = gui::Gui::new(&app);

    app.set_gui(gui.clone());
    app.show_hide_rh_omega(); // Hide this widget if defaulted in config.

    // Initialize the main loop.
    gtk::main();

    Ok(())
}
