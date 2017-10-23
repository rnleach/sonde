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
    let labels = collect_labels(cr, ac);
    draw_labels(cr, labels);
}

// FIXME: Move this somewhere else in GUI? Seems more useful than just here.
#[derive(Clone, Copy)]
struct ScreenRect {
    lower_left: ScreenCoords,
    upper_right: ScreenCoords,
}

impl ScreenRect {
    fn overlaps(&self, other: &ScreenRect) -> bool {
        let (xmin_s, ymin_s) = self.lower_left;
        let (xmax_s, ymax_s) = self.upper_right;
        let (xmin_o, ymin_o) = other.lower_left;
        let (xmax_o, ymax_o) = other.upper_right;

        if xmin_s > xmax_o {
            return false;
        }
        if xmax_s < xmin_o {
            return false;
        }
        if ymin_s > ymax_o {
            return false;
        }
        if ymax_s < ymin_o {
            return false;
        }

        true
    }

    fn inside(&self, big_rect: &ScreenRect) -> bool {
        let (xmin_s, ymin_s) = self.lower_left;
        let (xmax_s, ymax_s) = self.upper_right;
        let (xmin_o, ymin_o) = big_rect.lower_left;
        let (xmax_o, ymax_o) = big_rect.upper_right;

        if xmin_s < xmin_o {
            return false;
        }
        if xmax_s > xmax_o {
            return false;
        }
        if ymin_s < ymin_o {
            return false;
        }
        if ymax_s > ymax_o {
            return false;
        }

        true
    }

    fn width(&self) -> f64 {
        self.upper_right.0 - self.lower_left.0
    }

    fn height(&self) -> f64 {
        self.upper_right.1 - self.lower_left.1
    }
}

fn calculate_plot_edges(ac: &AppContext) -> ScreenRect {

    let (lower_left_screen, upper_right_screen) = ac.bounding_box_in_screen_coords();
    let (mut screen_x_min, mut screen_y_min) = lower_left_screen;
    let (mut screen_x_max, mut screen_y_max) = upper_right_screen;

    // If screen area is bigger than plot area, labels will be clipped, keep them on the plot
    let (xmin, ymin) = ac.convert_xy_to_screen((0.0, 0.0));
    let (xmax, ymax) = ac.convert_xy_to_screen((1.0, 1.0));

    if xmin > screen_x_min {
        screen_x_min = xmin;
    }
    if xmax < screen_x_max {
        screen_x_max = xmax;
    }
    if ymax < screen_y_max {
        screen_y_max = ymax;
    }
    if ymin > screen_y_min {
        screen_y_min = ymin;
    }

    // Add some padding to keep away from the window edge
    screen_x_max -= config::DEFAULT_PADDING;
    screen_y_max -= config::DEFAULT_PADDING;
    screen_x_min += config::DEFAULT_PADDING;
    screen_y_min += config::DEFAULT_PADDING;

    ScreenRect {
        lower_left: (screen_x_min, screen_y_min),
        upper_right: (screen_x_max, screen_y_max),
    }
}

fn collect_labels(cr: &Context, ac: &AppContext) -> Vec<(String, ScreenRect)> {
    let mut labels = vec![];

    let screen_edges = calculate_plot_edges(ac);
    #[allow(unused_variables)]
    let ScreenRect {
        lower_left,
        upper_right,
    } = screen_edges;

    for &p in config::ISOBARS.into_iter() {

        let label = format!("{}", p);

        let extents = cr.text_extents(&label);

        let (_, screen_y) = ac.convert_tp_to_screen((0.0, p));
        let screen_y = screen_y - extents.height / 2.0;

        let label_lower_left = (lower_left.0, screen_y);
        let label_upper_right = (lower_left.0 + extents.width, screen_y + extents.height);

        let pair = (
            label,
            ScreenRect {
                lower_left: label_lower_left,
                upper_right: label_upper_right,
            },
        );

        check_overlap_then_add(&mut labels, &screen_edges, pair);
    }

    let (_, screen_max_p) = ac.convert_screen_to_tp(lower_left);
    for &t in config::ISOTHERMS.into_iter() {

        let label = format!("{}", t);

        let extents = cr.text_extents(&label);

        let (mut xpos, mut ypos) = ac.convert_tp_to_screen((t, screen_max_p));
        xpos -= extents.width / 2.0; // Center
        ypos -= extents.height / 2.0; // Center
        ypos += extents.height; // Move up off bottom axis.
        xpos += extents.height; // Move right for 45 degree angle from move up

        let label_lower_left = (xpos, ypos);
        let label_upper_right = (xpos + extents.width, ypos + extents.height);

        let pair = (
            label,
            ScreenRect {
                lower_left: label_lower_left,
                upper_right: label_upper_right,
            },
        );
        check_overlap_then_add(&mut labels, &screen_edges, pair);
    }

    labels
}

fn check_overlap_then_add(
    vector: &mut Vec<(String, ScreenRect)>,
    plot_edges: &ScreenRect,
    label_pair: (String, ScreenRect),
) {

    // Make sure itis on screen
    if !label_pair.1.inside(plot_edges) {
        return;
    }

    // Check for overlap
    for &(_, ref rect) in vector.iter() {
        if label_pair.1.overlaps(&rect) {
            return;
        }
    }

    vector.push(label_pair);
}

fn draw_labels(cr: &Context, labels: Vec<(String, ScreenRect)>) {

    // FIXME: Move to config, and use when checking overlaps?
    const PADDING_PIXELS: f64 = 3.0;
    let (padding, _) = cr.device_to_user_distance(PADDING_PIXELS, PADDING_PIXELS);

    for (label, rect) in labels {
        // FIXME: Handle this by using better destructuring and an underscore
        #[allow(unused_variables)]
        let ScreenRect {
            lower_left,
            upper_right,
        } = rect;

        let rgb = config::BACKGROUND_RGB;
        cr.set_source_rgb(rgb.0, rgb.1, rgb.2);
        cr.rectangle(
            lower_left.0 - padding,
            lower_left.1 - padding,
            rect.width() + 2.0 * padding,
            rect.height() + 2.0 * padding,
        );
        cr.fill();

        // Setup label colors
        // FIXME: Better way of handling color
        let rgba = config::ISOBAR_RGBA;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, 1.0);
        cr.move_to(lower_left.0, lower_left.1);
        cr.show_text(&label);
    }
}

// Add a description box
pub fn draw_legend(cr: &Context, ac: &AppContext) {
    if !ac.plottable() {
        return;
    }

    let mut upper_left = ac.convert_device_to_screen((5.0, 5.0));
    // Make sure we stay on the x-y coords domain
    let (xmin, ymax) = ac.convert_xy_to_screen((0.0, 1.0));
    if ymax - upper_left.0 < upper_left.1 {
        upper_left.1 = ymax - upper_left.0;
    }

    if xmin + upper_left.0 > upper_left.0 {
        upper_left.0 = xmin + upper_left.0;
    }

    let font_extents = cr.font_extents();

    let (source_name, valid_time, location) = build_legend_strings(ac);

    // FIXME: Use ScreenRect
    let (box_width, box_height) =
        calculate_legend_box_size(cr, &font_extents, &source_name, &valid_time, &location);

    draw_legend_rectangle(cr, &upper_left, box_width, box_height);

    // FIXME: Use ScreenRect
    draw_legend_text(
        cr,
        &upper_left,
        &font_extents,
        &source_name,
        &valid_time,
        &location,
    );
}

fn build_legend_strings(ac: &AppContext) -> (Option<String>, Option<String>, Option<String>) {

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

// FIXME: Use ScreenRect
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
