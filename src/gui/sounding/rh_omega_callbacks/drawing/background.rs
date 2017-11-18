use cairo::{Context, FontFace, FontSlant, FontWeight};

use app::{AppContext, config};
use coords::{WPCoords, TPCoords, ScreenRect, ScreenCoords};
use gui::sounding::{plot_curve_from_points, set_font_size, check_overlap_then_add};

pub fn draw_background(cr: &Context, ac: &AppContext) {

    if ac.config.show_dendritic_zone {
        draw_dendtritic_snow_growth_zone(cr, ac);
    }

    draw_background_lines(cr, ac);
    draw_labels(cr, ac);
}

fn draw_background_lines(cr: &Context, ac: &AppContext) {
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
    if ac.config.show_iso_omega_lines {
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
                .map(|wp_coords| ac.rh_omega.convert_wp_to_screen(*wp_coords)),
        );
    }
}

fn draw_dendtritic_snow_growth_zone(cr: &Context, ac: &AppContext) {
    use sounding_base::Profile::Pressure;

    // If is plottable, draw snow growth zones
    if let Some(snd) = ac.get_sounding_for_display() {

        let rgba = ac.config.dendritic_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        for (bottom_p, top_p) in ::sounding_analysis::dendritic_growth_zone(snd, Pressure) {
            let mut coords = [
                (-ac.rh_omega.get_max_abs_omega(), bottom_p),
                (-ac.rh_omega.get_max_abs_omega(), top_p),
                (ac.rh_omega.get_max_abs_omega(), top_p),
                (ac.rh_omega.get_max_abs_omega(), bottom_p),
            ];

            // Convert points to screen coords
            for coord in &mut coords {
                let screen_coords = ac.rh_omega.convert_wp_to_screen(WPCoords {
                    w: coord.0,
                    p: coord.1,
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

pub fn draw_labels(cr: &Context, ac: &AppContext) {
    use coords::Rect;

    if ac.config.show_labels {
        let font_face =
            FontFace::toy_create(&ac.config.font_name, FontSlant::Normal, FontWeight::Bold);
        cr.set_font_face(font_face);

        set_font_size(ac.config.label_font_size, cr, ac);

        let labels = collect_labels(cr, ac);
        let padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;

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

fn collect_labels(cr: &Context, ac: &AppContext) -> Vec<(String, ScreenRect)> {
    use app::PlotContext;

    let mut labels = vec![];

    let screen_edges = ac.rh_omega.calculate_plot_edges(cr, ac);
    let ScreenRect { lower_left, .. } = screen_edges;

    if ac.config.show_iso_omega_lines {
        let WPCoords { p: screen_max_p, .. } = ac.rh_omega.convert_screen_to_wp(lower_left);

        for &w in &[
            0.0,
            -ac.rh_omega.get_max_abs_omega(),
            ac.rh_omega.get_max_abs_omega(),
        ]
        {

            let label = format!("{:.0}", w * 10.0);

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                x: mut xpos,
                y: mut ypos,
            } = ac.rh_omega.convert_wp_to_screen(
                WPCoords { w, p: screen_max_p },
            );
            xpos -= extents.width / 2.0; // Center
            ypos -= extents.height / 2.0; // Center
            ypos += extents.height; // Move up off bottom axis.

            let ScreenRect {
                lower_left: ScreenCoords { x: xmin, .. },
                upper_right: ScreenCoords { x: xmax, .. },
            } = screen_edges;

            if xpos < xmin {
                xpos = xmin;
            }
            if xpos + extents.width > xmax {
                xpos = xmax - extents.width;
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
