//! Event callbacks.

use cairo::{Context, Matrix};
use gdk::{EventScroll, ScrollDirection};
use gtk::{DrawingArea, Inhibit, WidgetExt};

use super::{config, sounding_context, utility};

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_sounding(
    sounding_area: &DrawingArea,
    cr: &Context,
    sc: &sounding_context::SoundingContextPointer,
) -> Inhibit {

    let mut sc = sc.borrow_mut();

    // Get the dimensions of the DrawingArea
    let alloc = sounding_area.get_allocation();
    sc.device_width = alloc.width;
    sc.device_height = alloc.height;
    let aspect_ratio = sc.device_width as f64 / sc.device_height as f64;

    // Make coordinates x: 0 -> aspect_ratio and y: 0 -> 1.0
    cr.scale(sc.device_height as f64, sc.device_height as f64);
    // Set origin at lower right.
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0,
        x0: 0.0,
        y0: 1.0,
    });

    // Draw black backgound
    cr.rectangle(0.0, 0.0, aspect_ratio, 1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.fill();

    // Draw isentrops
    for theta in &config::ISENTROPS {
        utility::plot_curve_from_points(
            cr,
            &sc,
            1.0,
            (0.6, 0.6, 0.0, 0.5),
            utility::generate_isentrop(*theta),
        );
    }

    // Draw blue lines below freezing
    let mut end_points: Vec<_> = config::COLD_ISOTHERMS
        .into_iter()
        .map(|t| {
            ((*t, sounding_context::SoundingContext::MAXP), (
                *t,
                sounding_context::SoundingContext::MINP,
            ))
        })
        .collect();
    utility::plot_straight_lines(cr, &sc, 1.0, (0.0, 0.0, 1.0, 0.5), &end_points);

    // Draw red lines above freezing
    end_points = config::WARM_ISOTHERMS
        .into_iter()
        .map(|t| {
            ((*t, sounding_context::SoundingContext::MAXP), (
                *t,
                sounding_context::SoundingContext::MINP,
            ))
        })
        .collect();
    utility::plot_straight_lines(cr, &sc, 1.0, (1.0, 0.0, 0.0, 0.5), &end_points);

    // Draw pressure lines
    end_points = config::ISOBARS
        .into_iter()
        .map(|p| ((-150.0, *p), (60.0, *p)))
        .collect();
    utility::plot_straight_lines(cr, &sc, 1.0, (1.0, 1.0, 1.0, 0.5), &end_points);

    Inhibit(false)
}

/// Handles zooming from the mouse whell. Connected to the scroll-event signal.
pub fn scroll_event(
    sounding_area: &DrawingArea,
    event: &EventScroll,
    sc: &sounding_context::SoundingContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f32 = 1.05;
    const MIN_ZOOM: f32 = 0.75;
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
