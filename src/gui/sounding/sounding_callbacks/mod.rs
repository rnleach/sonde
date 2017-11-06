//! Event callbacks.

use cairo::Context;
use gdk::{EventButton, EventMotion, EventScroll, EventCrossing, ScrollDirection, EventKey,
          keyval_from_name};
use gtk::{DrawingArea, Inhibit, WidgetExt};

use app;
use coords::{DeviceCoords, XYCoords};

mod drawing;

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_sounding(cr: &Context, ac: &app::AppContextPointer) -> Inhibit {

    let mut ac = ac.borrow_mut();

    drawing::prepare_to_draw(cr, &mut ac);
    drawing::draw_background(&cr, &ac);
    drawing::draw_temperature_profiles(&cr, &ac);
    drawing::draw_wind_profile(&cr, &ac);
    drawing::draw_labels(&cr, &ac);
    drawing::draw_active_sample(&cr, &mut ac);

    Inhibit(false)
}

/// Handles zooming from the mouse whell. Connected to the scroll-event signal.
pub fn scroll_event(
    _sounding_area: &DrawingArea,
    event: &EventScroll,
    ac: &app::AppContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f64 = 1.05;
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 10.0;

    let mut ac = ac.borrow_mut();

    let pos = ac.skew_t.convert_device_to_xy(event.get_position().into());
    let dir = event.get_direction();

    let old_zoom = ac.get_zoom_factor();
    let mut new_zoom = old_zoom;

    match dir {
        ScrollDirection::Up => {
            new_zoom *= DELTA_SCALE;
        }
        ScrollDirection::Down => {
            new_zoom /= DELTA_SCALE;
        }
        _ => {}
    }

    if new_zoom < MIN_ZOOM {
        new_zoom = MIN_ZOOM;
    } else if new_zoom > MAX_ZOOM {
        new_zoom = MAX_ZOOM;
    }
    ac.set_zoom_factor(new_zoom);

    let translate = ac.get_skew_t_translation();
    let translate_x = pos.x - old_zoom / new_zoom * (pos.x - translate.x);
    let translate_y = pos.y - old_zoom / new_zoom * (pos.y - translate.y);
    let translate = XYCoords {
        x: translate_x,
        y: translate_y,
    };
    ac.set_skew_t_translation(translate);

    ac.queue_draw_skew_t_rh_omega();

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
        ac.skew_t.last_cursor_position_skew_t = Some(event.get_position().into());
        ac.skew_t.left_button_pressed = true;
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
        ac.skew_t.last_cursor_position_skew_t = None;
        ac.skew_t.left_button_pressed = false;
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles leave notify
pub fn leave_event(
    _sounding_area: &DrawingArea,
    _event: &EventCrossing,
    ac: &app::AppContextPointer,
) -> Inhibit {
    let mut ac = ac.borrow_mut();

    ac.skew_t.last_cursor_position_skew_t = None;
    ac.last_sample_pressure = None;
    ac.queue_draw_skew_t_rh_omega();

    Inhibit(false)
}

/// Handles motion events
pub fn mouse_motion_event(
    sounding_area: &DrawingArea,
    event: &EventMotion,
    ac: &app::AppContextPointer,
) -> Inhibit {

    sounding_area.grab_focus();

    let mut ac = ac.borrow_mut();
    if ac.skew_t.left_button_pressed {
        if let Some(last_position) = ac.skew_t.last_cursor_position_skew_t {
            let old_position = ac.skew_t.convert_device_to_xy(last_position);
            let new_position = DeviceCoords::from(event.get_position());
            ac.skew_t.last_cursor_position_skew_t = Some(new_position);

            let new_position = ac.skew_t.convert_device_to_xy(new_position);
            let delta = (
                new_position.x - old_position.x,
                new_position.y - old_position.y,
            );
            let mut translate = ac.get_skew_t_translation();
            translate.x -= delta.0;
            translate.y -= delta.1;
            ac.set_skew_t_translation(translate);

            ac.queue_draw_skew_t_rh_omega();
        }
    } else if ac.plottable() {
        let position: DeviceCoords = event.get_position().into();

        ac.skew_t.last_cursor_position_skew_t = Some(position);
        let tp_position = ac.skew_t.convert_device_to_tp(position);
        ac.last_sample_pressure = Some(tp_position.pressure);

        ac.queue_draw_skew_t_rh_omega();
    }
    Inhibit(false)
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

/// Handles key-press events
pub fn key_press_event(
    _sounding_area: &DrawingArea,
    _event: &EventKey,
    _ac: &app::AppContextPointer,
) -> Inhibit {
    Inhibit(true)
}
