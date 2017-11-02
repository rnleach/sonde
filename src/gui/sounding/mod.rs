//! Module holds the code for drawing the skew-t plot.

use gdk::{SCROLL_MASK, BUTTON_PRESS_MASK, BUTTON_RELEASE_MASK, POINTER_MOTION_MASK,
          POINTER_MOTION_HINT_MASK, LEAVE_NOTIFY_MASK, KEY_RELEASE_MASK, KEY_PRESS_MASK};
use gtk::{DrawingArea, WidgetExt};

mod sounding_callbacks;

use app;

/// Initialize the drawing area and connect signal handlers.
pub fn set_up_sounding_area(sounding_area: &DrawingArea, app_context: app::AppContextPointer) {

    // Layout
    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    let ac = app_context.clone();
    sounding_area.connect_draw(move |da, cr| sounding_callbacks::draw_sounding(da, cr, &ac));

    let ac = app_context.clone();
    sounding_area.connect_scroll_event(move |da, ev| sounding_callbacks::scroll_event(da, ev, &ac));

    let ac = app_context.clone();
    sounding_area.connect_button_press_event(move |da, ev| {
        sounding_callbacks::button_press_event(da, ev, &ac)
    });

    let ac = app_context.clone();
    sounding_area.connect_button_release_event(move |da, ev| {
        sounding_callbacks::button_release_event(da, ev, &ac)
    });

    let ac = app_context.clone();
    sounding_area.connect_motion_notify_event(move |da, ev| {
        sounding_callbacks::mouse_motion_event(da, ev, &ac)
    });

    let ac = app_context.clone();
    sounding_area.connect_leave_notify_event(move |da, ev| {
        sounding_callbacks::leave_event(da, ev, &ac)
    });

    let ac = app_context.clone();
    sounding_area.connect_key_release_event(move |da, ev| {
        sounding_callbacks::key_release_event(da, ev, &ac)
    });
    let ac = app_context.clone();
    sounding_area.connect_key_press_event(move |da, ev| {
        sounding_callbacks::key_press_event(da, ev, &ac)
    });
    sounding_area.set_can_focus(true);

    sounding_area.add_events(
        (SCROLL_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK | POINTER_MOTION_HINT_MASK |
             POINTER_MOTION_MASK |
             LEAVE_NOTIFY_MASK | KEY_RELEASE_MASK | KEY_PRESS_MASK)
            .bits() as i32,
    );

}
