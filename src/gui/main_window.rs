use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{
    self,
    gio::{SimpleAction, SimpleActionGroup},
    glib::Propagation,
    prelude::*,
    Button, Notebook, Paned, Widget, Window,
};
use std::rc::Rc;

mod menu_callbacks;

const TABS: [(&str, &str); 8] = [
    ("skew_t", "Skew-T"),
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
    ($ac:ident, $id:expr, $method:tt) => {
        let button: Button = $ac.fetch_widget($id)?;
        let ac1: Rc<AppContext> = Rc::clone($ac);
        button.connect_clicked(move |_| {
            ac1.$method();
        });
    };
}

fn connect_header_bar(ac: &AppContextPointer) -> Result<(), SondeError> {
    use menu_callbacks::{
        load_default_theme, load_theme, open_toolbar_callback, save_image_callback, save_theme,
    };

    let win: Window = ac.fetch_widget("main_window")?;

    set_up_button!(ac, "file-open-button", win, open_toolbar_callback);
    set_up_button!(ac, "save-image-button", win, save_image_callback);

    set_up_button!(ac, "go-first-button", display_first);
    set_up_button!(ac, "go-previous-button", display_previous);
    set_up_button!(ac, "go-next-button", display_next);
    set_up_button!(ac, "go-last-button", display_last);

    set_up_button!(ac, "zoom-in-button", zoom_in);
    set_up_button!(ac, "zoom-out-button", zoom_out);

    set_up_button!(ac, "quit-button", win, update_window_config_and_exit);

    let window: Window = ac.fetch_widget("main_window")?;

    let burger_group = SimpleActionGroup::new();
    window.insert_action_group("hamburger", Some(&burger_group));

    let acp = ac.clone();
    let load_default_action = SimpleAction::new("load_default_theme", None);
    load_default_action.connect_activate(move |_action, _variant| {
        load_default_theme(&acp);
    });
    burger_group.add_action(&load_default_action);

    let acp = ac.clone();
    let winc = win.clone();
    let save_theme_action = SimpleAction::new("save_theme", None);
    save_theme_action.connect_activate(move |_action, _variant| {
        save_theme(&acp, &winc);
    });
    burger_group.add_action(&save_theme_action);

    let acp = ac.clone();
    let load_theme_action = SimpleAction::new("load_theme", None);
    load_theme_action.connect_activate(move |_action, _variant| {
        load_theme(&acp, &win);
    });
    burger_group.add_action(&load_theme_action);

    Ok(())
}

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
        ac.fetch_widget::<Notebook>("left-notebook"),
        ac.fetch_widget::<Notebook>("right-notebook"),
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
            for child in nb
                .pages()
                .iter::<gtk::NotebookPage>()
                .filter_map(|page| page.ok())
                .map(|page| page.child())
            {
                for (idx, tab) in tabs.iter().enumerate() {
                    if child == *tab {
                        cfg_tabs.push(TABS[idx].0.to_owned());
                    }
                }
            }
        };

        save_tabs(&mut config.left_tabs, &lnb);
        save_tabs(&mut config.right_tabs, &rnb);

        config.left_page_selected = lnb.current_page().unwrap_or(0) as i32;
        config.right_page_selected = rnb.current_page().unwrap_or(0) as i32;
    }

    win.property::<gtk::Application>("application").quit();
}

fn on_delete(win: &Window, ac: &AppContext) -> Propagation {
    update_window_config_and_exit(ac, win);
    Propagation::Proceed
}

fn layout_tabs_window(win: &Window, ac: &AppContext) -> Result<(), SondeError> {
    let cfg = ac.config.borrow();

    let pane: Paned = ac.fetch_widget("main_pane_view")?;

    let (width, height, pane_position) =
        { (cfg.window_width, cfg.window_height, cfg.pane_position) };

    if width > 0 || height > 0 {
        win.set_default_size(width, height);
    }

    if pane_position > 0.0 {
        let (width, _) = win.default_size();
        let pos = (width as f32 * pane_position).round() as i32;

        debug_assert!(pos < width);
        pane.set_position(pos);
    }

    if let (Ok(lnb), Ok(rnb)) = (
        ac.fetch_widget::<Notebook>("left-notebook"),
        ac.fetch_widget::<Notebook>("right-notebook"),
    ) {
        if !(cfg.left_tabs.is_empty() && cfg.right_tabs.is_empty()) {
            let restore_tabs = |cfg_tabs: &Vec<String>, tgt_nb: &Notebook, other_nb: &Notebook| {
                for tab_id in cfg_tabs {
                    TABS.iter().position(|&s| s.0 == tab_id).and_then(|idx| {
                        ac.fetch_widget::<Widget>(TABS[idx].0).ok().map(|widget| {
                            let tgt_children = tgt_nb
                                .pages()
                                .iter::<gtk::NotebookPage>()
                                .filter_map(|widget| widget.ok())
                                .map(|page| page.child())
                                .collect::<Vec<_>>();

                            let other_children = other_nb
                                .pages()
                                .iter::<gtk::NotebookPage>()
                                .filter_map(|widget| widget.ok())
                                .map(|page| page.child())
                                .collect::<Vec<_>>();

                            if tgt_children.contains(&widget) {
                                let pg_num = tgt_nb.page_num(&widget);
                                tgt_nb.remove_page(pg_num);
                            } else if other_children.contains(&widget) {
                                let pg_num = other_nb.page_num(&widget);
                                other_nb.remove_page(pg_num);
                            }

                            tgt_nb.append_page(&widget, None::<&Widget>);
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

        // Set the pages as detachable.
        for page in lnb
            .pages()
            .into_iter()
            .filter_map(|page_res| page_res.ok())
            .filter_map(|page| page.downcast::<gtk::NotebookPage>().ok())
        {
            page.set_detachable(true);
        }

        for page in rnb
            .pages()
            .into_iter()
            .filter_map(|page_res| page_res.ok())
            .filter_map(|page| page.downcast::<gtk::NotebookPage>().ok())
        {
            page.set_detachable(true);
        }
    }
    Ok(())
}
