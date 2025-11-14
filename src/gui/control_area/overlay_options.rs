use crate::{app::AppContextPointer, gui::control_area::BOX_SPACING};
use gtk::{self, gdk::RGBA, prelude::*, Frame, ScrolledWindow};
use std::rc::Rc;

pub fn make_overlay_frame(ac: &AppContextPointer) -> ScrolledWindow {
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

    build_config_color!(skewt_box, "Parcel profile", ac, parcel_rgba);
    build_config_color!(
        skewt_box,
        "Inversion mix down profile",
        ac,
        inversion_mix_down_rgba
    );
    build_config_color!(
        skewt_box,
        "Indexes: Parcel Highlight",
        ac,
        parcel_indexes_highlight
    );
    build_config_color!(skewt_box, "CAPE", ac, parcel_positive_rgba);
    build_config_color!(skewt_box, "CIN", ac, parcel_negative_rgba);
    build_config_color!(skewt_box, "Downburst Profile", ac, downburst_rgba);
    build_config_color!(skewt_box, "DCAPE", ac, dcape_area_color);
    build_config_color!(skewt_box, "Effective Inflow Layer", ac, inflow_layer_rgba);
    build_config_color!(skewt_box, "PFT - SP Curve", ac, pft_sp_curve_color);
    build_config_color!(
        skewt_box,
        "PFT - Mean Specific Humidity",
        ac,
        pft_mean_q_color
    );
    build_config_color!(
        skewt_box,
        "PFT - Mean Potential Temperature",
        ac,
        pft_mean_theta_color
    );
    build_config_color!(
        skewt_box,
        "PFT - Parcel (fc) Potential Temperature",
        ac,
        pft_fc_theta_color
    );
    build_config_color!(skewt_box, "PFT - Cloud Parcel", ac, pft_cloud_parcel_color);

    let hodo_frame = gtk::Frame::new(Some("Hodograph"));
    let hodo_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    hodo_frame.set_child(Some(&hodo_box));

    build_config_color!(hodo_box, "Storm Motion (hodo)", ac, storm_motion_rgba);
    build_config_color!(hodo_box, "Helicity area color (hodo)", ac, helicity_rgba);

    // Layout boxes in the frame
    f.set_child(Some(&v_box));
    v_box.append(&skewt_frame);
    v_box.append(&hodo_frame);
    let sw = ScrolledWindow::new();
    sw.set_child(Some(&f));

    sw
}
