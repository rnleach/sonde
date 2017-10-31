//! Helper functions for the draw callback.

use cairo::{Context, Matrix};
use gtk::{DrawingArea, WidgetExt};

use app::AppContext;
use config;
use coords::{TPCoords, XYCoords};

mod background;
mod labeling;
mod sample_readout;
mod temperature_profile;
mod wind_profile;

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
    let upper_right_xy = ac.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });
    let lower_left_xy = ac.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
    cr.rectangle(
        lower_left_xy.x,
        lower_left_xy.y,
        upper_right_xy.x - lower_left_xy.x,
        upper_right_xy.y - lower_left_xy.y,
    );
    cr.clip();

    // Calculate the various padding values
    ac.label_padding = cr.device_to_user_distance(config::LABEL_PADDING, 0.0).0;
    ac.edge_padding = cr.device_to_user_distance(config::EDGE_PADDING, 0.0).0;
}

pub fn draw_background(cr: &Context, ac: &AppContext) {
    background::draw_background_fill(&cr, &ac);
    background::draw_background_lines(&cr, &ac);
}

pub fn draw_temperature_profiles(cr: &Context, ac: &AppContext) {
    use self::temperature_profile::TemperatureType::{DewPoint, DryBulb, WetBulb};

    temperature_profile::draw_temperature_profile(WetBulb, &cr, &ac);
    temperature_profile::draw_temperature_profile(DewPoint, &cr, &ac);
    temperature_profile::draw_temperature_profile(DryBulb, &cr, &ac);
}

pub fn draw_wind_profile(cr: &Context, ac: &AppContext) {
    wind_profile::draw_wind_profile(cr, ac);
}

pub fn draw_labels(cr: &Context, ac: &AppContext) {
    labeling::prepare_to_label(cr, ac);
    labeling::draw_background_labels(cr, ac);
    labeling::draw_legend(cr, ac);
}

pub fn draw_active_sample(cr: &Context, ac: &AppContext) {
    sample_readout::draw_active_sample(cr, ac);
}

// Draw a straight line on the graph.
fn plot_straight_lines(
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
        cr.move_to(start.x, start.y);
        cr.line_to(end.x, end.y);
    }
    cr.stroke();
}

// Draw a straight line on the graph.
fn plot_straight_dashed_lines(
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

// Draw a curve connecting a list of points.
fn plot_curve_from_points(
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
    cr.move_to(start.x, start.y);
    for end in iter {
        let end = ac.convert_tp_to_screen(*end);
        cr.line_to(end.x, end.y);
    }

    cr.stroke();
}
