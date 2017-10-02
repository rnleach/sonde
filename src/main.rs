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

// Support modules for GUI
mod sonde_widgets;
mod main_window;
mod sounding;
mod hodograph;
mod index_areas;

// Support modules for managing data
mod data_context;

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

    // TODO: Make data type to manage currently loaded soundings

    // Set up Gtk+
    gtk::init().chain_err(|| "Error intializing Gtk+3")?;

    // Create widgets
    let widgets = sonde_widgets::SondeWidgets::new();

    // Create drawing area for the sounding
    let sounding_context = sounding::create_sounding_context();
    sounding::set_up_sounding_area(&widgets.get_sounding_area(), sounding_context.clone());

    // Create drawing area for the hodograph
    hodograph::set_up_hodograph_area(&widgets.get_hodograph_area());

    // Create drawing areas for reporting sounding index values.
    let (ia1, ia2) = widgets.get_index_areas();
    index_areas::set_up_index_areas(&ia1, &ia2);

    // create top level window
    let window = Window::new(WindowType::Toplevel);
    main_window::layout(window.clone(), widgets.clone());

    // Initialize the main loop.
    gtk::main();

    Ok(())
}
