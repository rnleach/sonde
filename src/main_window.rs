use gtk;
use gtk::prelude::*;
use gtk::{Window, DrawingArea, WidgetExt, GridExt};

pub fn layout(
    window: &Window,
    sounding: &DrawingArea,
    hodograph: &DrawingArea,
    index1: &DrawingArea,
    index2: &DrawingArea,
) {

    // TODO: Add menu bar

    // Layout the drawing areas
    let grid = gtk::Grid::new();
    grid.attach(sounding, 0, 0, 2, 3);
    grid.attach(hodograph, 2, 0, 1, 1);
    grid.attach(index1, 2, 1, 1, 2);
    grid.attach(index2, 0, 3, 3, 1);
    window.add(&grid);

    // Set up window title, size, etc
    window.set_title("Sonde");
    window.set_default_size(650, 650);
    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

}
