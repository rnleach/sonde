use gtk::{DrawingArea, WidgetExt, Inhibit};
use gdk::{EventMotion, EventCrossing};

use app;
use coords::DeviceCoords;

pub mod drawing;

/// Handles leave notify
pub fn leave_event(
    _sounding_area: &DrawingArea,
    _event: &EventCrossing,
    ac: &app::AppContextPointer,
) -> Inhibit {
    let mut ac = ac.borrow_mut();

    ac.set_sample_pressure(None);
    ac.queue_draw_skew_t_rh_omega();

    Inhibit(false)
}

/// Handles motion events
pub fn mouse_motion_event(
    rh_omega_area: &DrawingArea,
    event: &EventMotion,
    ac: &app::AppContextPointer,
) -> Inhibit {

    rh_omega_area.grab_focus();

    let mut ac = ac.borrow_mut();
    if ac.plottable() {
        let position: DeviceCoords = event.get_position().into();

        let wp_position = ac.rh_omega.convert_device_to_wp(position);
        ac.set_sample_pressure(Some(wp_position.p));

        ac.queue_draw_skew_t_rh_omega();
    }
    Inhibit(false)
}
