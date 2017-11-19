use std::rc::Rc;

use gtk;
use gtk::prelude::*;
use gtk::{Frame, ScrolledWindow, ColorButton, CheckButton};
use gdk::RGBA;

use gui::control_area::{BOX_SPACING, PADDING};

use app::AppContextPointer;

pub fn make_data_option_frame(acp: &AppContextPointer) -> ScrolledWindow {
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
        acp,
        show_active_readout,
        active_readout_line_rgba
    );

    // Show hide rh-omega
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
    let rh_omega = CheckButton::new();

    // Inner scope to borrow acp
    {
        let ac = acp.borrow();
        rh_omega.set_active(ac.config.show_rh_omega_frame);
    }

    // Create rh_omega callback
    let acp1 = Rc::clone(acp);
    rh_omega.connect_toggled(move |button| {

        let mut ac = acp1.borrow_mut();

        ac.config.show_rh_omega_frame = button.get_active();
        ac.show_hide_rh_omega();

        let acp2 = Rc::clone(&acp1);
        ::gtk::idle_add(move || {
            let ac = acp2.borrow_mut();
            ac.queue_draw_skew_t_rh_omega();
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
