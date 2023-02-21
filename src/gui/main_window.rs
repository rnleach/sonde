use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{
    self,
    //gdk::Event,
    prelude::*,
    //Button,
    Inhibit,
    //    Menu, MenuItem,
    Notebook,
    Paned,
    //    SeparatorMenuItem,
    Widget,
    Window,
};
use std::rc::Rc;

mod menu_callbacks;

const TABS: [(&str, &str); 8] = [
    ("skew_t", "Skew T"),
    ("hodograph_area", "Hodograph"),
    ("fire_plume_container", "Fire Plume"),
    ("text_area_container", "Text"),
    ("control_area", "Controls"),
    ("profiles_area_container", "Profiles"),
    ("indexes_scrolled_window", "Indexes"),
    ("provider_data_text_container", "Provider Data"),
];

pub fn set_up_main_window(ac: &AppContextPointer) -> Result<(), SondeError> {
    //    connect_header_bar(ac)?;
    configure_main_window(ac)?;

    Ok(())
}

/*
macro_rules! set_up_button {
    ($ac:ident, $id:expr, $win:ident, $fn:expr) => {
        let button: Button = $ac.fetch_widget($id)?;
        let ac1 = Rc::clone($ac);
        let win1 = $win.clone();
        button.connect_clicked(move |_| {
            $fn(&ac1, &win1);
        });
    };
    ($ac:ident, $id:expr, $method:tt) => {
        let button: Button = $ac.fetch_widget($id)?;
        let ac1: Rc<AppContext> = Rc::clone($ac);
        button.connect_clicked(move |_| {
            ac1.$method();
        });
    };
}

macro_rules! set_up_hamburger_menu_item {
    ($text:expr, $ac:ident, $win:ident, $fn:expr, $parent_menu:ident) => {
        let menu_item: MenuItem = MenuItem::with_label($text);
        let ac1 = Rc::clone($ac);
        let win1 = $win.clone();
        menu_item.connect_activate(move |_| $fn(&ac1, &win1));
        $parent_menu.append(&menu_item);
    };
}

fn connect_header_bar(ac: &AppContextPointer) -> Result<(), SondeError> {
    use menu_callbacks::{
        load_default_theme, load_theme, open_toolbar_callback, save_image_callback, save_theme,
    };

    let win: Window = ac.fetch_widget("main_window")?;

    set_up_button!(ac, "open-button", win, open_toolbar_callback);
    set_up_button!(ac, "save-image-button", win, save_image_callback);

    set_up_button!(ac, "go-first-button", display_first);
    set_up_button!(ac, "go-previous-button", display_previous);
    set_up_button!(ac, "go-next-button", display_next);
    set_up_button!(ac, "go-last-button", display_last);

    set_up_button!(ac, "zoom-in-button", zoom_in);
    set_up_button!(ac, "zoom-out-button", zoom_out);

    set_up_button!(ac, "quit-button", win, update_window_config_and_exit);

    // Set up the hamburger menu
    let menu: Menu = ac.fetch_widget("hamburger-menu")?;

    set_up_hamburger_menu_item!("Save Theme", ac, win, save_theme, menu);
    set_up_hamburger_menu_item!("Load Theme", ac, win, load_theme, menu);

    menu.append(&SeparatorMenuItem::new());

    set_up_hamburger_menu_item!("Load Default Theme", ac, win, load_default_theme, menu);

    menu.show_all();

    Ok(())
}
*/

fn configure_main_window(ac: &AppContextPointer) -> Result<(), SondeError> {
    let window: Window = ac.fetch_widget("main_window")?;

    layout_tabs_window(&window, ac)?;

    let ac1 = Rc::clone(ac);
    window.connect_close_request(move |win| on_delete(win, &ac1));

    Ok(())
}

fn update_window_config_and_exit(ac: &AppContext, win: &Window) {
    let mut config = ac.config.borrow_mut();

    // Save the window dimensions
    let (width, height) = (win.width(), win.height());
    config.window_width = width;
    config.window_height = height;

    // Save the Paned view slider position.
    if let Ok(pane) = ac.fetch_widget::<Paned>("main_pane_view") {
        let pos = (pane.position() as f32) / (width as f32);
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
            })
            .collect();

        //        let save_tabs = |cfg_tabs: &mut Vec<String>, nb: &Notebook| {
        //            cfg_tabs.clear();
        //            for child in nb.children() {
        //                for (idx, tab) in tabs.iter().enumerate() {
        //                    if child == *tab {
        //                        cfg_tabs.push(TABS[idx].0.to_owned());
        //                    }
        //                }
        //            }
        //        };

        //        save_tabs(&mut config.left_tabs, &lnb);
        //        save_tabs(&mut config.right_tabs, &rnb);

        //        config.left_page_selected = lnb.page();
        //        config.right_page_selected = rnb.page();
    }

    win.property::<gtk::Application>("application").quit();
}

fn on_delete(win: &Window, ac: &AppContext) -> Inhibit {
    update_window_config_and_exit(ac, win);
    Inhibit(false)
}

fn layout_tabs_window(win: &Window, ac: &AppContext) -> Result<(), SondeError> {
    let cfg = ac.config.borrow();

    //    FIXME
    //    let pane: Paned = ac.fetch_widget("main_pane_view")?;

    //    let (width, height, pane_position) =
    //        { (cfg.window_width, cfg.window_height, cfg.pane_position) };

    //    if width > 0 || height > 0 {
    //        win.resize(width, height);
    //    }

    //    if pane_position > 0.0 {
    //        let (width, _) = win.size();
    //        let pos = (width as f32 * pane_position).round() as i32;
    //
    //        debug_assert!(pos < width);
    //        pane.set_position(pos);
    //    }

    /*
    if !(cfg.left_tabs.is_empty() && cfg.right_tabs.is_empty()) {
        if let (Ok(lnb), Ok(rnb)) = (
            ac.fetch_widget::<Notebook>("left_notebook"),
            ac.fetch_widget::<Notebook>("right_notebook"),
        ) {
            let restore_tabs = |cfg_tabs: &Vec<String>, tgt_nb: &Notebook, other_nb: &Notebook| {
                for tab_id in cfg_tabs {
                    TABS.iter().position(|&s| s.0 == tab_id).and_then(|idx| {
                        ac.fetch_widget::<Widget>(TABS[idx].0).ok().map(|widget| {
                            let tgt_children = tgt_nb.children();
                            let other_children = other_nb.children();

                            if tgt_children.contains(&widget) {
                                tgt_nb.remove(&widget);
                            } else if other_children.contains(&widget) {
                                other_nb.remove(&widget);
                            }

                            tgt_nb.add(&widget);
                            tgt_nb.set_tab_label_text(&widget, TABS[idx].1);
                            tgt_nb.set_tab_detachable(&widget, true);
                            tgt_nb.set_tab_reorderable(&widget, true);
                        })
                    });
                }
            };

            restore_tabs(&cfg.left_tabs, &lnb, &rnb);
            restore_tabs(&cfg.right_tabs, &rnb, &lnb);

            lnb.set_page(cfg.left_page_selected);
            rnb.set_page(cfg.right_page_selected);
        }
    }
    */

    Ok(())
}
