//! Module holds the code for drawing the skew-t plot.

use gdk::{SCROLL_MASK, BUTTON_PRESS_MASK, BUTTON_RELEASE_MASK, POINTER_MOTION_MASK,
          POINTER_MOTION_HINT_MASK};
use gtk::{DrawingArea, WidgetExt};

mod callbacks;
mod config;
pub mod sounding_context;

/// Temperature-Pressure coordinates.
/// Origin lower left. (Temperature, Pressure)
pub type TPCoords = (f32, f32);
/// XY coordinates of the skew-t graph, range 0.0 to 1.0.
/// Origin lower left, (x,y)
pub type XYCoords = (f32, f32);
/// On screen coordinates. Meant to scale and translate XYCoords to fit on the screen.
/// Origin lower left, (x,y)
pub type ScreenCoords = (f64, f64);
/// Device coordinates (pixels positions).
///  Origin upper left, (Column, Row)
pub type DeviceCoords = (f64, f64);

/// Initialize the drawing area and connect signal handlers.
pub fn set_up_sounding_area(
    sounding_area: &DrawingArea,
    sounding_context: sounding_context::SoundingContextPointer,
) {

    // Layout
    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    let sc = sounding_context.clone();
    sounding_area.connect_draw(move |da, cr| callbacks::draw_sounding(da, cr, &sc));

    let sc = sounding_context.clone();
    sounding_area.connect_scroll_event(move |da, ev| callbacks::scroll_event(da, ev, &sc));

    let sc = sounding_context.clone();
    sounding_area.connect_button_press_event(move |da, ev| {
        callbacks::button_press_event(da, ev, &sc)
    });

    let sc = sounding_context.clone();
    sounding_area.connect_button_release_event(move |da, ev| {
        callbacks::button_release_event(da, ev, &sc)
    });

    let sc = sounding_context.clone();
    sounding_area.connect_motion_notify_event(move |da, ev| {
        callbacks::mouse_motion_event(da, ev, &sc)
    });

    sounding_area.add_events((SCROLL_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK |
         POINTER_MOTION_HINT_MASK | POINTER_MOTION_MASK)
        .bits() as i32);

}
