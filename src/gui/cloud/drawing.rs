
use app::config;
use coords::{ScreenCoords, PPCoords};
use gui::{plot_curve_from_points, DrawingArgs, PlotContextExt};

pub fn draw_background(args: DrawingArgs) {
    if args.ac.config.borrow().show_dendritic_zone {
        draw_dendtritic_snow_growth_zone(args);
    }

    draw_background_lines(args);
}

pub fn draw_data(args: DrawingArgs) {
    draw_cloud_profile(args);
}

pub fn draw_overlays(args: DrawingArgs) {
    draw_active_readout(args);
}

fn draw_cloud_profile(args: DrawingArgs) {
    
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{CloudFraction, Pressure};

        let pres_data = sndg.get_profile(Pressure);
        let c_data = sndg.get_profile(CloudFraction);
        let mut profile = izip!(pres_data, c_data)
            .filter_map(|pair| {
                if let (Some(p), Some(c)) = (
                    *pair.0,
                    *pair.1,
                ) {
                    Some((p, c))
                } else {
                    None
                }
            })
            .filter_map(|pair| {
                let (press, pcnt) = pair;
                if press > config::MINP {
                    Some(ac.cloud.convert_pp_to_screen(PPCoords { pcnt: pcnt / 100.0, press }))
                } else {
                    None
                }
            });

        let line_width = config.omega_line_width; // FIXME: use another lind width?
        let mut rgba = config.rh_rgba; // FIXME: Make a cloud rgba
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

fn draw_active_readout(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if config.show_active_readout {
        let sample_p = if let Some(sample) = ac.get_sample() {
            if let Some(sample_p) = sample.pressure {
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
        let start = ac.cloud.convert_pp_to_screen(PPCoords {
            pcnt: 0.0,
            press: sample_p,
        });
        let end = ac.cloud.convert_pp_to_screen(PPCoords {
            pcnt: 1.0,
            press: sample_p,
        });
        cr.move_to(start.x, start.y);
        cr.line_to(end.x, end.y);
        cr.stroke();
    }
}

fn draw_background_lines(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    // Draw isobars
    if config.show_isobars {
        for pnts in config::ISOBAR_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|xy_coords| ac.cloud.convert_xy_to_screen(*xy_coords));
            plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
        }
    }
}

pub fn draw_dendtritic_snow_growth_zone(args: DrawingArgs) {
    use sounding_base::Profile::Pressure;

    let (ac, cr) = (args.ac, args.cr);

    // If is plottable, draw snow growth zones
    if let Some(ref snd) = ac.get_sounding_for_display() {
        let rgba = ac.config.borrow().dendritic_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        for (bottom_p, top_p) in ::sounding_analysis::dendritic_growth_zone(snd, Pressure) {
            let mut coords = [
                (0.0, bottom_p),
                (0.0, top_p),
                (1.0, top_p),
                (1.0, bottom_p),
            ];

            // Convert points to screen coords
            for coord in &mut coords {
                let screen_coords = ac.cloud.convert_pp_to_screen(PPCoords {
                    pcnt: coord.0,
                    press: coord.1,
                });
                coord.0 = screen_coords.x;
                coord.1 = screen_coords.y;
            }

            let mut coord_iter = coords.iter();
            for coord in coord_iter.by_ref().take(1) {
                cr.move_to(coord.0, coord.1);
            }
            for coord in coord_iter {
                cr.line_to(coord.0, coord.1);
            }

            cr.close_path();
            cr.fill();
        }
    }
}
