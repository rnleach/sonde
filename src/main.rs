// `error_chain! can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

// GUI crates
extern crate cairo;
extern crate glib;
extern crate gtk;
use gtk::{Window, WindowType, DrawingArea};

// Library with non-gui related code
extern crate sonde_data;

// Errors
mod errors;
use errors::*;

// Support modules
mod main_window;
mod sounding;
mod hodograph;
mod index_areas;

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

    // Create drawing area for the sounding
    let sounding_area = DrawingArea::new();
    sounding::set_up_sounding_area(&sounding_area);

    // Create drawing area for the hodograph
    let hodo_area = DrawingArea::new();
    hodograph::set_up_hodograph_area(&hodo_area);

    // Create drawing areas for reporting sounding index values.
    let index_area1 = DrawingArea::new();
    let index_area2 = DrawingArea::new();
    index_areas::set_up_index_areas(&index_area1, &index_area2);

    // create top level window
    let window = Window::new(WindowType::Toplevel);
    main_window::layout(
        &window,
        &sounding_area,
        &hodo_area,
        &index_area1,
        &index_area2,
    );

    // Initialize the main loop.
    gtk::main();

    Ok(())
}
