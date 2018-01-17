use cairo::{Context, Matrix};

use app::AppContext;
use coords::{Rect, ScreenCoords, ScreenRect};
use gui::PlotContext;

// Draw a curve connecting a list of points.
pub fn plot_curve_from_points<I>(
    cr: &Context,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: I,
) where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);

    let mut points = points;
    if let Some(start) = points.by_ref().next() {
        cr.move_to(start.x, start.y);
        for end in points {
            cr.line_to(end.x, end.y);
        }

        cr.stroke();
    }
}

// Draw a dashed line on the graph.
pub fn plot_dashed_curve_from_points<I>(
    cr: &Context,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: I,
) where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_dash(&[0.02], 0.0);
    plot_curve_from_points(cr, line_width_pixels, rgba, points);
    cr.set_dash(&[], 0.0);
}

pub fn check_overlap_then_add(
    cr: &Context,
    ac: &AppContext,
    vector: &mut Vec<(String, ScreenRect)>,
    plot_edges: &ScreenRect,
    label_pair: (String, ScreenRect),
) {
    let padding = cr.device_to_user_distance(ac.config.borrow().label_padding, 0.0)
        .0;
    let padded_rect = label_pair.1.add_padding(padding);

    // Make sure it is on screen - but don't add padding to this check cause the screen already
    // has padding.
    if !label_pair.1.inside(plot_edges) {
        return;
    }

    // Check for overlap
    for &(_, ref rect) in vector.iter() {
        if padded_rect.overlaps(rect) {
            return;
        }
    }

    vector.push(label_pair);
}

#[derive(Clone, Copy)]
pub struct DrawingArgs<'a, 'b> {
    pub ac: &'a AppContext,
    pub cr: &'b Context,
}

impl<'a, 'b> DrawingArgs<'a, 'b> {
    pub fn new(ac: &'a AppContext, cr: &'b Context) -> Self {
        DrawingArgs { ac, cr }
    }
}

pub fn set_font_size<T: PlotContext>(pc: &T, size_in_pct: f64, cr: &Context) {
    let height = pc.get_device_rect().height();

    let mut font_size = size_in_pct / 100.0 * height;
    font_size = cr.device_to_user_distance(font_size, 0.0).0;

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