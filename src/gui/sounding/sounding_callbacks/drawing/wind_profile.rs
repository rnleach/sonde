use cairo::Context;

use app::AppContext;
use config;
use coords::{ScreenCoords, TPCoords};
use gui::ScreenRect;

pub fn draw_wind_profile(cr: &Context, ac: &AppContext) {
    // TODO: DRY - lots of repeated code here.
    // TODO: Make a wind barb struct that stores the center, box, shaft end, barb/penant enum
    // TODO: Make a private struct that stores shaft length, barb length, xcoord, dot size
    // TODO: Check for overlap and draw at same time - part of DRY
    let snd = if let Some(snd) = ac.get_sounding_for_display() {
        snd
    } else {
        return;
    };

    let dir = &snd.direction;
    let spd = &snd.speed;
    let pres = &snd.pressure;

    // FIXME: Move barb configuration to config module
    const SHAFT_LENGTH_IN_PIXELS: f64 = 25.0;
    const BARB_LENGTH_IN_PIXELS: f64 = 8.0;
    let (shaft_length, _barb_length) =
        cr.device_to_user_distance(SHAFT_LENGTH_IN_PIXELS, BARB_LENGTH_IN_PIXELS);
    let (padding, _) = cr.device_to_user_distance(config::EDGE_PADDING, 0.0);
    let ((_xmin, _ymin), (xmax, _ymax)) = ac.bounding_box_in_screen_coords();
    let barb_center_x = xmax - padding - shaft_length;

    let barb_data: Vec<(f64, f64, ScreenCoords, ScreenRect)> = izip!(pres, dir, spd)
        .filter_map(|tuple|{
            let (p, d, s) = tuple;
            if p.as_option().is_some() && d.as_option().is_some() && s.as_option().is_some() {
                Some((p.unwrap(), d.unwrap(), s.unwrap()))
            } else {
                None
            }
        })
        .map(|tuple| {
            let (p, d, s) = tuple;
            (d, s, get_wind_barb_center(p, barb_center_x, ac))
        })  // TODO: More filtering here. If it isn't on the screen, filter it out.
        .map(|tuple| {
            let (d, s, center) = tuple;
            (d, s, center, get_wind_barb_box(center, d, s, shaft_length, cr))
        })
        .collect();

    // Remove overlap
    let mut barbs_keep: Vec<(f64, f64, ScreenCoords)> = vec![];
    let mut last_kept: ScreenRect = ScreenRect {
        lower_left: (::std::f64::MAX, ::std::f64::MAX),
        upper_right: (::std::f64::MAX, ::std::f64::MAX),
    };
    for (d, s, center, rect) in barb_data {
        if rect.overlaps(&last_kept) {
            continue;
        }
        last_kept = rect;
        barbs_keep.push((d, s, center));
    }

    // Draw.
    for (d, s, center) in barbs_keep {
        draw_wind_barb(d, s, shaft_length, center, cr);
    }
}

fn get_wind_barb_center(pressure: f64, xcenter: f64, ac: &AppContext) -> ScreenCoords {

    let (_, yc) = ac.convert_tp_to_screen(TPCoords {
        temperature: 0.0,
        pressure,
    });

    (xcenter, yc)
}

fn get_wind_barb_box(
    center: ScreenCoords,
    direction: f64,
    _speed: f64,
    shaft_length: f64,
    cr: &Context,
) -> ScreenRect {

    let (dot_size, _) = cr.device_to_user_distance(8.0, 8.0);
    let mut lower_left = (center.0 - dot_size / 2.0, center.1 - dot_size / 2.0);
    let mut upper_right = (center.0 + dot_size / 2.0, center.1 + dot_size / 2.0);
    let dir = direction.to_radians();
    let (dx, dy) = (shaft_length * dir.sin(), shaft_length * dir.cos());
    let shaft_end = (center.0 + dx, center.1 + dy);

    if lower_left.0 > shaft_end.0 {
        lower_left.0 = shaft_end.0;
    }
    if upper_right.0 < shaft_end.0 {
        upper_right.0 = shaft_end.0;
    }
    if lower_left.1 > shaft_end.1 {
        lower_left.1 = shaft_end.1;
    }
    if upper_right.1 < shaft_end.1 {
        upper_right.1 = shaft_end.1;
    }

    // TODO: take barbs and penants into account

    ScreenRect {
        lower_left,
        upper_right,
    }
}

fn draw_wind_barb(
    direction: f64,
    _speed: f64,
    shaft_length: f64,
    center: ScreenCoords,
    cr: &Context,
) {


    let (dot_size, _) = cr.device_to_user_distance(6.0, 6.0);
    cr.arc(
        center.0,
        center.1,
        dot_size,
        0.0,
        2.0 * ::std::f64::consts::PI,
    );
    cr.fill();

    let dir = direction.to_radians();
    let (dx, dy) = (shaft_length * dir.sin(), shaft_length * dir.cos());
    let shaft_end = (center.0 + dx, center.1 + dy);

    cr.move_to(center.0, center.1);
    cr.line_to(shaft_end.0, shaft_end.1);
    cr.stroke();
}
