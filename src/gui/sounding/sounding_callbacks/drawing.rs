//! Helper functions for the draw callback.

use cairo::{Context, Matrix};
use gtk::{DrawingArea, WidgetExt};

use config;
use app::AppContext;
use coords::TPCoords;

// Prepare the drawing area with transforms, fill in the background, do the clipping
pub fn prepare_to_draw(sounding_area: &DrawingArea, cr: &Context, ac: &mut AppContext) {
    // Get the dimensions of the DrawingArea
    let alloc = sounding_area.get_allocation();
    ac.device_width = alloc.width;
    ac.device_height = alloc.height;
    let scale_factor = ac.scale_factor();

    // Fill with backgound color
    cr.rectangle(0.0, 0.0, ac.device_width as f64, ac.device_height as f64);
    cr.set_source_rgb(
        config::BACKGROUND_RGB.0,
        config::BACKGROUND_RGB.1,
        config::BACKGROUND_RGB.2,
    );
    cr.fill();

    // Set the scale factor
    cr.scale(scale_factor, scale_factor);
    // Set origin at lower left.
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0,
        x0: 0.0,
        y0: ac.device_height as f64 / scale_factor,
    });

    // Update the translation to center or bound the graph
    ac.bound_view();

    // Clip the drawing area
    let upper_right_xy = ac.convert_xy_to_screen((1.0, 1.0));
    let lower_left_xy = ac.convert_xy_to_screen((0.0, 0.0));
    cr.rectangle(
        lower_left_xy.0,
        lower_left_xy.1,
        upper_right_xy.0 - lower_left_xy.0,
        upper_right_xy.1 - lower_left_xy.1,
    );
    cr.clip();
}

// Draw background fills and patterns
pub fn draw_background_fill(cr: &Context, ac: &AppContext) {

    const MAXP: f64 = config::MAXP as f64;
    const MINP: f64 = config::MINP as f64;

    // Banding for temperatures.
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

pub enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
}

// Draw the temperature profile
#[inline]
pub fn draw_temperature_profile(t_type: TemperatureType, cr: &Context, ac: &AppContext) {

    if let Some(sndg) = ac.get_sounding_for_display() {

        let pres_data = &sndg.pressure;
        let temp_data = match t_type {
            TemperatureType::DryBulb => &sndg.temperature,
            TemperatureType::WetBulb => &sndg.wet_bulb,
            TemperatureType::DewPoint => &sndg.dew_point,
        };

        let line_width = match t_type {
            TemperatureType::DryBulb => config::TEMPERATURE_LINE_WIDTH,
            TemperatureType::WetBulb => config::WET_BULB_LINE_WIDTH,
            TemperatureType::DewPoint => config::DEW_POINT_LINE_WIDTH,
        };

        let line_rgba = match t_type {
            TemperatureType::DryBulb => config::TEMPERATURE_RGBA,
            TemperatureType::WetBulb => config::WET_BULB_RGBA,
            TemperatureType::DewPoint => config::DEW_POINT_RGBA,
        };

        let profile_data: Vec<_> = pres_data
            .iter()
            .zip(temp_data.iter())
            .filter_map(|val_pair| if let (Some(p), Some(t)) =
                (val_pair.0.as_option(), val_pair.1.as_option())
            {
                if p > config::MINP { Some((t, p)) } else { None }
            } else {
                None
            })
            .collect();

        plot_curve_from_points(cr, &ac, line_width, line_rgba, &profile_data);
    }
}

/// Draw a straight line on the graph.
#[inline]
pub fn plot_straight_lines(
    cr: &Context,
    ac: &AppContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    end_points: &[(TPCoords, TPCoords)],
) {
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);
    for &(start, end) in end_points {
        let start = ac.convert_tp_to_screen(start);
        let end = ac.convert_tp_to_screen(end);
        cr.move_to(start.0, start.1);
        cr.line_to(end.0, end.1);
    }
    cr.stroke();
}

/// Draw a straight line on the graph.
#[inline]
pub fn plot_straight_dashed_lines(
    cr: &Context,
    ac: &AppContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    end_points: &[(TPCoords, TPCoords)],
) {
    cr.set_dash(&[0.02], 0.0);
    plot_straight_lines(cr, ac, line_width_pixels, rgba, end_points);
    cr.set_dash(&[], 0.0);
}

/// Draw a curve connecting a list of points.
pub fn plot_curve_from_points(
    cr: &Context,
    ac: &AppContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: &[TPCoords],
) {

    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);

    let mut iter = points.iter();
    let start = ac.convert_tp_to_screen(*iter.by_ref().next().unwrap());
    cr.move_to(start.0, start.1);
    for end in iter {
        let end = ac.convert_tp_to_screen(*end);
        cr.line_to(end.0, end.1);
    }

    cr.stroke();
}
