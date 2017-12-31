//! Functions used for adding an active readout/sampling box.
use app::AppContext;
use coords::{DeviceCoords, ScreenCoords, ScreenRect, TPCoords, XYCoords};
use gui::plot_context::PlotContext;
use gui::{DrawingArgs, set_font_size};

use cairo::{Context, FontFace, FontSlant, FontWeight};

use sounding_base::{DataRow, Sounding};

pub fn draw_active_sample(args: DrawingArgs) {

    let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

    let font_face = FontFace::toy_create(&config.font_name, FontSlant::Normal, FontWeight::Bold);
    cr.set_font_face(font_face);

    set_font_size(&ac.skew_t, config.label_font_size, cr);

    let vals = if let Some(vals) = ac.get_sample() {
        vals
    } else {
        return;
    };

    let snd = if let Some(snd) = ac.get_sounding_for_display() {
        snd
    } else {
        return;
    };

    let sample_p = if let Some(sample_p) = vals.pressure.as_option() {
        sample_p
    } else {
        return;
    };

    let lines = create_text(&vals, &snd, ac);

    draw_sample_line(args, sample_p);

    let box_rect = calculate_screen_rect(args, &lines, sample_p);

    draw_sample_readout_text_box(&box_rect, cr, ac, &lines);
}

fn create_text(vals: &DataRow, snd: &Sounding, _ac: &AppContext) -> Vec<String> {
    let mut results = vec![];

    let t_c = vals.temperature.as_option();
    let dp_c = vals.dew_point.as_option();
    let pres = vals.pressure.as_option();
    let dir = vals.direction.as_option();
    let spd = vals.speed.as_option();
    let hgt_asl = vals.height.as_option();
    let omega = vals.omega.as_option();
    let elevation = snd.get_location().2.as_option();

    if t_c.is_some() || dp_c.is_some() || omega.is_some() {
        let mut line = String::with_capacity(128);
        if let Some(t_c) = t_c {
            line.push_str(&format!("{:.0}C", t_c));
        }
        if let Some(dp_c) = dp_c {
            if t_c.is_some() {
                line.push('/');
            }
            line.push_str(&format!("{:.0}C", dp_c));
        }
        if let (Some(t_c), Some(dp_c)) = (t_c, dp_c) {
            line.push_str(&format!(" {:.0}%", 100.0 * ::formula::rh(t_c, dp_c)));
        }
        if let Some(omega) = omega {
            line.push_str(&format!(" {:.1} hPa/s", omega * 10.0));
        }
        results.push(line);
    }

    if pres.is_some() || dir.is_some() || spd.is_some() {
        let mut line = String::with_capacity(128);
        if let Some(pres) = pres {
            line.push_str(&format!("{:.0}hPa", pres));
        }
        if let Some(dir) = dir {
            if pres.is_some() {
                line.push(' ');
            }
            let dir = (dir / 10.0).round() * 10.0;
            line.push_str(&format!("{:03.0}", dir));
        }
        if let Some(spd) = spd {
            if pres.is_some() && dir.is_none() {
                line.push(' ');
            }
            line.push_str(&format!("{:02.0}KT", spd));
        }
        results.push(line);
    }

    if let Some(hgt) = hgt_asl {
        results.push(format!("ASL: {:5.0}m ({:5.0}ft)", hgt, 3.28084 * hgt));
    }

    if elevation.is_some() && hgt_asl.is_some() {
        if let (Some(elev), Some(hgt)) = (elevation, hgt_asl) {
            let mut line = String::with_capacity(128);
            line.push_str(&format!(
                "AGL: {:5.0}m ({:5.0}ft)",
                hgt - elev,
                3.28084 * (hgt - elev)
            ));
            results.push(line);
        }
    }

    // Sample the screen coords. Leave these commented out for debugging later possibly.
    // {
    //     use app::PlotContext;
    //     if let Some(pnt) = _ac.skew_t.last_cursor_position_skew_t {
    //         let mut line = String::with_capacity(128);
    //         line.push_str(&format!(
    //             "col: {:3.0} row: {:3.0}",
    //             pnt.col,
    //             pnt.row
    //         ));
    //         results.push(line);
    //         let mut line = String::with_capacity(128);
    //         let pnt = _ac.skew_t.convert_device_to_screen(pnt);
    //         line.push_str(&format!(
    //             "screen x: {:.3} y: {:.3}",
    //             pnt.x,
    //             pnt.y
    //         ));
    //         results.push(line);
    //         let mut line = String::with_capacity(128);
    //         let pnt2 = _ac.skew_t.convert_screen_to_xy(pnt);
    //         line.push_str(&format!(
    //             "x: {:.3} y: {:.3}",
    //             pnt2.x,
    //             pnt2.y
    //         ));
    //         results.push(line);
    //         let mut line = String::with_capacity(128);
    //         let pnt = _ac.skew_t.convert_screen_to_tp(pnt);
    //         line.push_str(&format!(
    //             "t: {:3.0} p: {:3.0}",
    //             pnt.temperature,
    //             pnt.pressure
    //         ));
    //         results.push(line);
    //     }
    // }

    results
}

fn draw_sample_line(args: DrawingArgs, sample_p: f64) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    let rgba = config.active_readout_line_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(
        cr.device_to_user_distance(config.active_readout_line_width, 0.0)
            .0,
    );
    let start = ac.skew_t.convert_tp_to_screen(TPCoords {
        temperature: -200.0,
        pressure: sample_p,
    });
    let end = ac.skew_t.convert_tp_to_screen(TPCoords {
        temperature: 60.0,
        pressure: sample_p,
    });
    cr.move_to(start.x, start.y);
    cr.line_to(end.x, end.y);
    cr.stroke();
}

fn calculate_screen_rect(args: DrawingArgs, strings: &[String], sample_p: f64) -> ScreenRect {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    let mut width: f64 = 0.0;
    let mut height: f64 = 0.0;

    let font_extents = cr.font_extents();

    for line in strings.iter() {
        let line_extents = cr.text_extents(line);
        if line_extents.width > width {
            width = line_extents.width;
        }
        height += font_extents.height;
    }

    let (padding, _) = cr.device_to_user_distance(config.edge_padding, 0.0);

    width += 2.0 * padding;
    height += 2.0 * padding;

    let ScreenCoords { x: mut left, .. } = ac.skew_t
        .convert_device_to_screen(DeviceCoords { col: 5.0, row: 5.0 });
    let ScreenCoords { y: top, .. } = ac.skew_t.convert_tp_to_screen(TPCoords {
        temperature: 0.0,
        pressure: sample_p,
    });
    let mut bottom = top - height;

    let ScreenCoords { x: xmin, y: ymin } =
        ac.skew_t.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
    let ScreenCoords { x: xmax, y: ymax } =
        ac.skew_t.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });

    // Prevent clipping
    if left < xmin {
        left = xmin;
    }
    if left > xmax - width {
        left = xmax - width;
    }
    if bottom < ymin {
        bottom = ymin;
    }
    if bottom > ymax - height {
        bottom = ymax - height;
    }

    // Keep it on the screen
    let ScreenRect {
        lower_left: ScreenCoords { x: xmin, y: ymin },
        upper_right: ScreenCoords { x: xmax, y: ymax },
    } = ac.skew_t.bounding_box_in_screen_coords();
    if left < xmin {
        left = xmin;
    }
    if left > xmax - width {
        left = xmax - width;
    }
    if bottom < ymin {
        bottom = ymin;
    }
    if bottom > ymax - height {
        bottom = ymax - height;
    }

    let lower_left = ScreenCoords { x: left, y: bottom };
    let top_right = ScreenCoords {
        x: left + width,
        y: bottom + height,
    };

    ScreenRect {
        lower_left: lower_left,
        upper_right: top_right,
    }
}

fn draw_sample_readout_text_box(
    rect: &ScreenRect,
    cr: &Context,
    ac: &AppContext,
    lines: &[String],
) {
    let config = ac.config.borrow();

    let ScreenRect {
        lower_left: ScreenCoords { x: xmin, y: ymin },
        upper_right: ScreenCoords { x: xmax, y: ymax },
    } = *rect;

    let rgba = config.background_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.rectangle(xmin, ymin, xmax - xmin, ymax - ymin);
    cr.fill_preserve();
    let rgba = config.label_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
    cr.stroke();

    let (padding, _) = cr.device_to_user_distance(config.edge_padding, 0.0);

    let font_extents = cr.font_extents();
    let mut lines_drawn = 0.0;

    for line in lines {
        cr.move_to(
            xmin + padding,
            ymax - padding - font_extents.ascent - font_extents.height * lines_drawn,
        );
        cr.show_text(line);
        lines_drawn += 1.0;
    }
}
