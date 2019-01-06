use std::rc::Rc;

use gdk::Event;
use gtk::{self, prelude::*, Menu, MenuItem, Notebook, Paned, Widget, Window};

use crate::app::{AppContext, AppContextPointer};
use crate::errors::SondeError;

mod menu_callbacks;

const TABS: [(&str, &str); 7] = [
    ("skew_t", "Skew T"),
    ("hodograph_area", "Hodograph"),
    ("text_area_container", "Text"),
    ("control_area", "Controls"),
    ("profiles_area_container", "Profiles"),
    ("console_log_container", "Console"),
    ("indexes_scrolled_window", "Indexes"),
];

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

    // The save image item
    let save_image_item = MenuItem::new_with_label("Save Image");
    let win1: Window = ac.fetch_widget("main_window")?;
    let ac1 = Rc::clone(ac);
    save_image_item.connect_activate(move |m| menu_callbacks::save_image_callback(m, &ac1, &win1));

    // The quit item
    let quit_item = MenuItem::new_with_label("Quit");
    quit_item.connect_activate(|_| {
        gtk::main_quit();
    });

    // Build the file menu
    let file_menu: Menu = ac.fetch_widget("main_menu_file")?;
    file_menu.append(&open_item);
    file_menu.append(&save_image_item);
    file_menu.append(&quit_item);

    //
    // End the file menu
    //

    Ok(())
}

fn configure_main_window(ac: &AppContextPointer) -> Result<(), SondeError> {
    let window: Window = ac.fetch_widget("main_window")?;

    layout_tabs_window(&window, ac)?;

    let ac1 = Rc::clone(ac);
    window.connect_delete_event(move |win, ev| on_delete(win, ev, &ac1));

    window.show_all();

    Ok(())
}

fn on_delete(win: &Window, _ev: &Event, ac: &AppContext) -> Inhibit {
    let mut config = ac.config.borrow_mut();

    // Save the window dimensions
    let (width, height) = win.get_size();
    config.window_width = width;
    config.window_height = height;

    // Save the Paned view slider position.
    if let Ok(pane) = ac.fetch_widget::<Paned>("main_pane_view") {
        let pos = (pane.get_position() as f32) / (width as f32);
        config.pane_position = pos;
    }

    // Save the tabs, which notebook they are in, their order, and which ones were selected.
    if let (Ok(lnb), Ok(rnb)) = (
        ac.fetch_widget::<Notebook>("left_notebook"),
        ac.fetch_widget::<Notebook>("right_notebook"),
    ) {
        let tabs: Vec<Widget> = TABS
            .iter()
            // If there is an error here, it will ALWAYS fail. So go ahead and unwrap.
            .map(|&widget_id| {
                ac.fetch_widget::<Widget>(widget_id.0)
                    .expect("Error loading widget!")
            }).collect();

        let save_tabs = |cfg_tabs: &mut Vec<String>, nb: &Notebook| {
            cfg_tabs.clear();
            for child in nb.get_children() {
                for (idx, tab) in tabs.iter().enumerate() {
                    if child == *tab {
                        cfg_tabs.push(TABS[idx].0.to_owned());
                    }
                }
            }
        };

        save_tabs(&mut config.left_tabs, &lnb);
        save_tabs(&mut config.right_tabs, &rnb);

        config.left_page_selected = lnb.get_property_page();
        config.right_page_selected = rnb.get_property_page();
    }

    gtk::main_quit();
    Inhibit(false)
}

fn layout_tabs_window(win: &Window, ac: &AppContext) -> Result<(), SondeError> {
    let cfg = ac.config.borrow();

    let pane: Paned = ac.fetch_widget("main_pane_view")?;

    let (width, height, pane_position) =
        { (cfg.window_width, cfg.window_height, cfg.pane_position) };

    if width > 0 || height > 0 {
        win.resize(width, height);
    }

    if pane_position > 0.0 {
        let (width, _) = win.get_size();
        let pos = (width as f32 * pane_position).round() as i32;

        debug_assert!(pos < width);
        pane.set_position(pos);
    }

    if !(cfg.left_tabs.is_empty() && cfg.right_tabs.is_empty()) {
        if let (Ok(lnb), Ok(rnb)) = (
            ac.fetch_widget::<Notebook>("left_notebook"),
            ac.fetch_widget::<Notebook>("right_notebook"),
        ) {
            let restore_tabs = |cfg_tabs: &Vec<String>, tgt_nb: &Notebook, other_nb: &Notebook| {
                for tab_id in cfg_tabs {
                    TABS.iter().position(|&s| s.0 == tab_id).and_then(|idx| {
                        ac.fetch_widget::<Widget>(TABS[idx].0)
                            .ok()
                            .and_then(|widget| {
                                let tgt_children = tgt_nb.get_children();
                                let other_children = other_nb.get_children();

                                if tgt_children.contains(&widget) {
                                    tgt_nb.remove(&widget);
                                } else if other_children.contains(&widget) {
                                    other_nb.remove(&widget);
                                }

                                tgt_nb.add(&widget);
                                tgt_nb.set_tab_label_text(&widget, TABS[idx].1);
                                tgt_nb.set_tab_detachable(&widget, true);
                                tgt_nb.set_tab_reorderable(&widget, true);

                                Some(())
                            })
                    });
                }
            };

            restore_tabs(&cfg.left_tabs, &lnb, &rnb);
            restore_tabs(&cfg.right_tabs, &rnb, &lnb);

            lnb.set_property_page(cfg.left_page_selected);
            rnb.set_property_page(cfg.right_page_selected);
        }
    }

    Ok(())
}
