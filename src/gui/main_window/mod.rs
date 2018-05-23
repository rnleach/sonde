use std::rc::Rc;

use gtk;
use gtk::prelude::*;
use gtk::{Menu, MenuItem, Paned, Window};

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
    let pane: Paned = ac.fetch_widget("main_pane_view")?;

    let (width, height, pane_position) = {
        let cfg = ac.config.borrow();
        (cfg.window_width, cfg.window_height, cfg.pane_position)
    };

    if width > 0 || height > 0 {
        window.resize(width, height);
    }

    if pane_position > 0 {
        pane.set_position(pane_position / pane.get_scale_factor());
    }

    let ac1 = Rc::clone(ac);
    let pane2 = pane.clone();
    window.connect_delete_event(move |win, _| {
        let mut config = ac1.config.borrow_mut();

        let (width, height) = win.get_size();
        config.window_width = width;
        config.window_height = height;

        let pos = pane2.get_position();
        config.pane_position = pos;

        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();

    Ok(())
}
