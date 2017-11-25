use std::rc::Rc;

use cairo::{Context, Matrix};
use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use app::{AppContextPointer, AppContext, config};
use coords::{SDCoords, XYCoords};
use gui::plot_curve_from_points;


pub fn set_up_hodograph_area(hodo_area: &DrawingArea, app_context: &AppContextPointer) {

    hodo_area.set_hexpand(true);
    hodo_area.set_vexpand(true);

    let ac = Rc::clone(app_context);
    hodo_area.connect_draw(move |_da, cr| draw_hodo(cr, &ac));
}

fn draw_hodo(cr: &Context, acp: &AppContextPointer) -> Inhibit {

    let ac = &mut acp.borrow_mut();

    prepare_to_draw(cr, ac);
    draw_background(cr, ac);
    draw_hodo_line(cr, ac);

    Inhibit(false)
}

fn prepare_to_draw(cr: &Context, ac: &mut AppContext) {
    use app::PlotContext;

    // Get the dimensions of the DrawingArea
    ac.update_plot_context_allocations();
    let scale_factor = ac.hodo.scale_factor();

    // Fill with backgound color
    cr.rectangle(
        0.0,
        0.0,
        f64::from(ac.hodo.device_width),
        f64::from(ac.hodo.device_height),
    );
    cr.set_source_rgba(
        ac.config.background_rgba.0,
        ac.config.background_rgba.1,
        ac.config.background_rgba.2,
        ac.config.background_rgba.3,
    );
    cr.fill();

    // Set the scale factor
    cr.scale(scale_factor, scale_factor);
    // Set origin at lower left.
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0,
        x0: 0.0,
        y0: f64::from(ac.hodo.device_height) / scale_factor,
    });

    // Clip the drawing area
    let upper_right_xy = ac.hodo.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });
    let lower_left_xy = ac.hodo.convert_xy_to_screen(XYCoords { x: -1.0, y: -1.0 });
    cr.rectangle(
        lower_left_xy.x,
        lower_left_xy.y,
        upper_right_xy.x - lower_left_xy.x,
        upper_right_xy.y - lower_left_xy.y,
    );
    cr.clip();

    // Calculate the various padding values
    // FIXME: These should also be set in the connect_allocate callback.
    ac.hodo.label_padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
    ac.hodo.edge_padding = cr.device_to_user_distance(ac.config.edge_padding, 0.0).0;

}

fn draw_background(cr: &Context, ac: &AppContext) {

    if ac.config.show_iso_speed {
        for pnts in config::ISO_SPEED_PNTS.iter() {
            let pnts = pnts.iter().map(|sd_coords| {
                ac.hodo.convert_sd_to_screen(*sd_coords)
            });
            plot_curve_from_points(
                cr,
                ac.config.background_line_width,
                ac.config.iso_speed_rgba,
                pnts,
            );
        }
    }
}

fn draw_hodo_line(cr: &Context, ac: &AppContext) {

    use sounding_base::Profile::{Pressure, WindSpeed, WindDirection};

    if let Some(sndg) = ac.get_sounding_for_display() {

        let pres_data = sndg.get_profile(Pressure);        
        let speed_data = sndg.get_profile(WindSpeed);
        let dir_data = sndg.get_profile(WindDirection);

        let profile_data = izip!(pres_data, speed_data, dir_data).filter_map(
            |triplet| {
                if let (Some(p), Some(speed), Some(dir)) =
                    (triplet.0.as_option(), triplet.1.as_option(), triplet.2.as_option())
                {
                    if p > config::MINP {
                        let sd_coords = SDCoords {
                            speed,
                            dir,
                        };
                        Some(ac.hodo.convert_sd_to_screen(sd_coords))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        );

        plot_curve_from_points(cr, ac.config.velocity_line_width, ac.config.veclocity_rgba, profile_data);
    }
}
