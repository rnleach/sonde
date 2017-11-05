
use cairo::Context;
use gtk::{DrawingArea, Inhibit, WidgetExt};

use app::{AppContext, AppContextPointer};

/// Draws the sounding, connected to the on-draw event signal.
pub fn draw_omega(
    omega_area: &DrawingArea,
    cr: &Context,
    ac: &AppContextPointer,
) -> Inhibit {

    let mut ac = ac.borrow_mut();

    prepare_to_draw(omega_area, cr, &mut ac);

    // Fill
    let alloc = omega_area.get_allocation();
    cr.rectangle(0.0, 0.0, alloc.width as f64, alloc.height as f64);
    cr.set_source_rgb(0.0, 0.0, 1.0);
    cr.fill();

    Inhibit(false)
}

pub fn prepare_to_draw(omega_area: &DrawingArea, cr: &Context, ac: &mut AppContext) {
    // Get the dimensions of the DrawingArea
    // let alloc = omega_area.get_allocation();
    // ac.device_width = alloc.width;
    // ac.device_height = alloc.height;
    // let scale_factor = ac.scale_factor();

    // // Fill with backgound color
    // cr.rectangle(0.0, 0.0, ac.device_width as f64, ac.device_height as f64);
    // cr.set_source_rgba(
    //     ac.config.background_rgba.0,
    //     ac.config.background_rgba.1,
    //     ac.config.background_rgba.2,
    //     ac.config.background_rgba.3,
    // );
    // cr.fill();

    // // Set the scale factor
    // cr.scale(scale_factor, scale_factor);
    // // Set origin at lower left.
    // cr.transform(Matrix {
    //     xx: 1.0,
    //     yx: 0.0,
    //     xy: 0.0,
    //     yy: -1.0,
    //     x0: 0.0,
    //     y0: ac.device_height as f64 / scale_factor,
    // });

    // // Update the translation to center or bound the graph
    // ac.bound_view();

    // // Clip the drawing area
    // let upper_right_xy = ac.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });
    // let lower_left_xy = ac.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
    // cr.rectangle(
    //     lower_left_xy.x,
    //     lower_left_xy.y,
    //     upper_right_xy.x - lower_left_xy.x,
    //     upper_right_xy.y - lower_left_xy.y,
    // );
    // cr.clip();

    // // Calculate the various padding values
    // ac.label_padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
    // ac.edge_padding = cr.device_to_user_distance(ac.config.edge_padding, 0.0).0;
}