use crate::{
    app::AppContextPointer,
    gui::control_area::{BOX_SPACING, PADDING},
};
use gdk::RGBA;
use gtk::{self, prelude::*, Adjustment, Frame, ScrolledWindow};
use std::rc::Rc;

pub fn make_data_option_frame(ac: &AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);

    let skewt_frame = gtk::Frame::new(Some("Skew-T"));
    let skewt_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    skewt_frame.add(&skewt_box);

    build_config_color!(skewt_box, "Temperature", ac, temperature_rgba);
    build_config_color!(skewt_box, "Wet Bulb", ac, wet_bulb_rgba);
    build_config_color!(skewt_box, "Dew Point", ac, dew_point_rgba);
    build_config_color!(skewt_box, "Wind", ac, wind_rgba);

    let profiles_frame = gtk::Frame::new(Some("Profiles"));
    let profiles_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    profiles_frame.add(&profiles_box);

    build_config_color_and_check!(
        profiles_box,
        "Vertical Velocity (\u{03C9})",
        ac,
        show_omega,
        omega_rgba
    );
    build_config_color_and_check!(profiles_box, "Relative Humidity", ac, show_rh, rh_rgba);
    build_config_color_and_check!(
        profiles_box,
        "Relative Humidity (ice)",
        ac,
        show_rh_ice,
        rh_ice_rgba
    );
    build_config_color!(profiles_box, "Cloud Coverage", ac, cloud_rgba);

    let fire_plumes_frame = gtk::Frame::new(Some("Fire Plume"));
    let fire_plumes_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    fire_plumes_frame.add(&fire_plumes_box);

    build_config_color!(
        fire_plumes_box,
        "Lifting Condensation Level",
        ac,
        fire_plume_lcl_color
    );
    build_config_color!(
        fire_plumes_box,
        "Equilibrium Level",
        ac,
        fire_plume_el_color
    );
    build_config_color!(fire_plumes_box, "Maximum Height", ac, fire_plume_maxh_color);
    build_config_color!(
        fire_plumes_box,
        "Percent Wet Integrated Bouyancy",
        ac,
        fire_plume_pct_wet_cape_color
    );

    f.add(&v_box);
    v_box.pack_start(&skewt_frame, true, true, PADDING);
    v_box.pack_start(&profiles_frame, true, true, PADDING);
    v_box.pack_start(&fire_plumes_frame, true, true, PADDING);
    let sw = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    sw.add(&f);

    sw
}
