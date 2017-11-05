use glib;

use gtk;
use gtk::prelude::*;
use gtk::{Window, MenuBar, MenuItem, Menu};

use app::{AppContextPointer, AppContext};
use app::config;
use gui::Gui;

mod menu_callbacks;

pub fn layout(gui: Gui, app_context: AppContextPointer) {

    let window = gui.get_window();

    // Build the menu bar
    let menu_bar = build_menu_bar(&app_context, &window);

    // Layout main gui areas
    let frames = layout_frames(&gui);

    // Layout everything else
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v_box.pack_start(&menu_bar, false, false, 0);
    v_box.pack_start(&frames, true, true, 0);
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

fn layout_frames(gui: &Gui) -> gtk::Grid {

    const TOTAL_WIDTH: i32 = 4;
    const TOTAL_HEIGHT: i32 = 4;

    const SKEW_T_WIDTH_FRACTION: f32 = 0.75;
    const SKEW_T_WIDTH: i32 = (TOTAL_WIDTH as f32 * SKEW_T_WIDTH_FRACTION) as i32;
    const OTHER_WIDTH: i32 = TOTAL_WIDTH - SKEW_T_WIDTH;

    const HODO_FRACTION: f32 = 0.4;
    const HODO_HEIGHT: i32 = (TOTAL_HEIGHT as f32 * HODO_FRACTION) as i32;

    const INDEX_FRACTION: f32 = 0.4;
    const INDEX_HEIGHT: i32 = (TOTAL_HEIGHT as f32 * INDEX_FRACTION) as i32;

    const CONTROL_HEIGHT: i32 = TOTAL_HEIGHT - INDEX_HEIGHT - HODO_HEIGHT;

    let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    h_box.pack_start(&gui.get_omega_area(), false, true, 0);
    h_box.pack_start(&gui.get_sounding_area(), true, true, 0);

    let grid = gtk::Grid::new();
    grid.attach(&add_border_frame(&h_box), 0, 0, SKEW_T_WIDTH, TOTAL_HEIGHT);
    grid.attach(
        &add_border_frame(&gui.get_hodograph_area()),
        SKEW_T_WIDTH,
        0,
        OTHER_WIDTH,
        HODO_HEIGHT,
    );
    grid.attach(
        &add_border_frame(&gui.get_index_area()),
        SKEW_T_WIDTH,
        HODO_HEIGHT,
        OTHER_WIDTH,
        INDEX_HEIGHT,
    );
    grid.attach(
        &add_border_frame(&gui.get_control_area()),
        SKEW_T_WIDTH,
        HODO_HEIGHT + INDEX_HEIGHT,
        OTHER_WIDTH,
        CONTROL_HEIGHT,
    );

    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);

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
    f.set_hexpand(true);
    f.set_vexpand(true);

    f
}
