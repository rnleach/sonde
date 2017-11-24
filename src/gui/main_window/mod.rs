use std::rc::Rc;

use glib;

use gtk;
use gtk::prelude::*;
use gtk::{Window, MenuBar, MenuItem, Menu, Notebook, ScrolledWindow};

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

    macro_rules! add_tab {
        ($notebook:ident, $widget:expr, $label:expr) => {
            $widget.set_property_margin(config::WIDGET_MARGIN);
            $notebook.add(&$widget);
            $notebook.set_tab_label_text(&$widget, $label);
        };
    }

    const BOX_SPACING: i32 = 3;
    const BOX_PADDING: u32 = 1;

    let main_pane = gtk::Paned::new(gtk::Orientation::Horizontal);

    // Left pane
    let skew_t = gui.get_sounding_area();
    let rh_omega = gui.get_omega_area();
    let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    h_box.set_property_margin(config::WIDGET_MARGIN);
    h_box.pack_start(&rh_omega, false, true, 0);
    h_box.pack_start(&skew_t, true, true, 0);

    // Set up scrolled window for text area.
    let text_win = ScrolledWindow::new(None, None);
    text_win.add(&gui.get_text_area());
    let v_text_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_text_box.pack_start(&::gui::text_area::make_header_text_area(), false, true, 0);
    v_text_box.pack_start(&text_win, true, true, 0);

    // Right pane
    let notebook = Notebook::new();
    add_tab!(notebook, gui.get_hodograph_area(), "Hodograph");
    add_tab!(notebook, gui.get_index_area(), "Indexes");
    add_tab!(notebook, v_text_box, "Text");
    add_tab!(notebook, gui.get_control_area(), "Controls");
    let gui_c = gui.clone();
    notebook.connect_change_current_page(move |_, _| {
        gui_c.draw_right_pane();
        false
    });

    main_pane.add1(&add_border_frame(&h_box));
    main_pane.add2(&notebook);
    main_pane.set_position(ac.config.window_width / 2);

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
