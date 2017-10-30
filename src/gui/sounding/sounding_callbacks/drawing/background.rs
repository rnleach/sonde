use cairo::Context;

use app::AppContext;
use config;
use gui::sounding::sounding_callbacks::drawing::{plot_curve_from_points, plot_straight_dashed_lines, plot_straight_lines};

pub fn draw_background_fill(cr: &Context, ac: &AppContext) {

    const MAXP: f64 = config::MAXP;
    const MINP: f64 = config::MINP;

    // Banding for temperatures.
    // FIXME: Make my own function.
    let rgb = config::BACKGROUND_BAND_RGB;
    cr.set_source_rgb(rgb.0, rgb.1, rgb.2);
    let mut start_line = -160i32;
    while start_line < 100 {
        let t1 = start_line as f64;
        let t2 = t1 + 10.0;

        let mut coords = [(t1, MAXP), (t1, MINP), (t2, MINP), (t2, MAXP)];
        for coord in coords.iter_mut() {
            let f64_coord = (coord.0 as f64, coord.1 as f64);
            *coord = ac.convert_tp_to_screen(f64_coord);
        }
        cr.move_to(coords[0].0, coords[0].1);
        for i in 1..4 {
            cr.line_to(coords[i].0, coords[i].1);
        }
        cr.close_path();
        cr.fill();

        start_line += 20;
    }

    // Hail growth zone
    // FIXME: Make my own function
    let rgb = config::HAIL_ZONE_RGB;
    cr.set_source_rgb(rgb.0, rgb.1, rgb.2);
    let mut coords = [(-10.0, MAXP), (-10.0, MINP), (-30.0, MINP), (-30.0, MAXP)];
    for coord in coords.iter_mut() {
        let f64_coord = (coord.0 as f64, coord.1 as f64);
        *coord = ac.convert_tp_to_screen(f64_coord);
    }
    cr.move_to(coords[0].0, coords[0].1);
    for i in 1..4 {
        cr.line_to(coords[i].0, coords[i].1);
    }
    cr.close_path();
    cr.fill();

    // Dendritic snow growth zone
    // FIXME: Make my own function
    let rgb = config::DENDRTITIC_ZONE_RGB;
    cr.set_source_rgb(rgb.0, rgb.1, rgb.2);
    let mut coords = [(-12.0, MAXP), (-12.0, MINP), (-18.0, MINP), (-18.0, MAXP)];
    for coord in coords.iter_mut() {
        let f64_coord = (coord.0 as f64, coord.1 as f64);
        *coord = ac.convert_tp_to_screen(f64_coord);
    }
    cr.move_to(coords[0].0, coords[0].1);
    for i in 1..4 {
        cr.line_to(coords[i].0, coords[i].1);
    }
    cr.close_path();
    cr.fill();
}

// Draw isentrops, isotherms, isobars, ...
pub fn draw_background_lines(cr: &Context, ac: &AppContext) {
    // Draws background lines from the bottom up.

    // Draw isentrops
    for pnts in config::ISENTROP_PNTS.iter() {
        plot_curve_from_points(
            cr,
            &ac,
            config::BACKGROUND_LINE_WIDTH,
            config::ISENTROP_RGBA,
            pnts,
        );
    }

    // Draw theta-e lines
    for pnts in config::ISO_THETA_E_PNTS.iter() {
        plot_curve_from_points(
            cr,
            &ac,
            config::BACKGROUND_LINE_WIDTH,
            config::ISO_THETA_E_RGBA,
            pnts,
        );
    }

    // Draw mixing ratio lines
    plot_straight_dashed_lines(
        cr,
        &ac,
        config::BACKGROUND_LINE_WIDTH,
        config::ISO_MIXING_RATIO_RGBA,
        &config::ISO_MIXING_RATIO_PNTS,
    );

    // Draw isotherms
    plot_straight_lines(
        cr,
        &ac,
        config::BACKGROUND_LINE_WIDTH,
        config::ISOTHERM_RGBA,
        &config::ISOTHERM_PNTS,
    );

    // Draw isobars
    plot_straight_lines(
        cr,
        &ac,
        config::BACKGROUND_LINE_WIDTH,
        config::ISOBAR_RGBA,
        &config::ISOBAR_PNTS,
    );
}
