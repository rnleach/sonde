//! Module holds the code for drawing the skew-t plot area.
#![allow(dead_code)] // For now.

use gdk::{SCROLL_MASK, BUTTON_PRESS_MASK, BUTTON_RELEASE_MASK, POINTER_MOTION_MASK,
          POINTER_MOTION_HINT_MASK};
use gtk::{DrawingArea, WidgetExt};

mod callbacks;
mod config;
pub mod sounding_context;
mod utility;

/// Temperature-Pressure coordinates.
pub type TPCoords = (f32, f32);
/// XY coordinates of the skew-t graph.
pub type XYCoords = (f32, f32);
/// On screen coordinates.
pub type ScreenCoords = (f64, f64);
/// Device coordinates (pixels)
pub type DeviceCoords = (f64, f64);


/// Initialize the drawing area and connect signal handlers.
pub fn set_up_sounding_area(
    sounding_area: &DrawingArea,
    sounding_context: sounding_context::SoundingContextPointer,
) {

    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    let sc1 = sounding_context.clone();
    sounding_area.connect_draw(move |da, cr| callbacks::draw_sounding(da, cr, &sc1));

    let sc1 = sounding_context.clone();
    sounding_area.connect_scroll_event(move |da, ev| callbacks::scroll_event(da, ev, &sc1));

    let sc1 = sounding_context.clone();
    sounding_area.connect_button_press_event(move |da, ev| callbacks::button_press_event(da, ev, &sc1));

    let sc1 = sounding_context.clone();
    sounding_area.connect_button_release_event(move |da, ev| callbacks::button_release_event(da, ev, &sc1));

    let sc1 = sounding_context.clone();
    sounding_area.connect_motion_notify_event(move |da, ev| callbacks::mouse_motion_event(da, ev, &sc1));

    sounding_area.add_events((SCROLL_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK |
         POINTER_MOTION_HINT_MASK | POINTER_MOTION_MASK)
        .bits() as i32);

}
