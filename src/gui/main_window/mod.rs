use std::rc::Rc;

use glib;

use gtk;
use gtk::prelude::*;
use gtk::{Window, MenuBar, MenuItem, Menu};

use app::{AppContextPointer, AppContext};
use app::config;
use gui::Gui;

mod menu_callbacks;

pub fn layout(gui: &Gui, app_context: &AppContextPointer) {

    let ac = app_context.borrow();
    let window = gui.get_window();

    // Build the menu bar
    let menu_bar = build_menu_bar(app_context, &window);

    // Layout main gui areas
    let frames = layout_frames(gui, &ac);

    // Layout everything else
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v_box.pack_start(&menu_bar, false, false, 0);
    v_box.pack_start(&frames, true, true, 0);
    window.add(&v_box);

    configure_main_window(&window, &ac);
}

fn build_menu_bar(ac: &AppContextPointer, win: &Window) -> MenuBar {

    //
    // The file menu.
    //

    // The open item
    let open_item = MenuItem::new_with_label("Open");
    let win1 = win.clone();
    let ac1 = Rc::clone(ac);
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

fn layout_frames(gui: &Gui, ac: &AppContext) -> gtk::Paned {

    const BOX_SPACING:i32 = 0;
    const BOX_PADDING:u32 = 0;

    let skew_t = gui.get_sounding_area();
    let rh_omega = gui.get_omega_area();
    let hodo = gui.get_hodograph_area();
    let index = gui.get_index_area();
    let controls = gui.get_control_area();

    let main_pane = gtk::Paned::new(gtk::Orientation::Horizontal);
    let h_box = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);

    h_box.pack_start(&rh_omega, false, true, BOX_PADDING);
    h_box.pack_start(&skew_t, true, true, BOX_PADDING);

    let v_pane_inner = gtk::Paned::new(gtk::Orientation::Vertical);
    let v_pane_outer = gtk::Paned::new(gtk::Orientation::Vertical);

    v_pane_inner.add1(&add_border_frame(&hodo));
    v_pane_inner.add2(&add_border_frame(&index));
    v_pane_outer.add1(&v_pane_inner);
    v_pane_outer.add2(&add_border_frame(&controls));
    v_pane_inner.set_position(ac.config.window_height / 3);
    v_pane_outer.set_position(ac.config.window_height * 2 / 3);

    main_pane.add1(&add_border_frame(&h_box));
    main_pane.add2(&v_pane_outer);
    main_pane.set_position(ac.config.window_width * 11 / 16);

    main_pane
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
    f.set_shadow_type(gtk::ShadowType::In);

    f
}
