//! Module holds the code for drawing the skew-t plot.

use std::rc::Rc;

use cairo::{Context, Matrix};
use gdk::{SCROLL_MASK, BUTTON_PRESS_MASK, BUTTON_RELEASE_MASK, POINTER_MOTION_MASK,
          POINTER_MOTION_HINT_MASK, LEAVE_NOTIFY_MASK, KEY_RELEASE_MASK, KEY_PRESS_MASK};
use gtk::{DrawingArea, WidgetExt};

mod sounding_callbacks;
mod omega_callbacks;

use app;
use coords::{ScreenCoords, DeviceCoords, ScreenRect};

/// Initialize the drawing area and connect signal handlers.
pub fn set_up_sounding_area(sounding_area: &DrawingArea, app_context: &app::AppContextPointer) {

    // Layout
    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    let ac = Rc::clone(app_context);
    sounding_area.connect_draw(move |_da, cr| sounding_callbacks::draw_sounding(cr, &ac));

    let ac = Rc::clone(app_context);
    sounding_area.connect_scroll_event(move |da, ev| sounding_callbacks::scroll_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    sounding_area.connect_button_press_event(move |da, ev| {
        sounding_callbacks::button_press_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_button_release_event(move |da, ev| {
        sounding_callbacks::button_release_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_motion_notify_event(move |da, ev| {
        sounding_callbacks::mouse_motion_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_leave_notify_event(move |da, ev| {
        sounding_callbacks::leave_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_key_release_event(move |da, ev| {
        sounding_callbacks::key_release_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    sounding_area.connect_key_press_event(move |da, ev| {
        sounding_callbacks::key_press_event(da, ev, &ac)
    });
    sounding_area.set_can_focus(true);

    sounding_area.add_events(
        (SCROLL_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK | POINTER_MOTION_HINT_MASK |
             POINTER_MOTION_MASK |
             LEAVE_NOTIFY_MASK | KEY_RELEASE_MASK | KEY_PRESS_MASK)
            .bits() as i32,
    );

}

pub fn set_up_omega_area(omega_area: &DrawingArea, app_context: &app::AppContextPointer) {

    // Layout
    omega_area.set_hexpand(false);
    omega_area.set_vexpand(true);
    omega_area.set_property_width_request(80);

    let acp = Rc::clone(app_context);
    omega_area.connect_draw(move |da, cr| omega_callbacks::draw_omega(da, cr, &acp));
}

// Draw a curve connecting a list of points.
fn plot_curve_from_points<I>(
    cr: &Context,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: I,
) where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);

    let mut points = points;
    if let Some(start) = points.by_ref().next() {
        cr.move_to(start.x, start.y);
        for end in points {
            cr.line_to(end.x, end.y);
        }

        cr.stroke();
    }
}

// Draw a dashed line on the graph.
fn plot_dashed_curve_from_points<I>(
    cr: &Context,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: I,
) where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_dash(&[0.02], 0.0);
    plot_curve_from_points(cr, line_width_pixels, rgba, points);
    cr.set_dash(&[], 0.0);
}

fn set_font_size(size_in_pnts: f64, cr: &Context, ac: &app::AppContext) {
    use app::PlotContext;

    let dpi = match ac.get_dpi() {
        None => 72.0,
        Some(value) => value,
    };

    let font_size = size_in_pnts / 72.0 * dpi;
    let ScreenCoords { x: font_size, .. } = ac.skew_t.convert_device_to_screen(DeviceCoords {
        col: font_size,
        row: 0.0,
    });

    // Flip the y-coordinate so it displays the font right side up
    cr.set_font_matrix(Matrix {
        xx: 1.0 * font_size,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0 * font_size, // Reflect it to be right side up!
        x0: 0.0,
        y0: 0.0,
    });
}

fn check_overlap_then_add(
    cr: &Context,
    ac: &app::AppContext,
    vector: &mut Vec<(String, ScreenRect)>,
    plot_edges: &ScreenRect,
    label_pair: (String, ScreenRect),
) {
    use coords::Rect;

    let padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
    let padded_rect = label_pair.1.add_padding(padding);

    // Make sure it is on screen - but don't add padding to this check cause the screen already
    // has padding.
    if !label_pair.1.inside(plot_edges) {
        return;
    }

    // Check for overlap
    for &(_, ref rect) in vector.iter() {
        if padded_rect.overlaps(rect) {
            return;
        }
    }

    vector.push(label_pair);
}
