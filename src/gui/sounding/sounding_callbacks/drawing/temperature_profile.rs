use cairo::Context;

use app::AppContext;
use config;
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
            TemperatureType::DryBulb => config::TEMPERATURE_LINE_WIDTH,
            TemperatureType::WetBulb => config::WET_BULB_LINE_WIDTH,
            TemperatureType::DewPoint => config::DEW_POINT_LINE_WIDTH,
        };

        let line_rgba = match t_type {
            TemperatureType::DryBulb => config::TEMPERATURE_RGBA,
            TemperatureType::WetBulb => config::WET_BULB_RGBA,
            TemperatureType::DewPoint => config::DEW_POINT_RGBA,
        };

        let profile_data: Vec<_> = pres_data
            .iter()
            .zip(temp_data.iter())
            .filter_map(|val_pair| if let (Some(p), Some(t)) =
                (val_pair.0.as_option(), val_pair.1.as_option())
            {
                if p > config::MINP { Some((t, p)) } else { None }
            } else {
                None
            })
            .collect();

        plot_curve_from_points(cr, &ac, line_width, line_rgba, &profile_data);
    }
}