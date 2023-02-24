use gtk::{
    gio::{SimpleAction, SimpleActionGroup},
    prelude::*,
    Window,
};

use super::SkewTContext;

use crate::{app::AppContextPointer, errors::SondeError};
impl SkewTContext {
    pub fn build_sounding_area_context_menu(acp: &AppContextPointer) -> Result<(), SondeError> {
        let window: Window = acp.fetch_widget("main_window")?;

        let active_readout_group = SimpleActionGroup::new();
        window.insert_action_group("skew-t", Some(&active_readout_group));

        let ac = acp.clone();
        let show_active_readout_action = SimpleAction::new("show_active_readout", None);
        show_active_readout_action.connect_activate(move |_action, _variant| {
            let mut config = ac.config.borrow_mut();
            config.show_active_readout = !config.show_active_readout;
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
            crate::gui::update_text_views(&ac);
        });
        active_readout_group.add_action(&show_active_readout_action);

        Ok(())
    }
}

/*
use super::SkewTContext;
use crate::{
    app::{config::ParcelType, AppContextPointer},
    errors::SondeError,
};
use gtk::{prelude::*, CheckMenuItem, Menu, MenuItem, RadioMenuItem, SeparatorMenuItem};
use std::rc::Rc;

macro_rules! make_heading {
    ($menu:ident, $label:expr) => {
        let heading = MenuItem::with_label($label);
        heading.set_sensitive(false);
        $menu.append(&heading);
    };
}

macro_rules! make_check_item {
    ($menu:ident, $label:expr, $acp:ident, $check_val:ident) => {
        let check_menu_item = CheckMenuItem::with_label($label);
        check_menu_item.set_active($acp.config.borrow().$check_val);

        let ac = Rc::clone($acp);
        check_menu_item.connect_toggled(move |button| {
            ac.config.borrow_mut().$check_val = button.is_active();
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
            crate::gui::update_text_views(&ac);
        });

        $menu.append(&check_menu_item);
    };
}

impl SkewTContext {
    pub fn build_sounding_area_context_menu(acp: &AppContextPointer) -> Result<(), SondeError> {
        let menu: Menu = acp.fetch_widget("sounding_context_menu")?;

        Self::build_active_readout_section_of_context_menu(&menu, acp);
        menu.append(&SeparatorMenuItem::new());
        Self::build_overlays_section_of_context_menu(&menu, acp);
        menu.append(&SeparatorMenuItem::new());
        Self::build_profiles_section_of_context_menu(&menu, acp);

        menu.show_all();

        Ok(())
    }

    fn build_active_readout_section_of_context_menu(menu: &Menu, acp: &AppContextPointer) {
        make_heading!(menu, "Active readout");
        make_check_item!(menu, "Show active readout", acp, show_active_readout);
        make_check_item!(
            menu,
            "Show active readout text",
            acp,
            show_active_readout_text
        );
        make_check_item!(
            menu,
            "Show active readout line",
            acp,
            show_active_readout_line
        );
        make_check_item!(menu, "Draw sample parcel", acp, show_sample_parcel_profile);
        make_check_item!(menu, "Draw sample mix down", acp, show_sample_mix_down);
    }

    fn build_overlays_section_of_context_menu(menu: &Menu, acp: &AppContextPointer) {
        use crate::app::config::ParcelType::*;

        make_heading!(menu, "Parcel Type");

        let sfc = RadioMenuItem::with_label("Surface");
        let mxd = RadioMenuItem::with_label_from_widget(&sfc, Some("Mixed Layer"));
        let mu = RadioMenuItem::with_label_from_widget(&sfc, Some("Most Unstable"));
        let con = RadioMenuItem::with_label_from_widget(&sfc, Some("Convective"));
        let eff = RadioMenuItem::with_label_from_widget(&sfc, Some("Effective"));

        let p_type = acp.config.borrow().parcel_type;
        match p_type {
            Surface => sfc.set_active(true),
            MixedLayer => mxd.set_active(true),
            MostUnstable => mu.set_active(true),
            Convective => con.set_active(true),
            Effective => eff.set_active(true),
        }

        fn handle_toggle(button: &RadioMenuItem, parcel_type: ParcelType, ac: &AppContextPointer) {
            if button.is_active() {
                ac.config.borrow_mut().parcel_type = parcel_type;
                ac.mark_data_dirty();
                crate::gui::draw_all(&ac);
                crate::gui::update_text_views(&ac);
            }
        }

        let ac = Rc::clone(acp);
        sfc.connect_toggled(move |button| {
            handle_toggle(button, Surface, &ac);
        });

        let ac = Rc::clone(acp);
        mxd.connect_toggled(move |button| {
            handle_toggle(button, MixedLayer, &ac);
        });

        let ac = Rc::clone(acp);
        mu.connect_toggled(move |button| {
            handle_toggle(button, MostUnstable, &ac);
        });

        let ac = Rc::clone(acp);
        con.connect_toggled(move |button| {
            handle_toggle(button, Convective, &ac);
        });

        let ac = Rc::clone(acp);
        eff.connect_toggled(move |button| {
            handle_toggle(button, Effective, &ac);
        });

        menu.append(&sfc);
        menu.append(&mxd);
        menu.append(&mu);
        menu.append(&eff);
        menu.append(&con);

        menu.append(&SeparatorMenuItem::new());

        make_heading!(menu, "Parcel Options");
        make_check_item!(menu, "Show profile", acp, show_parcel_profile);
        make_check_item!(menu, "Fill CAPE/CIN", acp, fill_parcel_areas);
        make_check_item!(menu, "Show downburst", acp, show_downburst);
        make_check_item!(menu, "Fill downburst", acp, fill_dcape_area);
        make_check_item!(menu, "Show inflow layer", acp, show_inflow_layer);

        menu.append(&SeparatorMenuItem::new());
        make_check_item!(menu, "Show PFT", acp, show_pft);

        menu.append(&SeparatorMenuItem::new());

        make_heading!(menu, "Inversions");
        make_check_item!(menu, "Show inv. mix-down", acp, show_inversion_mix_down);
    }

    fn build_profiles_section_of_context_menu(menu: &Menu, acp: &AppContextPointer) {
        make_heading!(menu, "Profiles");
        make_check_item!(menu, "Temperature", acp, show_temperature);
        make_check_item!(menu, "Wet bulb", acp, show_wet_bulb);
        make_check_item!(menu, "Dew point", acp, show_dew_point);
        make_check_item!(menu, "Wind", acp, show_wind_profile);
    }
}
*/
