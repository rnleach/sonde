use cairo::{FontFace, FontSlant, FontWeight};

use app::config;
use coords::{WPCoords, TPCoords, ScreenRect, ScreenCoords};
use gui::{plot_curve_from_points, check_overlap_then_add};
use gui::{DrawingArgs, set_font_size};



pub fn draw_background_lines(args: DrawingArgs) {

    let (ac, cr, da) = (args.ac, args.cr, args.da);

    // Draw isobars
    if ac.config.show_isobars {
        for pnts in config::ISOBAR_PNTS.iter() {
            let TPCoords { pressure: p, .. } = pnts[0];

            let pnts = [
                WPCoords {
                    w: -config::MAX_ABS_W,
                    p,
                },
                WPCoords {
                    w: config::MAX_ABS_W,
                    p,
                },
            ];
            let pnts = pnts.iter().map(|wp_coords| {
                ac.rh_omega.convert_wp_to_screen(da, *wp_coords)
            });

            let mut line_width = ac.config.background_line_width;
            line_width = line_width / cr.device_to_user_distance(line_width, 0.0).0 *
                -cr.device_to_user_distance(0.0, line_width).1;

            plot_curve_from_points(cr, line_width, ac.config.isobar_rgba, pnts);
        }
    }

    // Draw w-lines
    if ac.config.show_iso_omega_lines {
        for v_line in config::ISO_OMEGA_PNTS.iter() {

            plot_curve_from_points(
                cr,
                ac.config.background_line_width,
                ac.config.isobar_rgba,
                v_line.iter().map(|wp_coords| {
                    ac.rh_omega.convert_wp_to_screen(da, *wp_coords)
                }),
            );
        }

        // Make a thicker zero line
        plot_curve_from_points(
            cr,
            ac.config.background_line_width * 2.6,
            ac.config.isobar_rgba,
            ([
                WPCoords {
                    w: 0.0,
                    p: config::MAXP,
                },
                WPCoords {
                    w: 0.0,
                    p: config::MINP,
                },
            ]).iter()
                .map(|wp_coords| ac.rh_omega.convert_wp_to_screen(da, *wp_coords)),
        );
    }
}

pub fn draw_dendtritic_snow_growth_zone(args: DrawingArgs) {
    use sounding_base::Profile::Pressure;

    let (ac, cr, da) = (args.ac, args.cr, args.da);

    // If is plottable, draw snow growth zones
    if let Some(snd) = ac.get_sounding_for_display() {

        let rgba = ac.config.dendritic_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        for (bottom_p, top_p) in ::sounding_analysis::dendritic_growth_zone(snd, Pressure) {
            let mut coords = [
                (-config::MAX_ABS_W, bottom_p),
                (-config::MAX_ABS_W, top_p),
                (config::MAX_ABS_W, top_p),
                (config::MAX_ABS_W, bottom_p),
            ];

            // Convert points to screen coords
            for coord in &mut coords {
                let screen_coords = ac.rh_omega.convert_wp_to_screen(
                    da,
                    WPCoords {
                        w: coord.0,
                        p: coord.1,
                    },
                );
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

pub fn draw_labels(args: DrawingArgs) {
    use coords::Rect;
    use gui::LazyDrawingCacheVar::OmegaLabelPadding;

    let (ac, cr, da) = (args.ac, args.cr, args.da);

    if ac.config.show_labels {
        let font_face =
            FontFace::toy_create(&ac.config.font_name, FontSlant::Normal, FontWeight::Bold);
        cr.set_font_face(font_face);


        set_font_size(&ac.rh_omega, da, ac.config.label_font_size, cr, ac);

        let labels = collect_labels(args);
        let padding = ac.drawing_cache.get(OmegaLabelPadding, args);

        for (label, rect) in labels {
            let ScreenRect { lower_left, .. } = rect;

            let mut rgba = ac.config.background_rgba;
            cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            cr.rectangle(
                lower_left.x - padding,
                lower_left.y - padding,
                rect.width() + 2.0 * padding,
                rect.height() + 2.0 * padding,
            );
            cr.fill();

            // Setup label colors
            rgba = ac.config.label_rgba;
            cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            cr.move_to(lower_left.x, lower_left.y);
            cr.show_text(&label);
        }
    }
}

pub fn collect_labels(args: DrawingArgs) -> Vec<(String, ScreenRect)> {
    use gui::plot_context::PlotContext;

    let (ac, cr, da) = (args.ac, args.cr, args.da);

    let mut labels = vec![];

    let screen_edges = ac.rh_omega.calculate_plot_edges(da, cr, ac);
    let ScreenRect { lower_left, .. } = screen_edges;

    if ac.config.show_iso_omega_lines {
        let WPCoords { p: screen_max_p, .. } = ac.rh_omega.convert_screen_to_wp(lower_left);

        for &w in [0.0].iter().chain(config::ISO_OMEGA.iter()) {

            let label = format!("{:.0}", w * 10.0);

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                x: mut xpos,
                y: mut ypos,
            } = ac.rh_omega.convert_wp_to_screen(
                da,
                WPCoords { w, p: screen_max_p },
            );
            xpos -= extents.width / 2.0; // Center
            ypos -= extents.height / 2.0; // Center
            ypos += extents.height; // Move up off bottom axis.

            let ScreenRect {
                lower_left: ScreenCoords { x: xmin, .. },
                upper_right: ScreenCoords { x: xmax, .. },
            } = screen_edges;

            if xpos < xmin || xpos + extents.width > xmax {
                continue;
            }

            let label_lower_left = ScreenCoords { x: xpos, y: ypos };
            let label_upper_right = ScreenCoords {
                x: xpos + extents.width,
                y: ypos + extents.height,
            };

            let pair = (
                label,
                ScreenRect {
                    lower_left: label_lower_left,
                    upper_right: label_upper_right,
                },
            );
            check_overlap_then_add(cr, ac, &mut labels, &screen_edges, pair);
        }
    }

    labels
}
