//! Event callbacks.

use cairo::Context;
use gdk::{EventButton, EventMotion, EventScroll, EventCrossing, ScrollDirection, EventKey,
          keyval_from_name};
use gtk::{DrawingArea, Inhibit, WidgetExt};

use app::AppContextPointer;
use coords::{DeviceCoords, XYCoords};
use gui::plot_context::PlotContext;
use gui::DrawingArgs;

mod drawing;

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_sounding(da: &DrawingArea, cr: &Context, ac: &AppContextPointer) -> Inhibit {

    let ac = &ac.borrow();

    let args = DrawingArgs::new(ac, cr, da);

    drawing::prepare_to_draw(args);
    drawing::draw_background(args);
    drawing::draw_labels(args);
    drawing::draw_temperature_profiles(args);
    drawing::draw_wind_profile(args);
    drawing::draw_active_sample(args);

    Inhibit(false)
}

/// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
pub fn scroll_event(
    sounding_area: &DrawingArea,
    event: &EventScroll,
    ac: &AppContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f64 = 1.05;
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 10.0;

    let mut ac = ac.borrow_mut();

    let pos = ac.skew_t.convert_device_to_xy(
        sounding_area,
        DeviceCoords::from(event.get_position()),
    );
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
    ac.skew_t.set_zoom_factor(new_zoom);

    let translate = ac.skew_t.get_translate();
    let translate_x = pos.x - old_zoom / new_zoom * (pos.x - translate.x);
    let translate_y = pos.y - old_zoom / new_zoom * (pos.y - translate.y);
    let translate = XYCoords {
        x: translate_x,
        y: translate_y,
    };
    ac.skew_t.set_translate(translate);

    // Bound the xy-coords to always be on screen.
    ac.skew_t.bound_view(sounding_area);
    let translate = ac.skew_t.get_translate();
    ac.rh_omega.set_translate_y(translate);

    ac.update_all_gui();

    Inhibit(true)
}

/// Handles button press events
pub fn button_press_event(
    _sounding_area: &DrawingArea,
    event: &EventButton,
    ac: &AppContextPointer,
) -> Inhibit {

    // Left mouse button
    if event.get_button() == 1 {
        let mut ac = ac.borrow_mut();
        ac.skew_t.set_last_cursor_position(
            Some(event.get_position().into()),
        );
        ac.skew_t.set_left_button_pressed(true);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles button release events
pub fn button_release_event(
    _sounding_area: &DrawingArea,
    event: &EventButton,
    ac: &AppContextPointer,
) -> Inhibit {
    if event.get_button() == 1 {
        let mut ac = ac.borrow_mut();
        ac.skew_t.set_last_cursor_position(None);
        ac.skew_t.set_left_button_pressed(false);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles leave notify
pub fn leave_event(
    _sounding_area: &DrawingArea,
    _event: &EventCrossing,
    ac: &AppContextPointer,
) -> Inhibit {
    let mut ac = ac.borrow_mut();

    ac.skew_t.set_last_cursor_position(None);
    ac.set_sample(None);
    ac.update_all_gui();

    Inhibit(false)
}

/// Handles motion events
pub fn mouse_motion_event(
    sounding_area: &DrawingArea,
    event: &EventMotion,
    ac: &AppContextPointer,
) -> Inhibit {

    sounding_area.grab_focus();

    let mut ac = ac.borrow_mut();
    if ac.skew_t.get_left_button_pressed() {
        if let Some(last_position) = ac.skew_t.get_last_cursor_position() {
            let old_position = ac.skew_t.convert_device_to_xy(sounding_area, last_position);
            let new_position = DeviceCoords::from(event.get_position());
            ac.skew_t.set_last_cursor_position(Some(new_position));

            let new_position = ac.skew_t.convert_device_to_xy(sounding_area, new_position);
            let delta = (
                new_position.x - old_position.x,
                new_position.y - old_position.y,
            );
            let mut translate = ac.skew_t.get_translate();
            translate.x -= delta.0;
            translate.y -= delta.1;
            ac.skew_t.set_translate(translate);

            // Bound the xy-coords to always be on screen.
            ac.skew_t.bound_view(sounding_area);
            let translate = ac.skew_t.get_translate();
            ac.rh_omega.set_translate_y(translate);

            ac.set_sample(None);
            ac.update_all_gui();
        }
    } else if ac.plottable() {
        let position: DeviceCoords = event.get_position().into();

        ac.skew_t.set_last_cursor_position(Some(position));
        let tp_position = ac.skew_t.convert_device_to_tp(sounding_area, position);
        let sample = ::sounding_analysis::linear_interpolate(
            ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
            tp_position.pressure,
        );
        ac.set_sample(Some(sample));
        ac.update_all_gui();
    }
    Inhibit(false)
}

/// Handles key-release events, display next or previous sounding in a series.
pub fn key_release_event(
    _sounding_area: &DrawingArea,
    _event: &EventKey,
    _ac: &AppContextPointer,
) -> Inhibit {
    Inhibit(false)
}

/// Handles key-press events
pub fn key_press_event(
    _sounding_area: &DrawingArea,
    event: &EventKey,
    ac: &AppContextPointer,
) -> Inhibit {

    let keyval = event.get_keyval();
    if keyval == keyval_from_name("Right") || keyval == keyval_from_name("KP_Right") {
        let mut ac = ac.borrow_mut();
        ac.display_next();
        Inhibit(true)
    } else if keyval == keyval_from_name("Left") || keyval == keyval_from_name("KP_Left") {
        let mut ac = ac.borrow_mut();
        ac.display_previous();
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}
