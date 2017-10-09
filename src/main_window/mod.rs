use gtk;
use gtk::prelude::*;
use gtk::{Window, WidgetExt, GridExt, MenuBar, MenuItem, Menu};

use sonde_widgets::SondeWidgets;
use super::data_context::DataContextPointer;
use super::sounding::sounding_context::SoundingContextPointer;

mod menu_callbacks;

pub fn layout(
    window: Window,
    widgets: SondeWidgets,
    data_context: DataContextPointer,
    sounding_context: SoundingContextPointer,
) {

    // Build the menu bar
    let menu_bar = build_menu_bar(&data_context, &window, &sounding_context);

    // Layout the drawing areas
    let drawing_areas = layout_drawing_areas(&widgets);

    // Layout everything else
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v_box.pack_start(&menu_bar, false, false, 0);
    v_box.pack_start(&drawing_areas, true, true, 0);
    window.add(&v_box);

    configure_main_window(&window);

}

fn build_menu_bar(dc: &DataContextPointer, win: &Window, sc: &SoundingContextPointer) -> MenuBar {

    //
    // The file menu.
    //

    // The open item
    let open_item = MenuItem::new_with_label("Open");
    let dc1 = dc.clone();
    let win1 = win.clone();
    let sc1 = sc.clone();
    open_item.connect_activate(move |mi| {
        menu_callbacks::open_callback(mi, &dc1, &win1, &sc1)
    });

    // The quit item
    let quit_item = MenuItem::new_with_label("Quit");
    quit_item.connect_activate(|_| { gtk::main_quit(); });

    // Build the file menu
    let file_menu = Menu::new();
    file_menu.append(&open_item);
    file_menu.append(&quit_item);

    // Build the file menu item
    let file = MenuItem::new_with_label("File");
    file.set_submenu(&file_menu);

    //
    // End the file menu
    //

    //
    // Build the menu bar
    //
    let menu_bar = MenuBar::new();
    menu_bar.append(&file);
    menu_bar
}

fn layout_drawing_areas(widgets: &SondeWidgets) -> gtk::Grid {

    let grid = gtk::Grid::new();
    grid.attach(&widgets.get_sounding_area(), 0, 0, 2, 3);
    grid.attach(&widgets.get_hodograph_area(), 2, 0, 1, 1);
    let (ia1, ia2) = widgets.get_index_areas();
    grid.attach(&ia1, 2, 1, 1, 2);
    grid.attach(&ia2, 0, 3, 3, 1);

    grid
}

fn configure_main_window(window: &Window) {
    // Set up window title, size, etc
    window.set_title("Sonde");
    window.set_default_size(650, 650);
    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
}
