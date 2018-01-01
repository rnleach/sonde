use cairo::Context;

use gdk::{keyval_from_name, EventButton, EventConfigure, EventCrossing, EventKey, EventMotion,
          EventScroll, ScrollDirection};

use gtk::prelude::*;
use gtk::{Allocation, DrawingArea};

use app::AppContextPointer;
use coords::DeviceCoords;
use gui::DrawingArgs;
use gui::plot_context::PlotContext;

pub fn draw_rh_omega(cr: &Context, acp: &AppContextPointer) -> Inhibit {
    let args = DrawingArgs::new(acp, cr);

    acp.rh_omega.init_matrix(args);
    acp.rh_omega.draw_background_cached(args);
    acp.rh_omega.draw_data_cached(args);
    acp.rh_omega.draw_overlay_cached(args);

    Inhibit(false)
}

pub fn scroll_event(_da: &DrawingArea, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
    const DELTA_SCALE: f64 = 1.05;
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 10.0;

    let pos = ac.rh_omega
        .convert_device_to_xy(DeviceCoords::from(event.get_position()));
    let dir = event.get_direction();

    let old_zoom = ac.rh_omega.get_zoom_factor();
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
    ac.rh_omega.set_zoom_factor(new_zoom);

    let mut translate = ac.rh_omega.get_translate();
    translate.y = pos.y - old_zoom / new_zoom * (pos.y - translate.y);
    ac.rh_omega.set_translate(translate);
    ac.rh_omega.bound_view();
    ac.rh_omega.mark_background_dirty();

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
        ac.rh_omega
            .set_last_cursor_position(Some(event.get_position().into()));
        ac.rh_omega.set_left_button_pressed(true);
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
        ac.rh_omega.set_last_cursor_position(None);
        ac.rh_omega.set_left_button_pressed(false);
        Inhibit(true)
    } else {
        Inhibit(false)
    }
}

pub fn leave_event(_da: &DrawingArea, _event: &EventCrossing, ac: &AppContextPointer) -> Inhibit {
    ac.rh_omega.set_last_cursor_position(None);
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

    if ac.rh_omega.get_left_button_pressed() {
        if let Some(last_position) = ac.rh_omega.get_last_cursor_position() {
            let old_position = ac.rh_omega.convert_device_to_xy(last_position);
            let new_position = DeviceCoords::from(event.get_position());
            ac.rh_omega.set_last_cursor_position(Some(new_position));

            let new_position = ac.rh_omega.convert_device_to_xy(new_position);
            let delta = (
                new_position.x - old_position.x,
                new_position.y - old_position.y,
            );
            let mut translate = ac.rh_omega.get_translate();
            translate.y -= delta.1;
            ac.rh_omega.set_translate_y(translate);
            ac.rh_omega.bound_view();
            ac.rh_omega.mark_background_dirty();
            ac.update_all_gui();

            ac.set_sample(None);
        }
    } else if ac.plottable() {
        let position: DeviceCoords = event.get_position().into();

        ac.rh_omega.set_last_cursor_position(Some(position));
        let wp_position = ac.rh_omega.convert_device_to_wp(position);
        let sample = ::sounding_analysis::linear_interpolate(
            &ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
            wp_position.p,
        );
        ac.set_sample(Some(sample));
        ac.mark_overlay_dirty();
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
    ac.rh_omega.update_cache_allocations(da);
}

pub fn configure_event(_da: &DrawingArea, event: &EventConfigure, ac: &AppContextPointer) -> bool {
    let rect = ac.rh_omega.get_device_rect();
    let (width, height) = event.get_size();
    if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON
        || (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
    {
        ac.rh_omega.mark_background_dirty();
    }
    false
}
