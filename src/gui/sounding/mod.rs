//! Module holds the code for drawing the skew-t plot.

use std::rc::Rc;

use gdk::EventMask;
use gtk::{DrawingArea, WidgetExt};

pub mod skew_t_context;
pub mod rh_omega_context;

mod sounding_callbacks;
mod rh_omega_callbacks;

use app::AppContextPointer;

/// Initialize the drawing area and connect signal handlers.
pub fn set_up_sounding_area(sounding_area: &DrawingArea, app_context: &AppContextPointer) {

    // Layout
    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    let ac = Rc::clone(app_context);
    sounding_area.connect_draw(move |da, cr| sounding_callbacks::draw_sounding(da, cr, &ac));

    let ac = Rc::clone(app_context);
    sounding_area.connect_scroll_event(move |da, ev| sounding_callbacks::scroll_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    sounding_area.connect_button_press_event(move |da, ev| {
        sounding_callbacks::button_press_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_button_release_event(move |da, ev| {
        sounding_callbacks::button_release_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_motion_notify_event(move |da, ev| {
        sounding_callbacks::mouse_motion_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_leave_notify_event(move |da, ev| {
        sounding_callbacks::leave_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_key_release_event(move |da, ev| {
        sounding_callbacks::key_release_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_key_press_event(move |da, ev| {
        sounding_callbacks::key_press_event(da, ev, &ac)
    });

    sounding_area.set_can_focus(true);

    sounding_area.add_events(
        (EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK |
             EventMask::POINTER_MOTION_HINT_MASK |
             EventMask::POINTER_MOTION_MASK |
             EventMask::LEAVE_NOTIFY_MASK | EventMask::KEY_RELEASE_MASK |
             EventMask::KEY_PRESS_MASK)
            .bits() as i32,
    );

}

pub fn set_up_rh_omega_area(omega_area: &DrawingArea, app_context: &AppContextPointer) {

    // Layout
    omega_area.set_hexpand(false);
    omega_area.set_vexpand(true);
    omega_area.set_property_width_request(80);

    let acp = Rc::clone(app_context);
    omega_area.connect_draw(move |da, cr| {
        rh_omega_callbacks::draw_rh_omega(da, cr, &acp)
    });

    let ac = Rc::clone(app_context);
    omega_area.connect_motion_notify_event(move |da, ev| {
        rh_omega_callbacks::mouse_motion_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    omega_area.connect_leave_notify_event(move |da, ev| {
        rh_omega_callbacks::leave_event(da, ev, &ac)
    });

    omega_area.add_events(
        (EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK |
             EventMask::POINTER_MOTION_HINT_MASK |
             EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK |
             EventMask::KEY_RELEASE_MASK | EventMask::KEY_PRESS_MASK)
            .bits() as i32,
    );
}
