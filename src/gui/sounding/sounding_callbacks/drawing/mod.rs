//! Helper functions for the draw callback.

use cairo::{Context, Matrix};
use gtk::{DrawingArea, WidgetExt};

use app::AppContext;
use coords::{ScreenCoords, XYCoords};

mod background;
mod labeling;
mod sample_readout;
mod temperature_profile;
mod wind_profile;
mod omega_profile;

pub fn prepare_to_draw(sounding_area: &DrawingArea, cr: &Context, ac: &mut AppContext) {
    // Get the dimensions of the DrawingArea
    let alloc = sounding_area.get_allocation();
    ac.device_width = alloc.width;
    ac.device_height = alloc.height;
    let scale_factor = ac.scale_factor();

    // Fill with backgound color
    cr.rectangle(0.0, 0.0, ac.device_width as f64, ac.device_height as f64);
    cr.set_source_rgba(
        ac.config.background_rgba.0,
        ac.config.background_rgba.1,
        ac.config.background_rgba.2,
        ac.config.background_rgba.3,
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
    ac.label_padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
    ac.edge_padding = cr.device_to_user_distance(ac.config.edge_padding, 0.0).0;
}

pub fn draw_background(cr: &Context, ac: &AppContext) {
    background::draw_background_fill(&cr, &ac);
    background::draw_background_lines(&cr, &ac);
}

pub fn draw_temperature_profiles(cr: &Context, ac: &AppContext) {
    use self::temperature_profile::TemperatureType::{DewPoint, DryBulb, WetBulb};

    if ac.config.show_wet_bulb {
        temperature_profile::draw_temperature_profile(WetBulb, &cr, &ac);
    }

    if ac.config.show_dew_point {
        temperature_profile::draw_temperature_profile(DewPoint, &cr, &ac);
    }

    if ac.config.show_temperature {
        temperature_profile::draw_temperature_profile(DryBulb, &cr, &ac);
    }
}

pub fn draw_wind_profile(cr: &Context, ac: &AppContext) {
    if ac.config.show_wind_profile {
        wind_profile::draw_wind_profile(cr, ac);
    }
}

pub fn draw_omega_profile(cr: &Context, ac: &AppContext) {

    if ac.config.show_omega {
        omega_profile::draw_omega_profile(cr, ac);
    }
}

pub fn draw_labels(cr: &Context, ac: &AppContext) {
    labeling::prepare_to_label(cr, ac);

    if ac.config.show_labels {
        labeling::draw_background_labels(cr, ac);
    }
    if ac.config.show_legend {
        labeling::draw_legend(cr, ac);
    }
}

pub fn draw_active_sample(cr: &Context, ac: &AppContext) {
    if ac.config.show_active_readout {
        sample_readout::draw_active_sample(cr, ac);
    }
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
