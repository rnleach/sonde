use crate::{app::AppContextPointer, gui::control_area::BOX_SPACING};
use gtk::{self, gdk::RGBA, prelude::*, Frame, ScrolledWindow};
use std::rc::Rc;

pub fn make_data_option_frame(ac: &AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    //f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);

    let skewt_frame = gtk::Frame::new(Some("Skew-T"));
    let skewt_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    skewt_frame.set_child(Some(&skewt_box));

    build_config_color!(skewt_box, "Temperature", ac, temperature_rgba);
    build_config_color!(skewt_box, "Wet Bulb", ac, wet_bulb_rgba);
    build_config_color!(skewt_box, "Dew Point", ac, dew_point_rgba);
    build_config_color!(skewt_box, "Wind", ac, wind_rgba);

    let hodo_frame = gtk::Frame::new(Some("Hodograph"));
    let hodo_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    hodo_frame.set_child(Some(&hodo_box));

    // TODO: Add Hodo configuration items - then uncomment to add it below.

    let profiles_frame = gtk::Frame::new(Some("Profiles"));
    let profiles_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    profiles_frame.set_child(Some(&profiles_box));

    build_config_color_and_check!(profiles_box, "Vertical Velocity (\u{03C9})", ac, show_omega, omega_rgba);
    build_config_color_and_check!(profiles_box, "Relative Humidity", ac, show_rh, rh_rgba);
    build_config_color_and_check!(profiles_box, "Relative Humidity (ice)", ac, show_rh_ice, rh_ice_rgba);
    build_config_color!(profiles_box, "Cloud Coverage", ac, cloud_rgba);

    let fire_plumes_frame = gtk::Frame::new(Some("Fire Plume"));
    let fire_plumes_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    fire_plumes_frame.set_child(Some(&fire_plumes_box));

    build_config_color!(fire_plumes_box, "Lifting Condensation Level", ac, fire_plume_lcl_color);
    build_config_color!(fire_plumes_box, "Level of Max Ingetgrated Buoyancy", ac, fire_plume_lmib_color);
    build_config_color!(fire_plumes_box, "Percent Wet Integrated Bouyancy", ac, fire_plume_pct_wet_cape_color);

    f.set_child(Some(&v_box));
    v_box.append(&skewt_frame);
    //v_box.append(&hodo_frame);
    v_box.append(&profiles_frame);
    v_box.append(&fire_plumes_frame);
    let sw = ScrolledWindow::new();
    sw.set_child(Some(&f));

    sw
}
