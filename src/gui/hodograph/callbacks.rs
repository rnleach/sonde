
use cairo::Context;
use gtk::prelude::*;


use gdk::{EventButton, EventMotion, EventScroll, EventCrossing, ScrollDirection, EventKey,
          EventConfigure, keyval_from_name};
use gtk::{DrawingArea, Inhibit};

use app::AppContextPointer;
use coords::{DeviceCoords, XYCoords};
use gui::DrawingArgs;
use gui::plot_context::PlotContext;

pub fn draw_hodo(da: &DrawingArea, cr: &Context, acp: &AppContextPointer) -> Inhibit {

    let args = DrawingArgs::new(acp, cr);

    if acp.hodo.reset_allocation.get() {
        acp.hodo.update_cache_allocations(da);
    }

    acp.hodo.init_matrix(args);
    acp.hodo.draw_background_cached(args);
    acp.hodo.draw_data_cached(args);
    acp.hodo.draw_overlay_cached(args);

    Inhibit(false)
}

/// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
pub fn scroll_event(
    _hodo_area: &DrawingArea,
    event: &EventScroll,
    ac: &AppContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f64 = 1.05;
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 10.0;

    let pos = ac.hodo.convert_device_to_xy(
        DeviceCoords::from(event.get_position()),
    );
    let dir = event.get_direction();

    let old_zoom = ac.hodo.get_zoom_factor();
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
    ac.hodo.set_zoom_factor(new_zoom);

    let translate = ac.hodo.get_translate();
    let translate_x = pos.x - old_zoom / new_zoom * (pos.x - translate.x);
    let translate_y = pos.y - old_zoom / new_zoom * (pos.y - translate.y);
    let translate = XYCoords {
        x: translate_x,
        y: translate_y,
    };
    ac.hodo.set_translate(translate);
    ac.hodo.bound_view();
    ac.hodo.mark_background_dirty();

    ac.update_all_gui();

    Inhibit(true)
}

/// Handles button press events
pub fn button_press_event(
    _hodo_area: &DrawingArea,
    event: &EventButton,
    ac: &AppContextPointer,
) -> Inhibit {

    // Left mouse button
    if event.get_button() == 1 {
        ac.hodo.set_last_cursor_position(
            Some(event.get_position().into()),
        );
        ac.hodo.set_left_button_pressed(true);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles button release events
pub fn button_release_event(
    _hodo_area: &DrawingArea,
    event: &EventButton,
    ac: &AppContextPointer,
) -> Inhibit {
    if event.get_button() == 1 {
        ac.hodo.set_last_cursor_position(None);
        ac.hodo.set_left_button_pressed(false);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

/// Handles leave notify
pub fn leave_event(
    _hodo_area: &DrawingArea,
    _event: &EventCrossing,
    ac: &AppContextPointer,
) -> Inhibit {

    ac.hodo.set_last_cursor_position(None);

    Inhibit(false)
}

/// Handles motion events
pub fn mouse_motion_event(
    hodo_area: &DrawingArea,
    event: &EventMotion,
    ac: &AppContextPointer,
) -> Inhibit {

    hodo_area.grab_focus();

    if ac.hodo.get_left_button_pressed() {
        if let Some(last_position) = ac.hodo.get_last_cursor_position() {
            let old_position = ac.hodo.convert_device_to_xy(last_position);
            let new_position = DeviceCoords::from(event.get_position());
            ac.hodo.set_last_cursor_position(Some(new_position));

            let new_position = ac.hodo.convert_device_to_xy(new_position);
            let delta = (
                new_position.x - old_position.x,
                new_position.y - old_position.y,
            );
            let mut translate = ac.hodo.get_translate();
            translate.x -= delta.0;
            translate.y -= delta.1;
            ac.hodo.set_translate(translate);
            ac.hodo.bound_view();
            ac.hodo.mark_background_dirty();
            ac.update_all_gui();
        }
    }
    Inhibit(false)
}

/// Handles key-release events, display next or previous sounding in a series.
pub fn key_release_event(
    _hodo_area: &DrawingArea,
    _event: &EventKey,
    _dc: &AppContextPointer,
) -> Inhibit {
    Inhibit(false)
}

/// Handles key-press events
pub fn key_press_event(
    _hodo_area: &DrawingArea,
    event: &EventKey,
    ac: &AppContextPointer,
) -> Inhibit {

    let keyval = event.get_keyval();
    if keyval == keyval_from_name("Right") || keyval == keyval_from_name("KP_Right") {
        ac.display_next();
        Inhibit(true)
    } else if keyval == keyval_from_name("Left") || keyval == keyval_from_name("KP_Left") {
        ac.display_previous();
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

// TODO: remove this when size allocate is connected.
pub fn configure_event(
    _hodo_area: &DrawingArea,
    event: &EventConfigure,
    ac: &AppContextPointer,
) -> bool {

    let rect = ac.hodo.get_device_rect();
    let (width, height) = event.get_size();
    if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON ||
        (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
    {
        ac.hodo.reset_allocation();
    }
    false
}

// TODO: connect_size_allocate to update bound_view anc create the matrix
