
use cairo::{Context, Matrix};

use gtk::prelude::*;
use gtk::DrawingArea;

use app::{AppContext, config};
use coords::{SDCoords, XYCoords, DeviceCoords, DeviceRect};
use gui::{PlotContext, plot_curve_from_points};

pub fn prepare_to_draw_hodo(da: &DrawingArea, cr: &Context, ac: &AppContext) {
    use gui::plot_context::PlotContext;

    let alloc = da.get_allocation();
    let device_rect = DeviceRect {
        upper_left: DeviceCoords { row: 0.0, col: 0.0 },
        width: f64::from(alloc.width),
        height: f64::from(alloc.height),
    };
    ac.hodo.set_device_rect(device_rect);
    let scale_factor = ac.hodo.scale_factor();

    let config = ac.config.borrow();

    // Fill with backgound color
    cr.rectangle(0.0, 0.0, device_rect.width, device_rect.height);
    cr.set_source_rgba(
        config.background_rgba.0,
        config.background_rgba.1,
        config.background_rgba.2,
        config.background_rgba.3,
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
        y0: device_rect.height / scale_factor,
    });

    // Clip the drawing area
    let upper_right_xy = ac.hodo.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });
    let lower_left_xy = ac.hodo.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
    cr.rectangle(
        lower_left_xy.x,
        lower_left_xy.y,
        upper_right_xy.x - lower_left_xy.x,
        upper_right_xy.y - lower_left_xy.y,
    );
    cr.clip();

    ac.hodo.bound_view();
}

pub fn draw_hodo_background(cr: &Context, ac: &AppContext) {

    let config = ac.config.borrow();

    if config.show_background_bands {
        draw_background_fill(cr, ac);
    }

    if config.show_iso_speed {
        draw_background_lines(cr, ac);
    }
}

fn draw_background_fill(cr: &Context, ac: &AppContext) {

    let mut do_draw = true;
    let rgba = ac.config.borrow().background_band_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

    for pnts in config::ISO_SPEED_PNTS.iter() {
        let mut pnts = pnts.iter().map(|xy_coords| {
            ac.hodo.convert_xy_to_screen(*xy_coords)
        });

        if let Some(pnt) = pnts.by_ref().next() {
            cr.move_to(pnt.x, pnt.y);
        }
        if do_draw {
            for pnt in pnts {
                cr.line_to(pnt.x, pnt.y);
            }
        } else {
            for pnt in pnts.rev() {
                cr.line_to(pnt.x, pnt.y);
            }
        }
        cr.close_path();
        if do_draw {
            cr.fill();
        }
        do_draw = !do_draw;
    }
}

fn draw_background_lines(cr: &Context, ac: &AppContext) {

    let config = ac.config.borrow();

    for pnts in config::ISO_SPEED_PNTS.iter() {
        let pnts = pnts.iter().map(|xy_coords| {
            ac.hodo.convert_xy_to_screen(*xy_coords)
        });
        plot_curve_from_points(
            cr,
            config.background_line_width,
            config.iso_speed_rgba,
            pnts,
        );
    }
}

pub fn draw_hodo_labels(_cr: &Context, ac: &AppContext) {
    let config = ac.config.borrow();

    if config.show_labels {
        // TODO:
    }
}

pub fn draw_hodo_line(cr: &Context, ac: &AppContext) {

    use sounding_base::Profile::{Pressure, WindSpeed, WindDirection};

    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {

        let pres_data = sndg.get_profile(Pressure);
        let speed_data = sndg.get_profile(WindSpeed);
        let dir_data = sndg.get_profile(WindDirection);

        let profile_data = izip!(pres_data, speed_data, dir_data).filter_map(
            |triplet| {
                if let (Some(p), Some(speed), Some(dir)) =
                    (
                        triplet.0.as_option(),
                        triplet.1.as_option(),
                        triplet.2.as_option(),
                    )
                {
                    if p > config::MINP {
                        let sd_coords = SDCoords { speed, dir };
                        Some(ac.hodo.convert_sd_to_screen(sd_coords))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        );

        plot_curve_from_points(
            cr,
            config.velocity_line_width,
            config.veclocity_rgba,
            profile_data,
        );
    }
}

pub fn draw_active_readout(cr: &Context, ac: &AppContext) {
    let config = ac.config.borrow();

    if !config.show_active_readout {
        return;
    }

    let (speed, dir) = if let Some(sample) = ac.get_sample() {
        if let (Some(speed), Some(dir)) = (sample.speed.as_option(), sample.direction.as_option()) {
            (speed, dir)
        } else {
            return;
        }
    } else {
        return;
    };

    let pnt_size = cr.device_to_user_distance(5.0, 0.0).0;
    let coords = ac.hodo.convert_sd_to_screen(SDCoords { speed, dir });

    let rgba = config.active_readout_line_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.arc(
        coords.x,
        coords.y,
        pnt_size,
        0.0,
        2.0 * ::std::f64::consts::PI,
    );
    cr.fill();
}
