use std::rc::Rc;

use glib;

use gtk;
use gtk::prelude::*;
use gtk::{Menu, MenuBar, MenuItem, Notebook, ScrolledWindow, Window};

use app::AppContextPointer;
use app::config;
use gui::Gui;

mod menu_callbacks;

pub fn layout(gui: &Gui, ac: &AppContextPointer) {
    let window = gui.get_window();

    // Build the menu bar
    let menu_bar = build_menu_bar(ac, &window);

    // Layout main gui areas
    let frames = layout_frames(gui, ac);

    // Layout everything else
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v_box.pack_start(&menu_bar, false, false, 0);
    v_box.pack_start(&frames, true, true, 0);
    window.add(&v_box);

    configure_main_window(&window, ac);
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
    quit_item.connect_activate(|_| {
        gtk::main_quit();
    });

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

fn layout_frames(gui: &Gui, ac: &AppContextPointer) -> gtk::Paned {
    macro_rules! add_tab {
        ($notebook:ident, $widget:expr, $label:expr) => {
            $widget.set_property_margin(config::WIDGET_MARGIN);
            $notebook.add(&$widget);
            $notebook.set_tab_label_text(&$widget, $label);
        };
    }

    const BOX_SPACING: i32 = 2;

    let main_pane = gtk::Paned::new(gtk::Orientation::Horizontal);

    // Left pane
    let skew_t = gui.get_sounding_area();

    // Set up scrolled window for text area.
    let text_win = ScrolledWindow::new(None, None);
    text_win.add(&gui.get_text_area());
    let v_text_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_text_box.pack_start(&::gui::text_area::make_header_text_area(), false, true, 0);
    v_text_box.pack_start(&text_win, true, true, 0);

    // Set up hbox for profiles
    let profile_box = super::profiles::set_up_profiles_box(gui, ac, BOX_SPACING);

    // Right pane
    let notebook = Notebook::new();
    add_tab!(notebook, gui.get_hodograph_area(), "Hodograph");
    add_tab!(notebook, profile_box, "Profiles");
    add_tab!(notebook, gui.get_index_area(), "Indexes");
    add_tab!(notebook, v_text_box, "Text");
    add_tab!(notebook, gui.get_control_area(), "Controls");
    notebook.set_tab_pos(gtk::PositionType::Right);

    main_pane.add1(&add_border_frame(&skew_t));
    main_pane.add2(&notebook);

    let (width, height) = {
        let cfg = ac.config.borrow();
        (cfg.window_width, cfg.window_height)
    };

    let position = if width > height {
        height
    } else {
        width * 2 / 3
    };

    main_pane.set_position(position);

    main_pane
}

fn configure_main_window(window: &Window, ac: &AppContextPointer) {
    let (width, height) = {
        let cfg = ac.config.borrow();
        (cfg.window_width, cfg.window_height)
    };

    if width > 0 || height > 0 {
        window.set_default_size(width, height);
    }

    window.set_position(gtk::WindowPosition::Center);

    window.set_title("Sonde");
    window.set_decorated(true);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let ac1 = Rc::clone(ac);
    window.connect_configure_event(move |win, _evt| {
        let (width, height) = win.get_size();
        let mut config = ac1.config.borrow_mut();
        config.window_width = width;
        config.window_height = height;
        false
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
