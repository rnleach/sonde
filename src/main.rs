// `error_chain! can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

// GUI crates
extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;
use gtk::{Window, WindowType};

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

    // Create widgets
    let widgets = gui::sonde_widgets::SondeWidgets::new();

    // Set up data context
    let app_context = app::AppContext::new();
    {
        let mut app = app_context.borrow_mut();
        app.set_widgets(widgets.clone());
    }

    // Create drawing area for the sounding
    gui::sounding::set_up_sounding_area(&widgets.get_sounding_area(), app_context.clone());

    // Create drawing area for the hodograph
    gui::hodograph::set_up_hodograph_area(&widgets.get_hodograph_area());

    // Create drawing areas for reporting sounding index values.
    let (ia1, ia2) = widgets.get_index_areas();
    gui::index_areas::set_up_index_areas(&ia1, &ia2);

    // create top level window
    let window = Window::new(WindowType::Toplevel);
    gui::main_window::layout(window.clone(), widgets.clone(), app_context.clone());

    // Initialize the main loop.
    gtk::main();

    Ok(())
}
