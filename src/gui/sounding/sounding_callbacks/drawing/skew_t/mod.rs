use cairo::Matrix;

use gui::plot_context::PlotContext;
use gui::DrawingArgs;

mod background;
mod labeling;
mod sample_readout;
mod temperature_profile;
mod wind_profile;

pub fn prepare_to_draw(args: DrawingArgs) {
    let ac = args.ac;
    let cr = args.cr;

    let scale_factor = ac.skew_t.scale_factor();

    cr.scale(scale_factor, scale_factor);

    // Set origin at lower left.
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0,
        x0: 0.0,
        y0: ac.skew_t.get_device_rect().height / scale_factor,
    });
}

pub fn draw_background(args: DrawingArgs) {
    background::draw_background_fill(args);
    background::draw_background_lines(args);
}

pub fn draw_temperature_profiles(args: DrawingArgs) {
    let config = args.ac.config.borrow();

    use self::temperature_profile::TemperatureType::{DewPoint, DryBulb, WetBulb};

    if config.show_wet_bulb {
        temperature_profile::draw_temperature_profile(WetBulb, args);
    }

    if config.show_dew_point {
        temperature_profile::draw_temperature_profile(DewPoint, args);
    }

    if config.show_temperature {
        temperature_profile::draw_temperature_profile(DryBulb, args);
    }
}

pub fn draw_wind_profile(args: DrawingArgs) {
    if args.ac.config.borrow().show_wind_profile {
        wind_profile::draw_wind_profile(args);
    }
}

pub fn draw_labels(args: DrawingArgs) {
    let config = args.ac.config.borrow();

    labeling::prepare_to_label(args);
    if config.show_labels {
        labeling::draw_background_labels(args);
    }
    if config.show_legend {
        labeling::draw_legend(args);
    }
}

pub fn draw_active_sample(args: DrawingArgs) {
    if args.ac.config.borrow().show_active_readout {
        sample_readout::draw_active_sample(args);
    }
}
