use cairo::Context;

use app::{AppContext, config};
use coords::TPCoords;
use gui::sounding::sounding_callbacks::drawing::plot_curve_from_points;

pub enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
}

// Draw the temperature profile
pub fn draw_temperature_profile(t_type: TemperatureType, cr: &Context, ac: &AppContext) {

    if let Some(sndg) = ac.get_sounding_for_display() {

        let pres_data = &sndg.pressure;
        let temp_data = match t_type {
            TemperatureType::DryBulb => &sndg.temperature,
            TemperatureType::WetBulb => &sndg.wet_bulb,
            TemperatureType::DewPoint => &sndg.dew_point,
        };

        let line_width = match t_type {
            TemperatureType::DryBulb => ac.config.temperature_line_width,
            TemperatureType::WetBulb => ac.config.wet_bulb_line_width,
            TemperatureType::DewPoint => ac.config.dew_point_line_width,
        };

        let line_rgba = match t_type {
            TemperatureType::DryBulb => ac.config.temperature_rgba,
            TemperatureType::WetBulb => ac.config.wet_bulb_rgba,
            TemperatureType::DewPoint => ac.config.dew_point_rgba,
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
                        Some(ac.convert_tp_to_screen(tp_coords))
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
