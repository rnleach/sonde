use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gdk::Event;
use gtk::{self, prelude::*, Button, Notebook, Paned, Widget, Window};
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
    connect_header_bar(ac)?;
    configure_main_window(ac)?;

    Ok(())
}

macro_rules! set_up_button {
    ($ac:ident, $id:expr, $win:ident, $fn:expr) => {
        let button: Button = $ac.fetch_widget($id)?;
        let ac1 = Rc::clone($ac);
        let win1 = $win.clone();
        button.connect_clicked(move |_| {
            $fn(&ac1, &win1);
        });
    };
    ($ac:ident, $id:expr, $fn:expr) => {
        let button: Button = $ac.fetch_widget($id)?;
        let ac1: Rc<AppContext> = Rc::clone($ac);
        button.connect_clicked(move |_| {
            $fn(&ac1);
        });
    };
}

fn connect_header_bar(ac: &AppContextPointer) -> Result<(), SondeError> {
    use menu_callbacks::{open_toolbar_callback, save_image_callback};

    let win: Window = ac.fetch_widget("main_window")?;

    set_up_button!(ac, "open-button", win, open_toolbar_callback);
    set_up_button!(ac, "save-image-button", win, save_image_callback);

    set_up_button!(ac, "go-first-button", |ac: &Rc<AppContext>| ac.display_first());
    set_up_button!(ac, "go-previous-button", |ac: &Rc<AppContext>| ac.display_previous());
    set_up_button!(ac, "go-next-button", |ac: &Rc<AppContext>| ac.display_next());
    set_up_button!(ac, "go-last-button", |ac: &Rc<AppContext>| ac.display_last());

    set_up_button!(ac, "zoom-in-button", |ac: &Rc<AppContext>| ac.zoom_in());
    set_up_button!(ac, "zoom-out-button", |ac: &Rc<AppContext>| ac.zoom_out());

    set_up_button!(ac, "quit-button", win, update_window_config_and_exit);

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

fn update_window_config_and_exit(ac: &AppContext, win: &Window) {
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
            })
            .collect();

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
}

fn on_delete(win: &Window, _ev: &Event, ac: &AppContext) -> Inhibit {
    update_window_config_and_exit(ac, win);
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
