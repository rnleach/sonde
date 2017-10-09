//! Helper functions for the draw callback.

use cairo::{Context, Matrix};
use gtk::{DrawingArea, WidgetExt};

use config;
use app::data_context::DataContext;
use app::sounding_context::SoundingContext;
use gui::sounding::TPCoords;

// Prepare the drawing area with transforms, fill in the background, do the clipping
pub fn prepare_to_draw(sounding_area: &DrawingArea, cr: &Context, sc: &mut SoundingContext) {
    // Get the dimensions of the DrawingArea
    let alloc = sounding_area.get_allocation();
    sc.device_width = alloc.width;
    sc.device_height = alloc.height;
    let scale_factor = sc.scale_factor();

    // Fill with backgound color
    cr.rectangle(0.0, 0.0, sc.device_width as f64, sc.device_height as f64);
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
        y0: sc.device_height as f64 / scale_factor,
    });

    // Update the translation to center or bound the graph
    sc.bound_view();

    // Clip the drawing area
    let upper_right_xy = sc.convert_xy_to_screen((1.0, 1.0));
    let lower_left_xy = sc.convert_xy_to_screen((0.0, 0.0));
    cr.rectangle(
        lower_left_xy.0,
        lower_left_xy.1,
        upper_right_xy.0 - lower_left_xy.0,
        upper_right_xy.1 - lower_left_xy.1,
    );
    cr.clip();
}

// Draw isentrops, isotherms, isobars, ...
pub fn draw_background_lines(cr: &Context, sc: &SoundingContext) {
    // Draws background lines from the bottom up.

    // Draw isentrops
    for theta in &config::ISENTROPS {
        plot_curve_from_points(
            cr,
            &sc,
            config::BACKGROUND_LINE_WIDTH,
            config::ISENTROP_RGBA,
            generate_isentrop(*theta),
        );
    }

    // Draw freezing and below isotherms
    let mut end_points: Vec<_> = config::COLD_ISOTHERMS
        .into_iter()
        .map(|t| ((*t, config::MAXP), (*t, config::MINP)))
        .collect();
    plot_straight_lines(
        cr,
        &sc,
        config::BACKGROUND_LINE_WIDTH,
        config::COLD_ISOTHERM_RGBA,
        &end_points,
    );

    // Draw above freezing isotherms
    end_points = config::WARM_ISOTHERMS
        .into_iter()
        .map(|t| ((*t, config::MAXP), (*t, config::MINP)))
        .collect();
    plot_straight_lines(
        cr,
        &sc,
        config::BACKGROUND_LINE_WIDTH,
        config::WARM_ISOTHERM_RGBA,
        &end_points,
    );

    // Draw isobars
    end_points = config::ISOBARS
        .into_iter()
        .map(|p| ((-150.0, *p), (60.0, *p)))
        .collect();
    plot_straight_lines(
        cr,
        &sc,
        config::BACKGROUND_LINE_WIDTH,
        config::ISOBAR_RGBA,
        &end_points,
    );
}

pub enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
}

// Draw the temperature profile
#[inline]
pub fn draw_temperature_profile(
    t_type: TemperatureType,
    cr: &Context,
    sc: &SoundingContext,
    dc: &DataContext,
) {

    if let Some(sndg) = dc.get_sounding_for_display() {

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

        plot_curve_from_points(cr, &sc, line_width, line_rgba, profile_data);
    }
}

/// Draw a straight line on the graph.
#[inline]
pub fn plot_straight_lines(
    cr: &Context,
    sc: &SoundingContext,
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
    sc: &SoundingContext,
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
    use config::{MAXP, ISENTROPS_TOP_P, POINTS_PER_ISENTROP};
    const P0: f32 = 1000.0; // For calcuating theta

    let mut result = vec![];

    let mut p = MAXP;
    while p >= ISENTROPS_TOP_P {
        let t = theta * f32::powf(P0 / p, -0.286) - 273.15;
        result.push((t, p));
        p += (ISENTROPS_TOP_P - MAXP) / (POINTS_PER_ISENTROP as f32);
    }

    result
}
