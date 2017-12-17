
use app::config;
use coords::TPCoords;
use gui::{DrawingArgs, plot_curve_from_points};

#[derive(Clone, Copy, Debug)]
pub enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
}

// Draw the temperature profile
pub fn draw_temperature_profile(t_type: TemperatureType, args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    use sounding_base::Profile::{Pressure, Temperature, WetBulb, DewPoint};

    if let Some(sndg) = ac.get_sounding_for_display() {

        let pres_data = sndg.get_profile(Pressure);
        let temp_data = match t_type {
            TemperatureType::DryBulb => sndg.get_profile(Temperature),
            TemperatureType::WetBulb => sndg.get_profile(WetBulb),
            TemperatureType::DewPoint => sndg.get_profile(DewPoint),
        };

        let line_width = match t_type {
            TemperatureType::DryBulb => config.temperature_line_width,
            TemperatureType::WetBulb => config.wet_bulb_line_width,
            TemperatureType::DewPoint => config.dew_point_line_width,
        };

        let line_rgba = match t_type {
            TemperatureType::DryBulb => config.temperature_rgba,
            TemperatureType::WetBulb => config.wet_bulb_rgba,
            TemperatureType::DewPoint => config.dew_point_rgba,
        };

        let profile_data = pres_data.iter().zip(temp_data.iter()).filter_map(
            |val_pair| {
                if let (Some(pressure), Some(temperature)) =
                    (val_pair.0.as_option(), val_pair.1.as_option())
                {
                    if pressure > config::MINP {
                        let tp_coords = TPCoords {
                            temperature,
                            pressure,
                        };
                        Some(ac.skew_t.convert_tp_to_screen(tp_coords))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        );

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    }
}
