//! Event callbacks.

use cairo::Context;
use gdk::{EventButton, EventMotion, EventScroll, ScrollDirection, EventKey, keyval_from_name};
use gtk::{DrawingArea, Inhibit, WidgetExt};

use app;

mod drawing;

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_sounding(
    sounding_area: &DrawingArea,
    cr: &Context,
    ac: &app::AppContextPointer,
) -> Inhibit {
    use self::drawing::TemperatureType::{DryBulb, WetBulb, DewPoint};

    let mut ac = ac.borrow_mut();

    drawing::prepare_to_draw(sounding_area, cr, &mut ac);

    // Draw isentrops, isotherms, isobars, ...
    drawing::draw_background_lines(&cr, &ac);
    drawing::draw_background_labels(&cr, &ac);

    // Draw temperature profiles
    drawing::draw_temperature_profile(WetBulb, &cr, &ac);
    drawing::draw_temperature_profile(DewPoint, &cr, &ac);
    drawing::draw_temperature_profile(DryBulb, &cr, &ac);

    Inhibit(false)
}

/// Handles zooming from the mouse whell. Connected to the scroll-event signal.
pub fn scroll_event(
    sounding_area: &DrawingArea,
    event: &EventScroll,
    ac: &app::AppContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f32 = 1.05;
    const MIN_ZOOM: f32 = 1.0;
    const MAX_ZOOM: f32 = 10.0;

    let mut ac = ac.borrow_mut();

    let pos = ac.convert_device_to_xy(event.get_position());
    let dir = event.get_direction();

    let old_zoom = ac.zoom_factor;

    match dir {
        ScrollDirection::Up => {
            ac.zoom_factor *= DELTA_SCALE;
        }
        ScrollDirection::Down => {
            ac.zoom_factor /= DELTA_SCALE;
        }
        _ => {}
    }

    if ac.zoom_factor < MIN_ZOOM {
        ac.zoom_factor = MIN_ZOOM;
    } else if ac.zoom_factor > MAX_ZOOM {
        ac.zoom_factor = MAX_ZOOM;
    }

    ac.translate_x = pos.0 - old_zoom / ac.zoom_factor * (pos.0 - ac.translate_x);
    ac.translate_y = pos.1 - old_zoom / ac.zoom_factor * (pos.1 - ac.translate_y);

    sounding_area.queue_draw();

    Inhibit(true)
}

/// Handles button press events
pub fn button_press_event(
    _sounding_area: &DrawingArea,
    event: &EventButton,
    ac: &app::AppContextPointer,
) -> Inhibit {

    // Left mouse button
    if event.get_button() == 1 {
        let mut ac = ac.borrow_mut();
        ac.left_button_press_start = event.get_position();
        ac.left_button_pressed = true;
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles button release events
pub fn button_release_event(
    _sounding_area: &DrawingArea,
    event: &EventButton,
    ac: &app::AppContextPointer,
) -> Inhibit {
    if event.get_button() == 1 {
        let mut ac = ac.borrow_mut();
        ac.left_button_press_start = (0.0, 0.0);
        ac.left_button_pressed = false;
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles motion events
pub fn mouse_motion_event(
    sounding_area: &DrawingArea,
    event: &EventMotion,
    ac: &app::AppContextPointer,
) -> Inhibit {

    let mut ac = ac.borrow_mut();
    if ac.left_button_pressed {
        let old_position = ac.convert_device_to_xy(ac.left_button_press_start);
        ac.left_button_press_start = event.get_position();
        let new_position = ac.convert_device_to_xy(ac.left_button_press_start);
        let delta = (
            new_position.0 - old_position.0,
            new_position.1 - old_position.1,
        );
        ac.translate_x -= delta.0;
        ac.translate_y -= delta.1;

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
    dc: &app::AppContextPointer,
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
