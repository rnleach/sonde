//! Event callbacks.

use cairo::Context;
use gdk::{EventButton, EventMotion, EventScroll, ScrollDirection, EventKey, keyval_from_name};
use gtk::{DrawingArea, Inhibit, WidgetExt};

use super::sounding_context;
use super::super::data_context;

mod drawing;

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_sounding(
    sounding_area: &DrawingArea,
    cr: &Context,
    sc: &sounding_context::SoundingContextPointer,
    dc: &data_context::DataContextPointer,
) -> Inhibit {
    use self::drawing::TemperatureType::{DryBulb, WetBulb, DewPoint};

    let mut sc = sc.borrow_mut();
    let dc = dc.borrow();

    drawing::prepare_to_draw(sounding_area, cr, &mut sc);

    // Draw isentrops, isotherms, isobars, ...
    drawing::draw_background_lines(&cr, &sc);

    // Draw temperature profiles
    drawing::draw_temperature_profile(WetBulb, &cr, &sc, &dc);
    drawing::draw_temperature_profile(DewPoint, &cr, &sc, &dc);
    drawing::draw_temperature_profile(DryBulb, &cr, &sc, &dc);

    Inhibit(false)
}

/// Handles zooming from the mouse whell. Connected to the scroll-event signal.
pub fn scroll_event(
    sounding_area: &DrawingArea,
    event: &EventScroll,
    sc: &sounding_context::SoundingContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f32 = 1.05;
    const MIN_ZOOM: f32 = 1.0;
    const MAX_ZOOM: f32 = 10.0;

    let mut sc = sc.borrow_mut();

    let pos = sc.convert_device_to_xy(event.get_position());
    let dir = event.get_direction();

    let old_zoom = sc.zoom_factor;

    match dir {
        ScrollDirection::Up => {
            sc.zoom_factor *= DELTA_SCALE;
        }
        ScrollDirection::Down => {
            sc.zoom_factor /= DELTA_SCALE;
        }
        _ => {}
    }

    if sc.zoom_factor < MIN_ZOOM {
        sc.zoom_factor = MIN_ZOOM;
    } else if sc.zoom_factor > MAX_ZOOM {
        sc.zoom_factor = MAX_ZOOM;
    }

    sc.translate_x = pos.0 - old_zoom / sc.zoom_factor * (pos.0 - sc.translate_x);
    sc.translate_y = pos.1 - old_zoom / sc.zoom_factor * (pos.1 - sc.translate_y);

    sounding_area.queue_draw();

    Inhibit(true)
}

/// Handles button press events
pub fn button_press_event(
    _sounding_area: &DrawingArea,
    event: &EventButton,
    sc: &sounding_context::SoundingContextPointer,
) -> Inhibit {

    // Left mouse button
    if event.get_button() == 1 {
        let mut sc = sc.borrow_mut();
        sc.left_button_press_start = event.get_position();
        sc.left_button_pressed = true;
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles button release events
pub fn button_release_event(
    _sounding_area: &DrawingArea,
    event: &EventButton,
    sc: &sounding_context::SoundingContextPointer,
) -> Inhibit {
    if event.get_button() == 1 {
        let mut sc = sc.borrow_mut();
        sc.left_button_press_start = (0.0, 0.0);
        sc.left_button_pressed = false;
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles motion events
pub fn mouse_motion_event(
    sounding_area: &DrawingArea,
    event: &EventMotion,
    sc: &sounding_context::SoundingContextPointer,
) -> Inhibit {

    let mut sc = sc.borrow_mut();
    if sc.left_button_pressed {
        let old_position = sc.convert_device_to_xy(sc.left_button_press_start);
        sc.left_button_press_start = event.get_position();
        let new_position = sc.convert_device_to_xy(sc.left_button_press_start);
        let delta = (
            new_position.0 - old_position.0,
            new_position.1 - old_position.1,
        );
        sc.translate_x -= delta.0;
        sc.translate_y -= delta.1;

        sounding_area.queue_draw();
        Inhibit(true)
    } else {

        Inhibit(false)
    }
}

/// Handles key-release events, display next or previous sounding in a series.
pub fn key_release_event(
    _sounding_area: &DrawingArea,
    event: &EventKey,
    dc: &data_context::DataContextPointer,
) -> Inhibit {

    let keyval = event.get_keyval();
    if keyval == keyval_from_name("Right") || keyval == keyval_from_name("KP_Right") {
        let mut dc = dc.borrow_mut();
        dc.display_next();
        Inhibit(true)
    } else if keyval == keyval_from_name("Left") || keyval == keyval_from_name("KP_Left") {
        let mut dc = dc.borrow_mut();
        dc.display_previous();
        Inhibit(true)
    } else {
        Inhibit(false)
    }


}
