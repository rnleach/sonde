use cairo::Context;
use gtk::prelude::*;
use gtk::{DrawingArea, Inhibit};
use gdk::{EventMotion, EventCrossing};

use app::AppContextPointer;
use coords::DeviceCoords;
use gui::DrawingArgs;

pub mod drawing;

/// Handles leave notify
pub fn leave_event(
    _sounding_area: &DrawingArea,
    _event: &EventCrossing,
    ac: &AppContextPointer,
) -> Inhibit {
    let mut ac = ac.borrow_mut();

    ac.set_sample(None);
    ac.update_all_gui();

    Inhibit(false)
}

/// Handles motion events
pub fn mouse_motion_event(
    rh_omega_area: &DrawingArea,
    event: &EventMotion,
    ac: &AppContextPointer,
) -> Inhibit {

    rh_omega_area.grab_focus();

    let mut ac = ac.borrow_mut();
    if ac.plottable() {
        let position: DeviceCoords = event.get_position().into();

        let wp_position = ac.rh_omega.convert_device_to_wp(rh_omega_area, position);
        let sample = ::sounding_analysis::linear_interpolate(
            ac.get_sounding_for_display().unwrap(),
            wp_position.p,
        );
        ac.set_sample(Some(sample));

        ac.update_all_gui();
    }
    Inhibit(false)
}

pub fn draw_rh_omega(da: &DrawingArea, cr: &Context, ac: &AppContextPointer) -> Inhibit {

    let ac = &ac.borrow();

    let args = DrawingArgs::new(ac, cr, da);

    drawing::prepare_to_draw(args);
    drawing::draw_background(args);
    drawing::draw_rh_profile(args);
    drawing::draw_omega_profile(args);
    drawing::draw_active_readout(args);

    Inhibit(false)
}
