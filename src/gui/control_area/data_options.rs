use std::rc::Rc;

use gdk::RGBA;
use gtk;
use gtk::prelude::*;
use gtk::{ColorButton, Frame, ScrolledWindow};

use crate::app::AppContextPointer;
use crate::gui::control_area::{BOX_SPACING, PADDING};

pub fn make_data_option_frame(ac: &AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);

    // First set active readout and omega-rh pane
    let sample_frame = gtk::Frame::new(Some("View"));
    let sample_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    sample_frame.add(&sample_box);

    // Active readout
    build_config_color!(sample_box, "Sampling marker", ac, active_readout_line_rgba);
    build_config_color!(
        sample_box,
        "Sample profile",
        ac,
        sample_parcel_profile_color
    );
    build_config_color!(
        sample_box,
        "Sample mix down profile",
        ac,
        sample_mix_down_rgba
    );

    // Second set is data
    let data_frame = gtk::Frame::new(Some("Profiles"));
    let data_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    data_frame.add(&data_box);

    build_config_color!(data_box, "Temperature", ac, temperature_rgba);
    build_config_color!(data_box, "Wet Bulb", ac, wet_bulb_rgba);
    build_config_color!(data_box, "Dew Point", ac, dew_point_rgba);
    build_config_color!(data_box, "Wind", ac, wind_rgba);
    build_config_color!(data_box, "Vertical Velocity (\u{03C9})", ac, omega_rgba);
    build_config_color!(data_box, "Relative Humidity", ac, rh_rgba);
    build_config_color!(data_box, "Cloud Coverage", ac, cloud_rgba);

    // Third set is overlays
    let overlays_frame = gtk::Frame::new(Some("Overlays"));
    let overlays_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    overlays_frame.add(&overlays_box);

    build_config_color!(overlays_box, "Parcel profile", ac, parcel_rgba);
    build_config_color!(
        overlays_box,
        "Inversion mix down profile",
        ac,
        inversion_mix_down_rgba
    );
    build_config_color!(overlays_box, "CAPE", ac, parcel_positive_rgba);
    build_config_color!(overlays_box, "CIN", ac, parcel_negative_rgba);
    build_config_color!(overlays_box, "Downburst profile", ac, downburst_rgba);
    build_config_color!(overlays_box, "DCAPE", ac, dcape_area_color);

    //
    // Layout boxes in the frame
    //
    f.add(&v_box);
    v_box.pack_start(&sample_frame, true, true, PADDING);
    v_box.pack_start(&data_frame, true, true, PADDING);
    v_box.pack_start(&overlays_frame, true, true, PADDING);
    let sw = ScrolledWindow::new(None, None);
    sw.add(&f);

    sw
}
