use std::rc::Rc;

use gtk;
use gtk::prelude::*;
use gtk::{Menu, MenuItem, Window};

use app::AppContextPointer;
use errors::SondeError;

mod menu_callbacks;

pub fn set_up_main_window(ac: &AppContextPointer) -> Result<(), SondeError> {
    build_menu_bar(ac)?;
    configure_main_window(ac)?;

    Ok(())
}

fn build_menu_bar(ac: &AppContextPointer) -> Result<(), SondeError> {
    //
    // The file menu.
    //

    // The open item
    let open_item = MenuItem::new_with_label("Open");
    let win1: Window = ac.fetch_widget("main_window")?;
    let ac1 = Rc::clone(ac);
    open_item.connect_activate(move |mi| menu_callbacks::open_callback(mi, &ac1, &win1));

    // The quit item
    let quit_item = MenuItem::new_with_label("Quit");
    quit_item.connect_activate(|_| {
        gtk::main_quit();
    });

    // Build the file menu
    let file_menu: Menu = ac.fetch_widget("main_menu_file")?;
    file_menu.append(&open_item);
    file_menu.append(&quit_item);

    //
    // End the file menu
    //

    Ok(())
}

fn configure_main_window(ac: &AppContextPointer) -> Result<(), SondeError> {
    let window: Window = ac.fetch_widget("main_window")?;

    let (width, height) = {
        let cfg = ac.config.borrow();
        (cfg.window_width, cfg.window_height)
    };

    if width > 0 || height > 0 {
        window.resize(width, height);
    }

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

    window.show_all();

    Ok(())
}
