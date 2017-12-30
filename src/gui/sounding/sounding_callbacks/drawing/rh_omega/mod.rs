use cairo::Matrix;

use app::config;
use coords::{Rect, ScreenCoords, WPCoords};
use gui::{plot_curve_from_points, DrawingArgs, PlotContext};

mod background;

pub fn prepare_to_draw(args: DrawingArgs) {
    let ac = args.ac;
    let cr = args.cr;

    let scale_factor = ac.rh_omega.scale_factor();
    ac.rh_omega.set_zoom_factor(ac.skew_t.get_zoom_factor());
    ac.rh_omega.set_translate_y(ac.skew_t.get_translate());
    ac.rh_omega.set_skew_t_scale(ac.skew_t.scale_factor());

    cr.scale(scale_factor, scale_factor);

    // Set origin at lower left.
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0,
        x0: 0.0,
        y0: ac.rh_omega.get_device_rect().height / scale_factor,
    });
}

pub fn draw_background(args: DrawingArgs) {
    if args.ac.config.borrow().show_dendritic_zone {
        background::draw_dendtritic_snow_growth_zone(args);
    }

    background::draw_background_lines(args);
    background::draw_labels(args);
}

pub fn draw_rh_profile(args: DrawingArgs) {
    use gui::plot_context::PlotContext;

    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if !config.show_rh_profile {
        return;
    }

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{DewPoint, Pressure, Temperature};

        let pres_data = sndg.get_profile(Pressure);
        let t_data = sndg.get_profile(Temperature);
        let td_data = sndg.get_profile(DewPoint);
        let mut profile = izip!(pres_data, t_data, td_data)
            .filter_map(|triplet| {
                if let (Some(p), Some(t), Some(td)) = (
                    triplet.0.as_option(),
                    triplet.1.as_option(),
                    triplet.2.as_option(),
                ) {
                    Some((p, ::formula::rh(t, td)))
                } else {
                    None
                }
            })
            .filter_map(|pair| {
                let (p, rh) = pair;
                if p > config::MINP {
                    let ScreenCoords { y, .. } =
                        ac.rh_omega.convert_wp_to_screen(WPCoords { w: 0.0, p });
                    let bb = ac.rh_omega.bounding_box_in_screen_coords();
                    let x = bb.lower_left.x + bb.width() * rh;

                    Some(ScreenCoords { x, y })
                } else {
                    None
                }
            });

        let line_width = config.omega_line_width;
        let mut rgba = config.rh_rgba;
        rgba.3 *= 0.75;

        cr.set_line_width(cr.device_to_user_distance(line_width, 0.0).0);
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        let mut previous: Option<ScreenCoords>;
        let mut curr: Option<ScreenCoords> = None;
        let mut next: Option<ScreenCoords> = None;
        loop {
            previous = curr;
            curr = next;
            next = profile.next();

            const XMIN: f64 = 0.0;
            let xmax: f64;
            let ymin: f64;
            let ymax: f64;
            if let (Some(p), Some(c), Some(n)) = (previous, curr, next) {
                // In the middle - most common
                xmax = c.x;
                let down = (c.y - p.y) / 2.0;
                let up = (n.y - c.y) / 2.0;
                ymin = c.y - down;
                ymax = c.y + up;
            } else if let (Some(p), Some(c), None) = (previous, curr, next) {
                // Last point
                xmax = c.x;
                let down = (c.y - p.y) / 2.0;
                let up = down;
                ymin = c.y - down;
                ymax = c.y + up;
            } else if let (None, Some(c), Some(n)) = (previous, curr, next) {
                // First point
                xmax = c.x;
                let up = (n.y - c.y) / 2.0;
                let down = up;
                ymin = c.y - down;
                ymax = c.y + up;
            } else if let (Some(_), None, None) = (previous, curr, next) {
                // Done - get out of here
                break;
            } else if let (None, None, Some(_)) = (previous, curr, next) {
                // Just getting into the loop - do nothing
                continue;
            } else {
                // Impossible state
                unreachable!();
            }

            cr.rectangle(XMIN, ymin, xmax, ymax - ymin);
            cr.fill_preserve();
            cr.stroke();
        }
    }
}

pub fn draw_omega_profile(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if !config.show_omega_profile {
        return;
    }

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{Pressure, PressureVerticalVelocity};

        let pres_data = sndg.get_profile(Pressure);
        let omega_data = sndg.get_profile(PressureVerticalVelocity);
        let line_width = config.omega_line_width;
        let line_rgba = config.omega_rgba;

        let profile_data = pres_data
            .iter()
            .zip(omega_data.iter())
            .filter_map(|val_pair| {
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
            });

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    }
}

pub fn draw_active_readout(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if config.show_active_readout {
        let sample_p = if let Some(sample) = ac.get_sample() {
            if let Some(sample_p) = sample.pressure.as_option() {
                sample_p
            } else {
                return;
            }
        } else {
            return;
        };

        let rgba = config.active_readout_line_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(
            cr.device_to_user_distance(config.active_readout_line_width, 0.0)
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
