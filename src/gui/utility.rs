use gtk::cairo::Context;

use crate::app::config::Rgba;
use crate::app::AppContext;
use crate::coords::{Rect, ScreenCoords, ScreenRect};

// Draw a curve connecting a list of points.
pub fn plot_curve_from_points<I>(cr: &Context, line_width_pixels: f64, rgba: Rgba, points: I)
where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(
        cr.device_to_user_distance(line_width_pixels, 0.0)
            .unwrap()
            .0,
    );

    let mut points = points;
    if let Some(start) = points.by_ref().next() {
        cr.move_to(start.x, start.y);
        for end in points {
            cr.line_to(end.x, end.y);
        }

        cr.stroke().unwrap();
    }
}

// Draw a dashed line on the graph.
pub fn plot_dashed_curve_from_points<I>(cr: &Context, line_width_pixels: f64, rgba: Rgba, points: I)
where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_dash(&[0.02], 0.0);
    plot_curve_from_points(cr, line_width_pixels, rgba, points);
    cr.set_dash(&[], 0.0);
}

// Draw a filled polygon
pub fn draw_filled_polygon<I>(cr: &Context, rgba: Rgba, points: I)
where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

    let mut points = points;
    if let Some(start) = points.by_ref().next() {
        cr.move_to(start.x, start.y);
        for end in points {
            cr.line_to(end.x, end.y);
        }
        cr.line_to(start.x, start.y);

        cr.close_path();

        cr.fill().unwrap();
    }
}

// Draw a horizontal bar graph like is done for RH and clouds.
pub fn draw_horizontal_bars<I>(cr: &Context, line_width_pixels: f64, rgba: Rgba, profile: I)
where
    I: Iterator<Item = ScreenCoords>,
{
    cr.push_group();
    cr.set_operator(gtk::cairo::Operator::Source);
    cr.set_line_width(
        cr.device_to_user_distance(line_width_pixels, 0.0)
            .unwrap()
            .0,
    );
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

    let mut profile = profile;
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
        } else if let (None, None, None) = (previous, curr, next) {
            // This means there was absolutely nothing in the iterator.
            break;
        } else {
            // Impossible state
            unreachable!();
        }

        cr.rectangle(XMIN, ymin, xmax, ymax - ymin);
        cr.fill_preserve().unwrap();
        cr.stroke().unwrap();
    }

    cr.pop_group_to_source().unwrap();
    cr.paint().unwrap();
}

pub fn check_overlap_then_add(
    cr: &Context,
    ac: &AppContext,
    vector: &mut Vec<(String, ScreenRect)>,
    plot_edges: &ScreenRect,
    label_pair: (String, ScreenRect),
) {
    let padding = cr
        .device_to_user_distance(ac.config.borrow().label_padding, 0.0)
        .unwrap()
        .0;
    let padded_rect = label_pair.1.add_padding(padding);

    // Make sure it is on screen - but don't add padding to this check cause the screen already
    // has padding.
    if !label_pair.1.inside(plot_edges) {
        return;
    }

    // Check for overlap
    for (_, rect) in vector.iter() {
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
