use cairo::Context;

use gdk::{keyval_from_name, EventButton, EventCrossing, EventKey, EventMotion, EventScroll,
          EventConfigure, ScrollDirection};

use gtk::prelude::*;
use gtk::{DrawingArea, Allocation};

use app::AppContextPointer;
use coords::{DeviceCoords, XYCoords};
use gui::DrawingArgs;
use gui::plot_context::PlotContext;

pub fn draw_skew_t(cr: &Context, acp: &AppContextPointer) -> Inhibit {
    let args = DrawingArgs::new(acp, cr);

    acp.skew_t.init_matrix(args);
    acp.skew_t.draw_background_cached(args);
    acp.skew_t.draw_data_cached(args);
    acp.skew_t.draw_overlay_cached(args);

    Inhibit(false)
}

/// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
pub fn scroll_event(_da: &DrawingArea, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
    const DELTA_SCALE: f64 = 1.05;
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 10.0;

    let pos = ac.skew_t.convert_device_to_xy(DeviceCoords::from(event.get_position()));
    let dir = event.get_direction();

    let old_zoom = ac.skew_t.get_zoom_factor();
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

    let mut translate = ac.skew_t.get_translate();
    translate = XYCoords {
        x: pos.x - old_zoom / new_zoom * (pos.x - translate.x),
        y: pos.y - old_zoom / new_zoom * (pos.y - translate.y),
    };
    ac.skew_t.set_translate(translate);
    ac.skew_t.bound_view();
    ac.skew_t.mark_background_dirty();

    ac.update_all_gui();

    Inhibit(true)
}

pub fn button_press_event(_da: &DrawingArea, event: &EventButton, ac: &AppContextPointer) -> Inhibit {
    // Left mouse button
    if event.get_button() == 1 {
        ac.skew_t.set_last_cursor_position(Some(event.get_position().into()));
        ac.skew_t.set_left_button_pressed(true);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

pub fn button_release_event(
    _da: &DrawingArea,
    event: &EventButton,
    ac: &AppContextPointer,
) -> Inhibit {
    if event.get_button() == 1 {
        ac.skew_t.set_last_cursor_position(None);
        ac.skew_t.set_left_button_pressed(false);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

pub fn leave_event(_da: &DrawingArea, _event: &EventCrossing, ac: &AppContextPointer) -> Inhibit {
    ac.skew_t.set_last_cursor_position(None);
    ac.set_sample(None);
    ac.update_all_gui();

    Inhibit(false)
}

pub fn mouse_motion_event(
    da: &DrawingArea,
    event: &EventMotion,
    ac: &AppContextPointer,
) -> Inhibit {
    da.grab_focus();

    if ac.skew_t.get_left_button_pressed() {
        if let Some(last_position) = ac.skew_t.get_last_cursor_position() {
            let old_position = ac.skew_t.convert_device_to_xy(last_position);
            let new_position = DeviceCoords::from(event.get_position());
            ac.skew_t.set_last_cursor_position(Some(new_position));

            let new_position = ac.skew_t.convert_device_to_xy(new_position);
            let delta = (
                new_position.x - old_position.x,
                new_position.y - old_position.y,
            );
            let mut translate = ac.skew_t.get_translate();
            translate.x -= delta.0;
            translate.y -= delta.1;
            ac.skew_t.set_translate(translate);
            ac.skew_t.bound_view();
            ac.skew_t.mark_background_dirty();
            ac.update_all_gui();

            ac.set_sample(None);
        }
    } else if ac.plottable() {
        let position: DeviceCoords = event.get_position().into();

        ac.skew_t.set_last_cursor_position(Some(position));
        let tp_position = ac.skew_t.convert_device_to_tp(position);
        let sample = ::sounding_analysis::linear_interpolate(
            &ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
            tp_position.pressure,
        );
        ac.set_sample(Some(sample));
        ac.skew_t.mark_overlay_dirty();
        ac.update_all_gui();
    }
    Inhibit(false)
}

pub fn key_release_event(_da: &DrawingArea, _event: &EventKey, _ac: &AppContextPointer) -> Inhibit {
    Inhibit(false)
}

pub fn key_press_event(_da: &DrawingArea, event: &EventKey, ac: &AppContextPointer) -> Inhibit {
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

pub fn size_allocate_event(da: &DrawingArea, _alloc: &Allocation, ac: &AppContextPointer) {
    ac.skew_t.update_cache_allocations(da);
}

pub fn configure_event(_da: &DrawingArea, event: &EventConfigure, ac: &AppContextPointer) -> bool {
    let rect = ac.skew_t.get_device_rect();
    let (width, height) = event.get_size();
    if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON
        || (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
    {
        ac.skew_t.mark_background_dirty();
    }
    false
}