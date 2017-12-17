//! Helper functions for the draw callback.

use gtk::prelude::*;
use gtk::DrawingArea;

use app::config;
use coords::{DeviceCoords, DeviceRect};
use gui::plot_context::PlotContext;
use gui::DrawingArgs;

mod skew_t;
mod rh_omega;

pub fn draw(args: DrawingArgs, da: &DrawingArea) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    let alloc = da.get_allocation();
    let (width, height) = (f64::from(alloc.width), f64::from(alloc.height));

    // Fill with backgound color
    cr.rectangle(0.0, 0.0, width, height);
    cr.set_source_rgba(
        config.background_rgba.0,
        config.background_rgba.1,
        config.background_rgba.2,
        config.background_rgba.3,
    );
    cr.fill();

    cr.save();

    //
    // Draw the Skew-T area.
    //

    // Clip the drawing area
    let rh_width = width * config::RH_OMEGA_WIDTH;
    cr.rectangle(rh_width, 0.0, width - rh_width, height);
    cr.clip();

    // Set the drawing area in the plot context
    let skew_t_rect = DeviceRect {
        upper_left: DeviceCoords {
            col: rh_width,
            row: 0.0,
        },
        width: width - rh_width,
        height: height,
    };
    ac.skew_t.set_device_rect(skew_t_rect);
    cr.translate(rh_width, 0.0);
    draw_skew_t(args);

    // Reset the matrix when done for the next sub-plot
    cr.restore();

    //
    // Draw the RH-Omega plot area
    //

    // Clip the drawing area
    cr.rectangle(0.0, 0.0, rh_width, height);
    cr.clip();

    // Set the drawing area in the plot context
    let rh_omega_rect = DeviceRect {
        upper_left: DeviceCoords { col: 0.0, row: 0.0 },
        width: rh_width,
        height: height,
    };
    ac.rh_omega.set_device_rect(rh_omega_rect);

    draw_rh_omega(args);
}

fn draw_skew_t(args: DrawingArgs) {

    skew_t::prepare_to_draw(args);
    skew_t::draw_background(args);
    skew_t::draw_labels(args);
    skew_t::draw_temperature_profiles(args);
    skew_t::draw_wind_profile(args);
    skew_t::draw_active_sample(args);
}

fn draw_rh_omega(args: DrawingArgs) {
    rh_omega::prepare_to_draw(args);
    rh_omega::draw_background(args);
    rh_omega::draw_rh_profile(args);
    rh_omega::draw_omega_profile(args);
    rh_omega::draw_active_readout(args);
}
