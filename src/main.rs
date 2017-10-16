// `error_chain! can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

// GUI crates
extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;

// Library with non-gui related code
extern crate sounding_base;
extern crate sounding_bufkit;

// Errors
mod errors;
use errors::*;

// Module for maintaining application state
mod app;

// Module for configuring application
mod config;

// GUI module
mod gui;

fn main() {

    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {

    // Set up Gtk+
    gtk::init().chain_err(|| "Error intializing Gtk+3")?;

    // Set up data context
    let app_context = app::AppContext::new();

    // Build the GUI
    let gui = gui::Gui::new(app_context.clone());

    // Connect the gui back to the app_context
    {
        let mut app = app_context.borrow_mut();
        app.set_gui(gui.clone());
    }

    // Initialize the main loop.
    gtk::main();

    Ok(())
}
