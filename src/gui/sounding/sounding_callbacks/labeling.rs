//! Functions used for adding labels to the sounding plot
use app::AppContext;
use config;

use coords::ScreenCoords;

use cairo::{Context, Matrix, FontExtents, FontFace, FontSlant, FontWeight};

// Set up the font matrix, and set the font, etc.
pub fn prepare_to_label(cr: &Context, ac: &AppContext) {

    // Configure the font.
    let font_face = FontFace::toy_create(config::FONT_NAME, FontSlant::Normal, FontWeight::Bold);
    cr.set_font_face(font_face);

    set_font_size(config::LARGE_FONT_SIZE, cr, ac);
}

// Set the font size by reseting the font matrix
fn set_font_size(size_in_pnts: f64, cr: &Context, ac: &AppContext) {
    let dpi = match ac.get_dpi() {
        None => 72.0,
        Some(value) => value,
    };

    let font_size = size_in_pnts / 72.0 * dpi;
    let (font_size, _) = ac.convert_device_to_screen((font_size, 0.0));

    // Flip the y-coordinate so it displays the font right side up
    cr.set_font_matrix(Matrix {
        xx: 1.0 * font_size,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0 * font_size, // Reflect it to be right side up!
        x0: 0.0,
        y0: 0.0,
    });
}

// Label the pressure, temperatures, etc lines.
pub fn draw_background_labels(cr: &Context, ac: &AppContext) {

    // Get min/max screen coordinate values
    let (lower_left_screen, upper_right_screen) = ac.bounding_box_in_screen_coords();
    let (mut screen_x_min, _screen_y_min) = lower_left_screen;
    let (screen_x_max, screen_y_max) = upper_right_screen;

    // Get coordinates to keep labels from flowing off the chart.
    let (xmin, _ymin) = ac.convert_xy_to_screen((0.0, 0.0));
    let (_, mut screen_max_p) = ac.convert_screen_to_tp((0.0, 0.0));
    if screen_max_p > config::MAXP {
        screen_max_p = config::MAXP;
    }
    if xmin > screen_x_min {
        screen_x_min = xmin;
    }

    // Label isobars
    let mut max_p_label_right_edge: f64 = 0.0; // Used for checking overlap later with T
    let mut last_label_y = ::std::f64::MIN; // Used for checking overlap between pressure labels
    let mut rgba = config::ISOBAR_RGBA;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, 1.0);
    for &p in config::ISOBARS.into_iter() {
        // Make the label text
        let label = format!("{}", p);

        // Calculate position of lower left edge for label
        let (_, mut screen_y) = ac.convert_tp_to_screen((0.0, p));
        screen_y += 0.005 * screen_y_max; // Lift off the pressure line slightly

        // Check for vertical overlap.
        let label_extents = cr.text_extents(&label);
        let (label_width, label_height) = (label_extents.width, label_extents.height);
        if screen_y < label_height + last_label_y {
            continue;
        }
        last_label_y = screen_y;

        // Update right edge for checking with T later.
        if label_width > max_p_label_right_edge {
            max_p_label_right_edge = label_width;
        }

        // Draw the label
        cr.move_to(screen_x_min + 0.01 * screen_x_max, screen_y);
        cr.show_text(&label);
    }
    // This is width, add position of left edge to get right edge
    max_p_label_right_edge += screen_x_min + 0.01 * screen_x_max;

    // Label isotherms
    rgba = config::ISOTHERM_RGBA;
    let (mut last_label_x_max, mut last_label_x_min) = (0.0, 0.0);
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, 1.0);
    for &t in config::ISOTHERMS.into_iter() {
        // Make the label text
        let label = format!("{}", t);

        // Calculate position for lower left edge of label
        let (mut screen_x, mut screen_y) = ac.convert_tp_to_screen((t, screen_max_p));
        screen_y += 0.008 * screen_y_max;
        screen_x += 0.008 * screen_y_max;

        // Check for overlap with pressure labels
        if screen_x < max_p_label_right_edge {
            continue;
        }

        // Check for overlap with other T labels
        let extents_width = cr.text_extents(&label).width;
        let (x_min, x_max) = (screen_x, screen_x + extents_width);
        if (x_min > last_label_x_min && x_min < last_label_x_max) ||
            (x_max > last_label_x_min && x_max < last_label_x_max)
        {
            continue;
        }

        last_label_x_min = x_min - 0.008 * screen_x_max;
        last_label_x_max = x_max + 0.008 * screen_x_max;

        // Draw the label
        cr.move_to(screen_x, screen_y);
        cr.show_text(&label);
    }
}

// Add a description box
pub fn draw_legend(cr: &Context, ac: &AppContext) {
    if !ac.plottable() {
        return;
    }

    let upper_left = ac.convert_device_to_screen((5.0, 5.0));
    let font_extents = cr.font_extents();

    let (source_name, valid_time, location) = build_label_strings(ac);

    let (box_width, box_height) =
        calculate_legend_box_size(cr, &font_extents, &source_name, &valid_time, &location);

    draw_legend_rectangle(cr, &upper_left, box_width, box_height);

    draw_legend_text(
        cr,
        &upper_left,
        &font_extents,
        &source_name,
        &valid_time,
        &location,
    );

}

fn build_label_strings(ac: &AppContext) -> (Option<String>, Option<String>, Option<String>) {

    let source_name: Option<String> = ac.get_source_name();
    let mut valid_time: Option<String> = None;
    let mut location: Option<String> = None;
    if let Some(snd) = ac.get_sounding_for_display() {
        // Build the valid time part
        if let Some(vt) = snd.valid_time {
            use chrono::{Datelike, Timelike};
            valid_time = Some(format!(
                "Valid: {:02}/{:02}/{:04} {:02}Z",
                vt.month(),
                vt.day(),
                vt.year(),
                vt.hour()
            ));
        }

        // Build location part.
        if snd.lat.as_option().is_some() || snd.lon.as_option().is_some() ||
            snd.elevation.as_option().is_some()
        {
            location = Some("".to_owned());
            if let Some(ref mut loc) = location {
                if let Some(lat) = snd.lat.as_option() {
                    loc.push_str(&format!("{:.2}", lat));
                }
                if let Some(lon) = snd.lon.as_option() {
                    loc.push_str(&format!(", {:.2}", lon));
                }
                if let Some(el) = snd.elevation.as_option() {
                    loc.push_str(&format!(", {:.0}m ({:.0}ft)", el, el * 3.28084));
                }
            }
        }
    }

    (source_name, valid_time, location)
}

fn calculate_legend_box_size(
    cr: &Context,
    font_extents: &FontExtents,
    source_name: &Option<String>,
    valid_time: &Option<String>,
    location: &Option<String>,
) -> (f64, f64) {

    let mut box_width: f64 = 0.0;
    let mut box_height: f64 = 0.0;

    if let &Some(ref src) = source_name {
        let extents = cr.text_extents(src);
        if extents.width > box_width {
            box_width = extents.width;
        }
        box_height += extents.height;
    }

    if let &Some(ref vt) = valid_time {
        let extents = cr.text_extents(vt);
        if extents.width > box_width {
            box_width = extents.width;
        }
        box_height += extents.height;
        // Add line spacing if previous line was there.
        if source_name.is_some() {
            box_height += font_extents.height - extents.height;
        }
    }

    if let &Some(ref loc) = location {
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
    box_height += 2.0 * config::DEFAULT_PADDING;
    box_width += 2.0 * config::DEFAULT_PADDING;

    (box_width, box_height)
}

fn draw_legend_rectangle(cr: &Context, upper_left: &ScreenCoords, width: f64, height: f64) {
    cr.rectangle(upper_left.0, upper_left.1 - height, width, height);

    let rgb = config::ISOBAR_RGBA;
    cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
    cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
    cr.stroke_preserve();
    let rgb = config::BACKGROUND_RGB;
    cr.set_source_rgb(rgb.0, rgb.1, rgb.2);
    cr.fill();
}

fn draw_legend_text(
    cr: &Context,
    upper_left: &ScreenCoords,
    font_extents: &FontExtents,
    source_name: &Option<String>,
    valid_time: &Option<String>,
    location: &Option<String>,
) {
    let rgb = config::ISOBAR_RGBA;
    cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);

    // Remember how many lines we have drawn so far for setting position of the next line.
    let mut num_lines_drawn = 0;

    // Get into the initial position
    cr.move_to(
        upper_left.0 + config::DEFAULT_PADDING,
        upper_left.1 - config::DEFAULT_PADDING - font_extents.ascent,
    );

    if let &Some(ref src) = source_name {
        cr.show_text(src);
        num_lines_drawn += 1;
        cr.move_to(
            upper_left.0 + config::DEFAULT_PADDING,
            upper_left.1 - config::DEFAULT_PADDING - font_extents.ascent -
                num_lines_drawn as f64 * font_extents.height,
        );
    }
    if let &Some(ref vt) = valid_time {
        cr.show_text(vt);
        num_lines_drawn += 1;
        cr.move_to(
            upper_left.0 + config::DEFAULT_PADDING,
            upper_left.1 - config::DEFAULT_PADDING - font_extents.ascent -
                num_lines_drawn as f64 * font_extents.height,
        );
    }
    if let &Some(ref loc) = location {
        cr.show_text(loc);
        num_lines_drawn += 1;
        cr.move_to(
            upper_left.0 + config::DEFAULT_PADDING,
            upper_left.1 - config::DEFAULT_PADDING - font_extents.ascent -
                num_lines_drawn as f64 * font_extents.height,
        );
    }
}
