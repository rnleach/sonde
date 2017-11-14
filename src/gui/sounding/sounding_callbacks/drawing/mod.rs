//! Helper functions for the draw callback.

use cairo::{Context, Matrix};

use app::AppContext;
use coords::XYCoords;

mod background;
mod labeling;
mod sample_readout;
mod temperature_profile;
mod wind_profile;

pub fn prepare_to_draw(cr: &Context, ac: &mut AppContext) {
    use app::PlotContext;

    // Get the dimensions of the DrawingArea
    ac.update_plot_context_allocations();
    let scale_factor = ac.skew_t.scale_factor();

    // Fill with backgound color
    cr.rectangle(
        0.0,
        0.0,
        f64::from(ac.skew_t.device_width),
        f64::from(ac.skew_t.device_height),
    );
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
        y0: f64::from(ac.skew_t.device_height) / scale_factor,
    });

    // Clip the drawing area
    let upper_right_xy = ac.skew_t.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });
    let lower_left_xy = ac.skew_t.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
    cr.rectangle(
        lower_left_xy.x,
        lower_left_xy.y,
        upper_right_xy.x - lower_left_xy.x,
        upper_right_xy.y - lower_left_xy.y,
    );
    cr.clip();

    // Calculate the various padding values
    ac.skew_t.label_padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
    ac.skew_t.edge_padding = cr.device_to_user_distance(ac.config.edge_padding, 0.0).0;

    // Bound the xy-coords to always be on screen.
    ac.bound_view();
}

pub fn draw_background(cr: &Context, ac: &AppContext) {
    background::draw_background_fill(cr, ac);
    background::draw_background_lines(cr, ac);
}

pub fn draw_temperature_profiles(cr: &Context, ac: &AppContext) {
    use self::temperature_profile::TemperatureType::{DewPoint, DryBulb, WetBulb};

    if ac.config.show_wet_bulb {
        temperature_profile::draw_temperature_profile(WetBulb, cr, ac);
    }

    if ac.config.show_dew_point {
        temperature_profile::draw_temperature_profile(DewPoint, cr, ac);
    }

    if ac.config.show_temperature {
        temperature_profile::draw_temperature_profile(DryBulb, cr, ac);
    }
}

pub fn draw_wind_profile(cr: &Context, ac: &AppContext) {
    if ac.config.show_wind_profile {
        wind_profile::draw_wind_profile(cr, ac);
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

pub fn draw_active_sample(cr: &Context, ac: &mut AppContext) {
    if ac.config.show_active_readout {
        sample_readout::draw_active_sample(cr, ac);
    }
}
