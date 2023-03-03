use gtk::{
    gio::{SimpleAction, SimpleActionGroup},
    prelude::*,
    Window,
};

use super::SkewTContext;

use crate::{
    app::{config::ParcelType, AppContextPointer},
    errors::SondeError,
};

macro_rules! make_check_item {
    ($group:ident, $action:expr, $acp:ident, $check_val:ident) => {
        let ac = $acp.clone();
        let action = SimpleAction::new($action, None);
        action.connect_activate(move |_action, _variant| {
            let mut config = ac.config.borrow_mut();
            config.$check_val = !config.$check_val;
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
            crate::gui::update_text_views(&ac);
        });
        $group.add_action(&action);
    };
}

impl SkewTContext {
    pub fn build_sounding_area_context_menu(acp: &AppContextPointer) -> Result<(), SondeError> {
        let window: Window = acp.fetch_widget("main_window")?;

        let skew_t_group = SimpleActionGroup::new();
        window.insert_action_group("skew-t", Some(&skew_t_group));

        // Set some options for the active readout.
        make_check_item!(
            skew_t_group,
            "show_active_readout",
            acp,
            show_active_readout
        );
        make_check_item!(
            skew_t_group,
            "show_active_readout_text",
            acp,
            show_active_readout_text
        );
        make_check_item!(
            skew_t_group,
            "show_active_readout_line",
            acp,
            show_active_readout_line
        );
        make_check_item!(
            skew_t_group,
            "show_sample_parcel_profile",
            acp,
            show_sample_parcel_profile
        );
        make_check_item!(
            skew_t_group,
            "show_sample_mix_down",
            acp,
            show_sample_mix_down
        );

        // Set the parcel type
        let current_parcel = match acp.config.borrow().parcel_type {
            ParcelType::Surface => "surface",
            ParcelType::MixedLayer => "mixed",
            ParcelType::MostUnstable => "unstable",
            ParcelType::Convective => "convective",
            ParcelType::Effective => "effective",
        };

        let ac = acp.clone();
        let parcel_action = SimpleAction::new_stateful(
            "parcel_type_action",
            Some(gtk::glib::VariantTy::STRING),
            current_parcel.into(),
        );

        parcel_action.connect_activate(move |action, variant| {
            let val: &str = variant.unwrap().str().unwrap();
            action.set_state(val.into());

            let parcel_type = match val {
                "surface" => ParcelType::Surface,
                "mixed" => ParcelType::MixedLayer,
                "unstable" => ParcelType::MostUnstable,
                "convective" => ParcelType::Convective,
                "effective" => ParcelType::Effective,
                _ => unreachable!(),
            };

            ac.config.borrow_mut().parcel_type = parcel_type;
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
            crate::gui::update_text_views(&ac);
        });
        skew_t_group.add_action(&parcel_action);

        // Set some options for the parcel trace
        make_check_item!(
            skew_t_group,
            "show_parcel_profile",
            acp,
            show_parcel_profile
        );
        make_check_item!(skew_t_group, "fill_parcel_areas", acp, fill_parcel_areas);
        make_check_item!(skew_t_group, "show_downburst", acp, show_downburst);
        make_check_item!(skew_t_group, "fill_dcape_area", acp, fill_dcape_area);
        make_check_item!(skew_t_group, "show_inflow_layer", acp, show_inflow_layer);

        make_check_item!(skew_t_group, "show_pft", acp, show_pft);

        make_check_item!(
            skew_t_group,
            "show_inversion_mix_down",
            acp,
            show_inversion_mix_down
        );

        make_check_item!(skew_t_group, "show_temperature", acp, show_temperature);
        make_check_item!(skew_t_group, "show_wet_bulb", acp, show_wet_bulb);
        make_check_item!(skew_t_group, "show_dew_point", acp, show_dew_point);
        make_check_item!(skew_t_group, "show_wind_profile", acp, show_wind_profile);

        Ok(())
    }
}
