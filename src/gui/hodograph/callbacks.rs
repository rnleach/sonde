use cairo::Context;

use gdk::{keyval_from_name, EventButton, EventConfigure, EventCrossing, EventKey, EventMotion,
          EventScroll, ScrollDirection};

use gtk::prelude::*;
use gtk::{Allocation, DrawingArea};

use app::AppContextPointer;
use coords::{DeviceCoords, XYCoords};
use gui::DrawingArgs;
use gui::plot_context::{Drawable, PlotContext, PlotContextExt};

pub fn draw_hodo(cr: &Context, acp: &AppContextPointer) -> Inhibit {
    let args = DrawingArgs::new(acp, cr);

    acp.hodo.init_matrix(args);
    acp.hodo.draw_background_cached(args);
    acp.hodo.draw_data_cached(args);
    acp.hodo.draw_overlay_cached(args);

    Inhibit(false)
}

/// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
pub fn scroll_event(_da: &DrawingArea, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
    const DELTA_SCALE: f64 = 1.05;
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 10.0;

    let pos = ac.hodo
        .convert_device_to_xy(DeviceCoords::from(event.get_position()));
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

    let mut translate = ac.hodo.get_translate();
    translate = XYCoords {
        x: pos.x - old_zoom / new_zoom * (pos.x - translate.x),
        y: pos.y - old_zoom / new_zoom * (pos.y - translate.y),
    };
    ac.hodo.set_translate(translate);
    ac.hodo.bound_view();
    ac.hodo.mark_background_dirty();

    ac.update_all_gui();

    Inhibit(true)
}

pub fn button_press_event(
    _da: &DrawingArea,
    event: &EventButton,
    ac: &AppContextPointer,
) -> Inhibit {
    // Left mouse button
    if event.get_button() == 1 {
        ac.hodo
            .set_last_cursor_position(Some(event.get_position().into()));
        ac.hodo.set_left_button_pressed(true);
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
        ac.hodo.set_last_cursor_position(None);
        ac.hodo.set_left_button_pressed(false);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

pub fn leave_event(_da: &DrawingArea, _event: &EventCrossing, ac: &AppContextPointer) -> Inhibit {
    ac.hodo.set_last_cursor_position(None);

    Inhibit(false)
}

pub fn mouse_motion_event(
    da: &DrawingArea,
    event: &EventMotion,
    ac: &AppContextPointer,
) -> Inhibit {
    da.grab_focus();

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

pub fn size_allocate_event(hodo_area: &DrawingArea, _alloc: &Allocation, ac: &AppContextPointer) {
    ac.hodo.update_cache_allocations(hodo_area);
}

pub fn configure_event(_da: &DrawingArea, event: &EventConfigure, ac: &AppContextPointer) -> bool {
    let rect = ac.hodo.get_device_rect();
    let (width, height) = event.get_size();
    if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON
        || (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
    {
        ac.hodo.mark_background_dirty();
    }
    false
}
