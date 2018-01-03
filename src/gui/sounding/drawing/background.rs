use app::config;
use coords::TPCoords;
use gui::{plot_curve_from_points, plot_dashed_curve_from_points, DrawingArgs, PlotContextExt};

pub fn draw_background_fill(args: DrawingArgs) {
    let ac = args.ac;
    let config = ac.config.borrow();

    if config.show_background_bands {
        draw_temperature_banding(args);
    }

    if config.show_hail_zone {
        draw_hail_growth_zone(args);
    }

    if config.show_dendritic_zone {
        draw_dendtritic_growth_zone(args);
    }
}

// Draw isentrops, isotherms, isobars, ...
pub fn draw_background_lines(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    // Draws background lines from the bottom up.

    // Draw isentrops
    if config.show_isentrops {
        for pnts in config::ISENTROP_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|xy_coords| ac.skew_t.convert_xy_to_screen(*xy_coords));
            plot_curve_from_points(cr, config.background_line_width, config.isentrop_rgba, pnts);
        }
    }

    // Draw theta-e lines
    if config.show_iso_theta_e {
        for pnts in config::ISO_THETA_E_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|xy_coords| ac.skew_t.convert_xy_to_screen(*xy_coords));
            plot_curve_from_points(
                cr,
                config.background_line_width,
                config.iso_theta_e_rgba,
                pnts,
            );
        }
    }

    // Draw mixing ratio lines
    if config.show_iso_mixing_ratio {
        for pnts in config::ISO_MIXING_RATIO_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|xy_coords| ac.skew_t.convert_xy_to_screen(*xy_coords));
            plot_dashed_curve_from_points(
                cr,
                config.background_line_width,
                config.iso_mixing_ratio_rgba,
                pnts,
            );
        }
    }

    // Draw isotherms
    if config.show_isotherms {
        for pnts in config::ISOTHERM_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|tp_coords| ac.skew_t.convert_xy_to_screen(*tp_coords));
            plot_curve_from_points(cr, config.background_line_width, config.isotherm_rgba, pnts);
        }
    }

    // Draw isobars
    if config.show_isobars {
        for pnts in config::ISOBAR_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|xy_coords| ac.skew_t.convert_xy_to_screen(*xy_coords));

            plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
        }
    }

    // Draw the freezing line
    if config.show_freezing_line {
        let pnts = &[
            TPCoords {
                temperature: 0.0,
                pressure: config::MAXP,
            },
            TPCoords {
                temperature: 0.0,
                pressure: config::MINP,
            },
        ];
        let pnts = pnts.iter()
            .map(|tp_coords| ac.skew_t.convert_tp_to_screen(*tp_coords));
        plot_curve_from_points(
            cr,
            config.freezing_line_width,
            config.freezing_line_color,
            pnts,
        );
    }
}

fn draw_temperature_banding(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);

    let rgba = ac.config.borrow().background_band_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    let mut start_line = -160i32;
    while start_line < 100 {
        let t1 = f64::from(start_line);
        let t2 = t1 + 10.0;

        draw_temperature_band(t1, t2, args);

        start_line += 20;
    }
}

fn draw_hail_growth_zone(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);

    let rgba = ac.config.borrow().hail_zone_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    draw_temperature_band(-30.0, -10.0, args);
}

fn draw_dendtritic_growth_zone(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);

    let rgba = ac.config.borrow().dendritic_zone_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

    draw_temperature_band(-18.0, -12.0, args);
}

fn draw_temperature_band(cold_t: f64, warm_t: f64, args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);

    // Assume color has already been set up for us.

    const MAXP: f64 = config::MAXP;
    const MINP: f64 = config::MINP;

    let mut coords = [
        (warm_t, MAXP),
        (warm_t, MINP),
        (cold_t, MINP),
        (cold_t, MAXP),
    ];

    // Convert points to screen coords
    for coord in &mut coords {
        let screen_coords = ac.skew_t.convert_tp_to_screen(TPCoords {
            temperature: coord.0,
            pressure: coord.1,
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
