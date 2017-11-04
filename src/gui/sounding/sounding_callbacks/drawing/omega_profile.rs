use cairo::Context;

use gui::sounding::sounding_callbacks::drawing::plot_curve_from_points;

use app::AppContext;
use coords::{ScreenCoords, ScreenRect, OPCoords};

pub fn draw_omega_profile(cr: &Context, ac: &AppContext) {
    use app::config;

    if let Some(sndg) = ac.get_sounding_for_display() {

        let pres_data = &sndg.pressure;
        let omega_data = &sndg.omega;

        let line_width = ac.config.omega_line_width;
        let line_rgba = ac.config.omega_rgba;

        let profile_data = pres_data.iter().zip(omega_data.iter()).filter_map(
            |val_pair| {
                if let (Some(pressure), Some(omega)) =
                    (val_pair.0.as_option(), val_pair.1.as_option())
                {
                    if pressure > config::MINP {
                        let op_coords = OPCoords { omega, pressure };
                        Some(ac.convert_op_to_screen(op_coords))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        );

        draw_omega_axes(cr, ac);

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    }
}

fn draw_omega_axes(cr: &Context, ac: &AppContext) {

    let ScreenRect {
        lower_left: ScreenCoords { y: y_min, .. },
        upper_right: ScreenCoords { y: y_max, .. },
    } = ac.bounding_box_in_screen_coords();

    let ScreenCoords { x: x_min, .. } = ac.convert_op_to_screen(OPCoords {
        omega: -ac.max_abs_omega,
        pressure: 0.0,
    });

    let ScreenCoords { x: x_max, .. } = ac.convert_op_to_screen(OPCoords {
        omega: ac.max_abs_omega,
        pressure: 0.0,
    });

    let center_line = (x_max + x_min) / 2.0;

    let omega_axis = [
        ScreenCoords { x: x_min, y: y_max },
        ScreenCoords { x: x_max, y: y_max },
    ];
    let vertical_axis = [
        ScreenCoords {
            x: center_line,
            y: y_min,
        },
        ScreenCoords {
            x: center_line,
            y: y_max,
        },
    ];

    let rgba = (0.5, 0.5, 0.5, 1.0); //TODO:
    let line_width = 1.0; //TODO:

    plot_curve_from_points(cr, line_width, rgba, omega_axis.iter().cloned());
    plot_curve_from_points(cr, line_width, rgba, vertical_axis.iter().cloned());
}
