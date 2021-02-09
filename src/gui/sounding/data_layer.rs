use super::SkewTContext;
use crate::{
    analysis::{Analysis, PrecipTypeAlgorithm},
    app::config::{self},
    coords::{ScreenCoords, TPCoords, XYCoords},
    gui::{
        utility::{draw_filled_polygon, plot_curve_from_points},
        Drawable, DrawingArgs, PlotContextExt,
    },
};
use itertools::izip;
use log::warn;
use metfor::{Celsius, Fahrenheit, JpKg, Quantity};
use sounding_analysis::{self, Parcel, ParcelAscentAnalysis};

mod precip_type;

#[derive(Clone, Copy, Debug)]
enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
}

impl SkewTContext {
    pub fn draw_temperature_profiles(args: DrawingArgs<'_, '_>) {
        let config = args.ac.config.borrow();

        if config.show_wet_bulb {
            Self::draw_temperature_profile(TemperatureType::WetBulb, args);
        }

        if config.show_dew_point {
            Self::draw_temperature_profile(TemperatureType::DewPoint, args);
        }

        if config.show_temperature {
            Self::draw_temperature_profile(TemperatureType::DryBulb, args);
        }
    }

    fn draw_temperature_profile(t_type: TemperatureType, args: DrawingArgs<'_, '_>) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let anal0 = if let Some(anal) = ac.get_sounding0_for_display() {
            anal
        } else {
            return;
        };

        let anal = anal0.borrow();

        let sndg = anal.sounding();
        let pres_data = sndg.pressure_profile();
        let temp_data = match t_type {
            TemperatureType::DryBulb => sndg.temperature_profile(),
            TemperatureType::WetBulb => sndg.wet_bulb_profile(),
            TemperatureType::DewPoint => sndg.dew_point_profile(),
        };

        let line_width = match t_type {
            TemperatureType::DryBulb => config.temperature_line_width,
            TemperatureType::WetBulb => config.wet_bulb_line_width,
            TemperatureType::DewPoint => config.dew_point_line_width,
        };

        let line_rgba = (0.0, 0.0, 0.0, 1.0);

        let profile_data = izip!(pres_data, temp_data).filter_map(|(pres, temp)| {
            if let (Some(pressure), Some(temperature)) = (pres.into(), temp.into()) {
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
        });

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);

        let anal1 = if let Some(anal) = ac.get_sounding1_for_display() {
            anal
        } else {
            return;
        };

        let anal = anal1.borrow();

        let sndg = anal.sounding();
        let pres_data = sndg.pressure_profile();
        let temp_data = match t_type {
            TemperatureType::DryBulb => sndg.temperature_profile(),
            TemperatureType::WetBulb => sndg.wet_bulb_profile(),
            TemperatureType::DewPoint => sndg.dew_point_profile(),
        };

        let line_width = match t_type {
            TemperatureType::DryBulb => config.temperature_line_width,
            TemperatureType::WetBulb => config.wet_bulb_line_width,
            TemperatureType::DewPoint => config.dew_point_line_width,
        };

        let line_rgba = (1.0, 0.5, 0.0, 1.0);

        let profile_data = izip!(pres_data, temp_data).filter_map(|(pres, temp)| {
            if let (Some(pressure), Some(temperature)) = (pres.into(), temp.into()) {
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
        });

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    }

    pub fn draw_data_overlays(args: DrawingArgs<'_, '_>) {
    }

    fn draw_cape_cin_fill(args: DrawingArgs<'_, '_>, parcel_analysis: &ParcelAscentAnalysis) {
    }

    fn draw_downburst(args: DrawingArgs<'_, '_>, sounding_analysis: &Analysis) {
    }

    pub fn draw_precip_icons(&self, args: DrawingArgs<'_, '_>) {
    }
}
