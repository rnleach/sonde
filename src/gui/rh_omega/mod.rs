use gui::DrawingArgs;
use gtk::DrawingArea;
use gtk::prelude::*;

use app::config;
use coords::{DeviceCoords, DeviceRect};
use gui::PlotContext;

pub mod rh_omega_context;
mod rh_omega;

fn draw_rh_omega(args: DrawingArgs) {
    rh_omega::prepare_to_draw(args);
    rh_omega::draw_background(args);
    rh_omega::draw_rh_profile(args);
    rh_omega::draw_omega_profile(args);
    rh_omega::draw_active_readout(args);
}

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
    // Draw the RH-Omega plot area
    //

    // Clip the drawing area
    cr.clip();

    // Set the drawing area in the plot context
    let rh_omega_rect = DeviceRect {
        upper_left: DeviceCoords { col: 0.0, row: 0.0 },
        width: width,
        height: height,
    };
    ac.rh_omega.set_device_rect(rh_omega_rect);

    draw_rh_omega(args);
}