use glib;

use gtk;
use gtk::prelude::*;
use gtk::{Window, WidgetExt, GridExt, MenuBar, MenuItem, Menu, ContainerExt};

use app::{AppContextPointer, AppContext};
use config;
use gui::Gui;

mod menu_callbacks;

pub fn layout(gui: Gui, app_context: AppContextPointer) {

    let window = gui.get_window();

    // Build the menu bar
    let menu_bar = build_menu_bar(&app_context, &window);

    // Layout the drawing areas
    let drawing_areas = layout_drawing_areas(&gui);

    // Layout everything else
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v_box.pack_start(&menu_bar, false, false, 0);
    v_box.pack_start(&drawing_areas, true, true, 0);
    window.add(&v_box);

    configure_main_window(&window, &app_context.borrow());

}

fn build_menu_bar(ac: &AppContextPointer, win: &Window) -> MenuBar {

    //
    // The file menu.
    //

    // The open item
    let open_item = MenuItem::new_with_label("Open");
    let win1 = win.clone();
    let ac1 = ac.clone();
    open_item.connect_activate(move |mi| menu_callbacks::open_callback(mi, &ac1, &win1));

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

fn layout_drawing_areas(gui: &Gui) -> gtk::Grid {

    let grid = gtk::Grid::new();
    grid.attach(&add_border_frame(&gui.get_sounding_area()), 0, 0, 3, 8);
    grid.attach(&add_border_frame(&gui.get_hodograph_area()), 3, 0, 1, 2);
    grid.attach(&add_border_frame(&gui.get_index_area()), 3, 2, 1, 3);
    grid.attach(&add_border_frame(&gui.get_control_area()), 3, 5, 1, 3);

    grid
}

fn configure_main_window(window: &Window, ac: &AppContext) {
    // Set up window title, size, etc
    window.set_title("Sonde");
    window.set_default_size(ac.config.window_width, ac.config.window_height);
    window.set_decorated(true);
    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
}

fn add_border_frame<P: glib::IsA<gtk::Widget>>(widget: &P) -> gtk::Frame {
    let f = gtk::Frame::new(None);
    f.add(widget);
    f.set_border_width(config::BORDER_WIDTH);

    f
}
