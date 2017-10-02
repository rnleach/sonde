//! Several utility routines to help with plotting soundings.

use cairo::Context;

use super::{TPCoords, sounding_context};

/// Draw a straight line on the graph.
#[inline]
pub fn plot_straight_lines(
    cr: &Context,
    sc: &sounding_context::SoundingContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    end_points: &[(TPCoords, TPCoords)],
) {
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);
    for &(start, end) in end_points {
        let start = sc.convert_tp_to_screen(start);
        let end = sc.convert_tp_to_screen(end);
        cr.move_to(start.0, start.1);
        cr.line_to(end.0, end.1);
    }
    cr.stroke();
}

/// Draw a curve connecting a list of points.
pub fn plot_curve_from_points(
    cr: &Context,
    sc: &sounding_context::SoundingContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: Vec<TPCoords>,
) {
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);

    let mut iter = points.into_iter();
    let start = sc.convert_tp_to_screen(iter.by_ref().next().unwrap());
    cr.move_to(start.0, start.1);
    for end in iter {
        let end = sc.convert_tp_to_screen(end);
        cr.line_to(end.0, end.1);
    }

    cr.stroke();
}

/// Generate a list of Temperature, Pressure points along an isentrope.
pub fn generate_isentrop(theta: f32) -> Vec<TPCoords> {
    use std::f32;
    const P0: f32 = 1000.0; // For calcuating theta
    const PS: f32 = sounding_context::SoundingContext::MAXP;
    const PF: f32 = 300.0; // TODO: move to config?
    const NUM_PNTS: u32 = 30; // TODO: move to config?

    let mut result = vec![];

    let mut p = sounding_context::SoundingContext::MAXP;
    while p >= 300.0 {
        let t = theta * f32::powf(P0 / p, -0.286) - 273.15;
        result.push((t, p));
        p += (PF - PS) / (NUM_PNTS as f32);
    }

    result
}
