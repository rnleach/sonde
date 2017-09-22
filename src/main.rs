// `error_chain! can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

// GUI crates
extern crate gtk;
use gtk::prelude::*;
use gtk::{ Window, WindowType, DrawingArea, WidgetExt, GridExt };

extern crate cairo;
use cairo::Context;
use cairo::enums::{FontSlant, FontWeight};

extern crate glib;

// Library with non-gui related code
extern crate sonde_data;

// Errors
mod errors;
use errors::*;

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
    // TODO: Function to draw sounding area
    // TODO: Data type to hold the sounding area state
    sounding_area.connect_draw(draw_sounding);
    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    // Create drawing area for the hodograph
    let hodo_area = DrawingArea::new();
    // TODO: Function to draw hodo area
    // TODO: Data type to hold the hodo area state
    hodo_area.connect_draw(draw_hodo);
    hodo_area.set_hexpand(true);
    hodo_area.set_vexpand(true);

    let index_area = DrawingArea::new();
    // TODO: Function to draw index area
    // TODO: Data type to hold the index area
    index_area.connect_draw(draw_index);
    index_area.set_hexpand(true);
    index_area.set_vexpand(true);

    // create top level window
    let window = Window::new(WindowType::Toplevel);
    // TODO: function to set up and layout main window
    // TODO: Add menu bar
    // TODO: layout drawing areas
    let grid = gtk::Grid::new();
    grid.attach(&sounding_area, 0, 0, 2, 3);
    grid.attach(&hodo_area, 2, 0, 1, 1);
    grid.attach(&index_area, 2, 1, 1, 2);
    window.add(&grid);
    //window.add(&hodo_area);
    window.set_title("Sonde");
    window.set_default_size(650, 650);
    window.show_all();
    window.connect_delete_event(|_,_| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Initialize the main loop.
    gtk::main();

    Ok(())
}

fn draw_sounding(sounding_area: &DrawingArea, cr: &Context) -> Inhibit 
{
    // Draw some text to mark this area.
    cr.set_source_rgb( 0.0, 0.0, 0.0);
    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(40.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Sounding Goes Here");
    
    // Draw a red border
    let alloc = sounding_area.get_allocation();
    cr.rectangle(0.0, 0.0, alloc.width as f64, alloc.height as f64);
    cr.set_source_rgb(1.0, 0.0, 0.0);
    cr.set_line_width(9.0);
    cr.stroke();

    Inhibit(false)
}

fn draw_hodo(hodo_area: &DrawingArea, cr: &Context) -> Inhibit 
{
    // Draw some text to mark this area.
    cr.set_source_rgb( 0.0, 0.0, 0.0);
    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(40.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Hodograph Goes Here");
    
    // Draw a green border
    let alloc = hodo_area.get_allocation();
    cr.rectangle(0.0, 0.0, alloc.width as f64, alloc.height as f64);
    cr.set_source_rgb(0.0, 1.0, 0.0);
    cr.set_line_width(9.0);
    cr.stroke();

    Inhibit(false)
}

fn draw_index(index_area: &DrawingArea, cr: &Context) -> Inhibit 
{
    // Draw some text to mark this area.
    cr.set_source_rgb( 0.0, 0.0, 0.0);
    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(40.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Hodograph Goes Here");
    
    // Draw a blue border
    let alloc = index_area.get_allocation();
    cr.rectangle(0.0, 0.0, alloc.width as f64, alloc.height as f64);
    cr.set_source_rgb(0.0, 0.0, 1.0);
    cr.set_line_width(9.0);
    cr.stroke();

    Inhibit(false)
}
