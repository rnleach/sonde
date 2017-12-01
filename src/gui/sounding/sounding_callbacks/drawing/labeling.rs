//! Functions used for adding labels to the sounding plot
use app::{AppContext, PlotContext, config};
use coords::{ScreenCoords, ScreenRect, TPCoords, XYCoords, DeviceCoords, Rect};
use gui::check_overlap_then_add;
use gui::sounding::set_font_size;

use cairo::{Context, FontExtents, FontFace, FontSlant, FontWeight};

pub fn prepare_to_label(cr: &Context, ac: &AppContext) {

    let font_face = FontFace::toy_create(&ac.config.font_name, FontSlant::Normal, FontWeight::Bold);
    cr.set_font_face(font_face);

    set_font_size(ac.config.label_font_size, cr, ac);
}

// Label the pressure, temperatures, etc lines.
pub fn draw_background_labels(cr: &Context, ac: &AppContext) {
    let labels = collect_labels(cr, ac);
    draw_labels(cr, ac, labels);
}

fn collect_labels(cr: &Context, ac: &AppContext) -> Vec<(String, ScreenRect)> {
    use app::PlotContext;

    let mut labels = vec![];

    let screen_edges = ac.skew_t.calculate_plot_edges(cr, ac);
    let ScreenRect { lower_left, .. } = screen_edges;

    if ac.config.show_isobars {
        for &p in &config::ISOBARS {

            let label = format!("{}", p);

            let extents = cr.text_extents(&label);

            let ScreenCoords { y: screen_y, .. } = ac.skew_t.convert_tp_to_screen(TPCoords {
                temperature: 0.0,
                pressure: p,
            });
            let screen_y = screen_y - extents.height / 2.0;

            let label_lower_left = ScreenCoords {
                x: lower_left.x,
                y: screen_y,
            };
            let label_upper_right = ScreenCoords {
                x: lower_left.x + extents.width,
                y: screen_y + extents.height,
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

    if ac.config.show_isotherms {
        let TPCoords { pressure: screen_max_p, .. } = ac.skew_t.convert_screen_to_tp(lower_left);
        for &t in &config::ISOTHERMS {

            let label = format!("{}", t);

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                x: mut xpos,
                y: mut ypos,
            } = ac.skew_t.convert_tp_to_screen(TPCoords {
                temperature: t,
                pressure: screen_max_p,
            });
            xpos -= extents.width / 2.0; // Center
            ypos -= extents.height / 2.0; // Center
            ypos += extents.height; // Move up off bottom axis.
            xpos += extents.height; // Move right for 45 degree angle from move up

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

fn draw_labels(cr: &Context, ac: &AppContext, labels: Vec<(String, ScreenRect)>) {

    let padding = ac.skew_t.get_label_padding();

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

// Add a description box
pub fn draw_legend(cr: &Context, ac: &AppContext) {
    use app::PlotContext;

    if !(ac.plottable() && ac.config.show_legend) {
        return;
    }

    let mut upper_left = ac.skew_t.convert_device_to_screen(DeviceCoords::origin());
    upper_left.x += ac.skew_t.get_edge_padding();
    upper_left.y -= ac.skew_t.get_edge_padding();

    // Make sure we stay on the x-y coords domain
    let ScreenCoords { x: xmin, y: ymax } =
        ac.skew_t.convert_xy_to_screen(XYCoords { x: 0.0, y: 1.0 });
    let edge_offset = upper_left.x; // This distance is used to push off the edge by 5 pixels
    if ymax - edge_offset < upper_left.y {
        upper_left.y = ymax - edge_offset;
    }

    if xmin + edge_offset > upper_left.x {
        upper_left.x = xmin + edge_offset;
    }

    let font_extents = cr.font_extents();

    let (source_description, valid_time, location) = build_legend_strings(ac);

    let (box_width, box_height) = calculate_legend_box_size(
        cr,
        ac,
        &font_extents,
        &source_description,
        &valid_time,
        &location,
    );

    let legend_rect = ScreenRect {
        lower_left: ScreenCoords {
            x: upper_left.x,
            y: upper_left.y - box_height,
        },
        upper_right: ScreenCoords {
            x: upper_left.x + box_width,
            y: upper_left.y,
        },
    };

    draw_legend_rectangle(cr, ac, &legend_rect);

    draw_legend_text(
        cr,
        ac,
        &upper_left,
        &font_extents,
        &source_description,
        &valid_time,
        &location,
    );
}

fn build_legend_strings(ac: &AppContext) -> (Option<String>, Option<String>, Option<String>) {
    use chrono::Weekday::*;

    let source_description: Option<String> = ac.get_source_description();
    let mut valid_time: Option<String> = None;
    let mut location: Option<String> = None;

    if let Some(snd) = ac.get_sounding_for_display() {
        // Build the valid time part
        if let Some(vt) = snd.get_valid_time() {
            use chrono::{Datelike, Timelike};
            let mut temp_string = format!(
                "Valid: {} {:02}/{:02}/{:04} {:02}Z",
                match vt.weekday() {
                    Sun => "Sunday",
                    Mon => "Monday",
                    Tue => "Tuesday",
                    Wed => "Wednesday",
                    Thu => "Thursday",
                    Fri => "Friday",
                    Sat => "Saturday",
                },
                vt.month(),
                vt.day(),
                vt.year(),
                vt.hour()
            );

            if let Some(lt) = snd.get_lead_time().as_option() {
                temp_string.push_str(&format!(" F{:03}", lt));
            }

            valid_time = Some(temp_string);
        }

        // Build location part.
        let (lat, lon, elevation) = snd.get_location();
        if lat.as_option().is_some() || lon.as_option().is_some() ||
            elevation.as_option().is_some()
        {
            location = Some("".to_owned());
            if let Some(ref mut loc) = location {
                if let Some(lat) = lat.as_option() {
                    loc.push_str(&format!("{:.2}", lat));
                }
                if let Some(lon) = lon.as_option() {
                    loc.push_str(&format!(", {:.2}", lon));
                }
                if let Some(el) = elevation.as_option() {
                    loc.push_str(&format!(", {:.0}m ({:.0}ft)", el, el * 3.28084));
                }
            }
        }
    }

    (source_description, valid_time, location)
}

fn calculate_legend_box_size(
    cr: &Context,
    ac: &AppContext,
    font_extents: &FontExtents,
    source_description: &Option<String>,
    valid_time: &Option<String>,
    location: &Option<String>,
) -> (f64, f64) {

    let mut box_width: f64 = 0.0;
    let mut box_height: f64 = 0.0;

    if let Some(ref src) = *source_description {
        let extents = cr.text_extents(src);
        if extents.width > box_width {
            box_width = extents.width;
        }
        box_height += extents.height;
    }

    if let Some(ref vt) = *valid_time {
        let extents = cr.text_extents(vt);
        if extents.width > box_width {
            box_width = extents.width;
        }
        box_height += extents.height;
        // Add line spacing if previous line was there.
        if source_description.is_some() {
            box_height += font_extents.height - extents.height;
        }
    }

    if let Some(ref loc) = *location {
        let extents = cr.text_extents(loc);
        if extents.width > box_width {
            box_width = extents.width;
        }
        box_height += extents.height;
        // Add line spacing if previous line was there.
        if valid_time.is_some() {
            box_height += font_extents.height - extents.height;
        }
    }

    // Add room for the last line's descent
    box_height += font_extents.descent;

    // Add padding last
    let padding = ac.skew_t.get_edge_padding();
    box_height += 2.0 * padding;
    box_width += 2.0 * padding;

    (box_width, box_height)
}

fn draw_legend_rectangle(cr: &Context, ac: &AppContext, screen_rect: &ScreenRect) {
    let ScreenRect { lower_left, .. } = *screen_rect;

    cr.rectangle(
        lower_left.x,
        lower_left.y,
        screen_rect.width(),
        screen_rect.height(),
    );

    let rgb = ac.config.label_rgba;
    cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
    cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
    cr.stroke_preserve();
    let rgba = ac.config.background_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.fill();
}

fn draw_legend_text(
    cr: &Context,
    ac: &AppContext,
    upper_left: &ScreenCoords,
    font_extents: &FontExtents,
    source_description: &Option<String>,
    valid_time: &Option<String>,
    location: &Option<String>,
) {
    let rgb = ac.config.label_rgba;
    cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);

    let padding = ac.skew_t.get_edge_padding();

    // Remember how many lines we have drawn so far for setting position of the next line.
    let mut num_lines_drawn = 0;

    // Get into the initial position
    cr.move_to(
        upper_left.x + padding,
        upper_left.y - padding - font_extents.ascent,
    );

    if let Some(ref src) = *source_description {
        cr.show_text(src);
        num_lines_drawn += 1;
        cr.move_to(
            upper_left.x + padding,
            upper_left.y - padding - font_extents.ascent -
                f64::from(num_lines_drawn) * font_extents.height,
        );
    }
    if let Some(ref vt) = *valid_time {
        cr.show_text(vt);
        num_lines_drawn += 1;
        cr.move_to(
            upper_left.x + padding,
            upper_left.y - padding - font_extents.ascent -
                f64::from(num_lines_drawn) * font_extents.height,
        );
    }
    if let Some(ref loc) = *location {
        cr.show_text(loc);
        num_lines_drawn += 1;
        cr.move_to(
            upper_left.x + padding,
            upper_left.y - padding - font_extents.ascent -
                f64::from(num_lines_drawn) * font_extents.height,
        );
    }
}
