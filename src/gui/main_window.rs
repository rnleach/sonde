use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gdk::Event;
use gtk::{
    self, prelude::*, Button, HeaderBar, IconSize, Notebook, Orientation, Paned, Separator, Widget,
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
    connect_header_bar(ac)?;
    configure_main_window(ac)?;

    Ok(())
}

fn connect_header_bar(ac: &AppContextPointer) -> Result<(), SondeError> {
    let win: Window = ac.fetch_widget("main_window")?;
    let header_bar: HeaderBar = ac.fetch_widget("header-bar")?;

    let open_button = Button::new_from_icon_name(Some("document-open"), IconSize::SmallToolbar);
    open_button.set_label("Open");
    open_button.set_always_show_image(true);
    let ac1 = Rc::clone(ac);
    let win1 = win.clone();
    open_button.connect_clicked(move |_| {
        menu_callbacks::open_toolbar_callback(&ac1, &win1);
    });
    header_bar.pack_start(&open_button);

    let save_image_button =
        Button::new_from_icon_name(Some("insert-image"), IconSize::SmallToolbar);
    save_image_button.set_label("Save Image");
    save_image_button.set_always_show_image(true);
    let ac1 = Rc::clone(ac);
    let win1 = win.clone();
    save_image_button.connect_clicked(move |_| menu_callbacks::save_image_callback(&ac1, &win1));
    header_bar.pack_start(&save_image_button);

    header_bar.pack_start(&Separator::new(Orientation::Vertical));

    let first_button = Button::new_from_icon_name(Some("go-first"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    first_button.connect_clicked(move |_| ac1.display_first());
    header_bar.pack_start(&first_button);

    let previous_button =
        Button::new_from_icon_name(Some("media-skip-backward"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    previous_button.connect_clicked(move |_| ac1.display_previous());
    header_bar.pack_start(&previous_button);

    let next_button =
        Button::new_from_icon_name(Some("media-skip-forward"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    next_button.connect_clicked(move |_| ac1.display_next());
    header_bar.pack_start(&next_button);

    let last_button = Button::new_from_icon_name(Some("go-last"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    last_button.connect_clicked(move |_| ac1.display_last());
    header_bar.pack_start(&last_button);

    header_bar.pack_start(&Separator::new(Orientation::Vertical));

    let zoom_in_button = Button::new_from_icon_name(Some("zoom-in"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    zoom_in_button.connect_clicked(move |_| ac1.zoom_in());
    header_bar.pack_start(&zoom_in_button);

    let zoom_out_button = Button::new_from_icon_name(Some("zoom-out"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    zoom_out_button.connect_clicked(move |_| ac1.zoom_out());
    header_bar.pack_start(&zoom_out_button);

    let quit_button =
        Button::new_from_icon_name(Some("application-exit-symbolic"), IconSize::SmallToolbar);
    let ac1 = Rc::clone(ac);
    quit_button.connect_clicked(move |_| {
        update_window_config_and_exit(&win, &ac1);
    });
    header_bar.pack_end(&quit_button);

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

fn update_window_config_and_exit(win: &Window, ac: &AppContext) {
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
    update_window_config_and_exit(win, ac);
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
