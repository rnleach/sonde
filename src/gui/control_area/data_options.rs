use std::rc::Rc;

use gtk;
use gtk::prelude::*;
use gtk::{Frame, ScrolledWindow, ColorButton, CheckButton};
use gdk::RGBA;

use gui::control_area::{BOX_SPACING, PADDING};

use app::AppContextPointer;

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
    build_config_color_and_check!(
        sample_box,
        "Sampling",
        ac,
        show_active_readout,
        active_readout_line_rgba
    );

    // Show hide rh-omega
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
    let rh_omega = CheckButton::new();

    rh_omega.set_active(ac.config.borrow().show_rh_omega_frame);

    // Create rh_omega callback
    let acp = Rc::clone(ac);
    rh_omega.connect_toggled(move |button| {

        acp.config.borrow_mut().show_rh_omega_frame = button.get_active();
        acp.show_hide_rh_omega();

        let acp2 = Rc::clone(&acp);
        ::gtk::idle_add(move || {
            acp2.update_all_gui();
            Continue(false)
        });
    });

    // Layout
    hbox.pack_start(&rh_omega, false, true, PADDING);
    hbox.pack_start(&gtk::Label::new("RH-Omega"), false, true, PADDING);

    sample_box.pack_start(&hbox, false, true, PADDING);

    // Second set is data
    let data_frame = gtk::Frame::new(Some("Profiles"));
    let data_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    data_frame.add(&data_box);

    let acp = Rc::clone(ac);
    build_config_color_and_check!(data_box, "Wet Bulb", acp, show_wet_bulb, wet_bulb_rgba);
    build_config_color_and_check!(data_box, "Dew Point", acp, show_dew_point, dew_point_rgba);
    build_config_color_and_check!(
        data_box,
        "Temperature",
        acp,
        show_temperature,
        temperature_rgba
    );
    build_config_color_and_check!(data_box, "Wind", acp, show_wind_profile, wind_rgba);
    build_config_color_and_check!(
        data_box,
        "Vertical Velocity (\u{03C9})",
        acp,
        show_omega_profile,
        omega_rgba
    );
    build_config_color_and_check!(data_box, "Relative Humidity", acp, show_rh_profile, rh_rgba);

    //
    // Layout boxes in the frame
    //
    f.add(&v_box);
    v_box.pack_start(&sample_frame, true, true, PADDING);
    v_box.pack_start(&data_frame, true, true, PADDING);
    let sw = ScrolledWindow::new(None, None);
    sw.add(&f);

    sw
}
