
use cairo::{Context, Matrix};
use gtk::{DrawingArea, Inhibit, WidgetExt};

use app::{AppContext, AppContextPointer, config};
use coords::{XYCoords, WPCoords, TPCoords};
use gui::sounding::plot_curve_from_points;

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_omega(omega_area: &DrawingArea, cr: &Context, ac: &AppContextPointer) -> Inhibit {

    let mut ac = ac.borrow_mut();

    prepare_to_draw(omega_area, cr, &mut ac);
    draw_background(cr, &mut ac);
    draw_labels(cr, &ac);
    draw_rh_profile(cr, &ac);
    draw_omega_profile(cr, &ac);
    draw_active_readout(cr, &ac);

    Inhibit(false)
}

fn prepare_to_draw(omega_area: &DrawingArea, cr: &Context, ac: &mut AppContext) {
    // Get the dimensions of the DrawingArea
    let alloc = omega_area.get_allocation();
    ac.rh_omega.device_width = alloc.width;
    ac.rh_omega.device_height = alloc.height;

    ac.update_skew_t_allocation();
    let scale_factor = ac.skew_t.scale_factor();
    ac.rh_omega.skew_t_scale_factor = scale_factor;

    // Fill with backgound color
    cr.rectangle(
        0.0,
        0.0,
        ac.rh_omega.device_width as f64,
        ac.rh_omega.device_height as f64,
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
        y0: ac.rh_omega.device_height as f64 / scale_factor,
    });

    // Clip the drawing area
    let upper_right_xy = ac.rh_omega.convert_xy_to_screen(
        XYCoords { x: 1.0, y: 1.0 },
    );
    let lower_left_xy = ac.rh_omega.convert_xy_to_screen(
        XYCoords { x: 0.0, y: 0.0 },
    );
    cr.rectangle(
        lower_left_xy.x,
        lower_left_xy.y,
        upper_right_xy.x - lower_left_xy.x,
        upper_right_xy.y - lower_left_xy.y,
    );
    cr.clip();
}

fn draw_background(cr: &Context, ac: &mut AppContext) {

    // If is plottable, draw snow growth zones
    // TODO:

    // Draw isobars
    if ac.config.show_isobars {
        for pnts in config::ISOBAR_PNTS.iter() {
            let TPCoords { pressure: p, .. } = pnts[0];

            let pnts = [
                WPCoords {
                    w: -ac.rh_omega.get_max_abs_omega(),
                    p,
                },
                WPCoords {
                    w: ac.rh_omega.get_max_abs_omega(),
                    p,
                },
            ];
            let pnts = pnts.iter().map(|wp_coords| {
                ac.rh_omega.convert_wp_to_screen(*wp_coords)
            });
            plot_curve_from_points(
                cr,
                ac.config.background_line_width,
                ac.config.isobar_rgba,
                pnts,
            );
        }
    }

    // Draw w-lines
    for v_line in config::ISO_OMEGA_PNTS.iter() {

        plot_curve_from_points(
            cr,
            ac.config.background_line_width,
            ac.config.isobar_rgba,
            v_line.iter().map(|wp_coords| {
                ac.rh_omega.convert_wp_to_screen(*wp_coords)
            }),
        );
    }
}

fn draw_labels(cr: &Context, ac: &AppContext) {
    // TODO:
}

fn draw_omega_profile(cr: &Context, ac: &AppContext) {

    if !ac.config.show_omega_profile {
        return;
    }

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{Pressure, PressureVerticalVelocity};

        let pres_data = sndg.get_profile(Pressure);
        let omega_data = sndg.get_profile(PressureVerticalVelocity);
        let line_width = ac.config.omega_line_width;
        let line_rgba = ac.config.omega_rgba;

        let profile_data = pres_data.iter().zip(omega_data.iter()).filter_map(
            |val_pair| {
                if let (Some(p), Some(w)) = (val_pair.0.as_option(), val_pair.1.as_option()) {
                    if p > config::MINP {
                        let wp_coords = WPCoords { w, p };
                        Some(ac.rh_omega.convert_wp_to_screen(wp_coords))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        );

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    }
}

fn draw_rh_profile(cr: &Context, ac: &AppContext) {
    // TODO:
}

fn draw_active_readout(cr: &Context, ac: &AppContext) {
    if let Some(sample_p) = ac.last_sample_pressure {

        let rgba = ac.config.active_readout_line_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(
            cr.device_to_user_distance(ac.config.active_readout_line_width, 0.0)
                .0,
        );
        let start = ac.rh_omega.convert_wp_to_screen(WPCoords {
            w: -1000.0,
            p: sample_p,
        });
        let end = ac.rh_omega.convert_wp_to_screen(WPCoords {
            w: 1000.0,
            p: sample_p,
        });
        cr.move_to(start.x, start.y);
        cr.line_to(end.x, end.y);
        cr.stroke();
    }
}
