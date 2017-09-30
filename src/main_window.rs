use gtk;
use gtk::prelude::*;
use gtk::{Window, WidgetExt, GridExt};

use ::sonde_widgets::SondeWidgets;

pub fn layout( window: Window, widgets: SondeWidgets ) {

    // TODO: Add menu bar

    // Layout the drawing areas
    let grid = gtk::Grid::new();
    grid.attach(&widgets.get_sounding_area(), 0, 0, 2, 3);
    grid.attach(&widgets.get_hodograph_area(), 2, 0, 1, 1);
    let (ia1, ia2) = widgets.get_index_areas();
    grid.attach(&ia1, 2, 1, 1, 2);
    grid.attach(&ia2, 0, 3, 3, 1);
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
