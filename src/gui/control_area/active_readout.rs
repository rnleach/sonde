
use crate::{
    app::AppContextPointer,
    gui::control_area::{BOX_SPACING, PADDING},
};
use gdk::RGBA;
use gtk::{self, prelude::*, Adjustment, Frame, ScrolledWindow};
use std::rc::Rc;

pub fn make_active_readout_frame(ac: &AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);


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
    build_config_color!(sample_box, "Fire Plume Profile", ac, fire_plume_line_color);

    // Layout boxes in the frame
    f.add(&v_box);
    v_box.pack_start(&sample_frame, true, true, PADDING);
    let sw = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    sw.add(&f);

    sw
}
