use super::SkewTContext;
use crate::{
    analysis::Analysis,
    app::config::{self, Config, Rgba},
    coords::TPCoords,
    gui::{
        utility::{draw_filled_polygon, plot_curve_from_points},
        Drawable, DrawingArgs,
    },
};
use itertools::izip;
use metfor::{rh, Fahrenheit, Feet, Quantity};
use sounding_analysis::{
    self, experimental::fire::PlumeAscentAnalysis, DataRow, Parcel, ParcelAscentAnalysis,
    ParcelProfile,
};

impl SkewTContext {
    pub fn create_active_readout_text_sounding(
        data: &DataRow,
        anal: &Analysis,
        pcl_anal: &Option<ParcelAscentAnalysis>,
        config: &Config,
        results: &mut Vec<(String, Rgba)>,
    ) {
        let default_color = config.label_rgba;

        let t_c = data.temperature;
        let dp_c = data.dew_point;
        let pres = data.pressure;
        let wind = data.wind;
        let hgt_asl = data.height;
        let omega = data.pvv;
        let elevation = anal.sounding().station_info().elevation();

        if t_c.is_some() || dp_c.is_some() || omega.is_some() {
            if let Some(t_c) = t_c.into_option() {
                let mut line = String::with_capacity(10);
                line.push_str(&format!("{:.0}\u{00B0}C", t_c.unpack().round()));
                if dp_c.is_none() && omega.is_none() {
                    line.push('\n');
                } else if dp_c.is_none() {
                    line.push(' ');
                }
                results.push((line, config.temperature_rgba));
            }
            if let Some(dp_c) = dp_c.into_option() {
                if t_c.is_some() {
                    results.push(("/".to_owned(), default_color));
                }
                let mut line = String::with_capacity(10);
                line.push_str(&format!("{:.0}\u{00B0}C", dp_c.unpack().round()));
                if t_c.is_none() && omega.is_none() {
                    line.push('\n');
                } else {
                    line.push(' ');
                }
                results.push((line, config.dew_point_rgba));
            }

            if let (Some(t_c), Some(dp_c)) = (t_c.into_option(), dp_c.into_option()) {
                if let Some(rh) = rh(t_c, dp_c) {
                    let mut line = String::with_capacity(5);
                    line.push_str(&format!(" {:.0}%", 100.0 * rh));
                    if omega.is_none() {
                        line.push('\n');
                    } else {
                        line.push(' ');
                    }
                    results.push((line, config.rh_rgba));
                }
            }

            if let Some(omega) = omega.into_option() {
                results.push((
                    format!(" {:.1} Pa/s\n", (omega.unpack() * 10.0).round() / 10.0),
                    config.omega_rgba,
                ));
            }
        }

        if pres.is_some() || wind.is_some() {
            if let Some(pres) = pres.into_option() {
                let mut line = String::with_capacity(10);
                line.push_str(&format!("{:.0}hPa", pres.unpack()));
                if wind.is_none() {
                    line.push('\n');
                } else {
                    line.push(' ');
                }
                results.push((line, config.isobar_rgba));
            }
            if let Some(wind) = wind.into_option() {
                results.push((
                    format!(
                        "{:03.0} {:02.0}KT\n",
                        wind.direction,
                        wind.speed.unpack().round()
                    ),
                    config.wind_rgba,
                ));
            }
        }

        if let Some(hgt) = hgt_asl.into_option() {
            let color = config.active_readout_line_rgba;

            results.push((
                format!(
                    "ASL: {:5.0}m ({:5.0}ft)\n",
                    hgt.unpack().round(),
                    Feet::from(hgt).unpack().round()
                ),
                color,
            ));
        }

        if elevation.is_some() && hgt_asl.is_some() {
            if let (Some(elev), Some(hgt)) = (elevation.into_option(), hgt_asl.into_option()) {
                let color = config.active_readout_line_rgba;
                let mut line = String::with_capacity(128);
                line.push_str(&format!(
                    "AGL: {:5.0}m ({:5.0}ft)\n",
                    (hgt - elev).unpack().round(),
                    Feet::from(hgt - elev).unpack().round(),
                ));
                results.push((line, color));
            }
        }

        if config.show_sample_parcel_profile {
            if let Some(ref pcl_anal) = pcl_anal {
                let mut line = String::with_capacity(32);
                let color = config.parcel_positive_rgba;
                if let Some(cape) = pcl_anal.cape().into_option() {
                    line.push_str(&format!("CAPE: {:.0} J/Kg ", cape.unpack()));
                } else {
                    line.push_str("CAPE: 0 J/Kg ");
                }
                results.push((line, color));

                let mut line = String::with_capacity(32);
                let color = config.parcel_negative_rgba;
                if let Some(cin) = pcl_anal.cin().into_option() {
                    line.push_str(&format!("CIN: {:.0} J/Kg\n", cin.unpack()));
                } else {
                    line.push_str("CIN: 0 J/Kg\n");
                }
                results.push((line, color));
            }
        }
    }

    pub fn create_active_readout_text_plume(
        parcel_low: &Parcel,
        anal: &Analysis,
        plume_anal_low: &PlumeAscentAnalysis,
        plume_anal_high: &PlumeAscentAnalysis,
        config: &Config,
        results: &mut Vec<(String, Rgba)>,
    ) {
        let default_color = config.label_rgba;

        let t_c = parcel_low.temperature;
        let starting_t_c = anal
            .starting_parcel_for_blow_up_anal()
            .map(|pcl| pcl.temperature);
        let delta_t_c = starting_t_c.map(|stc| t_c - stc);

        if let Some(delta_t) = delta_t_c {
            let mut line = String::with_capacity(10);
            line.push_str(&format!("∆T {:.1}\u{00B0}C\n", delta_t.unpack()));
            results.push((line, default_color));
        }

        if config.show_sample_parcel_profile {
            let mut line = String::with_capacity(32);
            let color = config.parcel_positive_rgba;
            if let (Some(cape_low), Some(cape_high)) = (
                plume_anal_low.max_int_buoyancy.into_option(),
                plume_anal_high.max_int_buoyancy.into_option(),
            ) {
                line.push_str(&format!(
                    "Net CAPE: {:.0} - {:.0} J/Kg\n",
                    cape_high.unpack(),
                    cape_low.unpack()
                ));
            } else {
                line.push_str("Net CAPE: 0 J/Kg\n");
            }
            results.push((line, color));
            let mut line = String::with_capacity(32);
            if let (Some(el_low), Some(el_high)) = (
                plume_anal_low.el_height.into_option(),
                plume_anal_high.el_height.into_option(),
            ) {
                line.push_str(&format!(
                    "LMIB: {:.0} - {:.0} m\n",
                    el_high.unpack(),
                    el_low.unpack()
                ));
            }
            results.push((line, default_color));
            let mut line = String::with_capacity(32);
            if let (Some(mh_low), Some(mh_high)) = (
                plume_anal_low.max_height.into_option(),
                plume_anal_high.max_height.into_option(),
            ) {
                line.push_str(&format!(
                    "Max Height: {:.0} - {:.0} m\n",
                    mh_high.unpack(),
                    mh_low.unpack()
                ));
            }
            results.push((line, default_color));
        }
    }

    pub fn draw_plume_parcel_profiles(
        args: DrawingArgs<'_, '_>,
        parcel_low: Parcel,
        profile_low: &ParcelProfile,
        profile_high: &ParcelProfile,
    ) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let color = config.fire_plume_line_color;

        let pres_up = &profile_low.pressure;
        let temp_up = &profile_low.parcel_t;
        let pres_down = &profile_high.pressure;
        let temp_down = &profile_high.parcel_t;

        let upside = izip!(pres_up, temp_up);
        let downside = izip!(pres_down, temp_down).rev();
        let polygon = upside.chain(downside);

        let polygon = polygon.map(|(&pressure, &temperature)| {
            let tp_coords = TPCoords {
                temperature,
                pressure,
            };
            ac.skew_t.convert_tp_to_screen(tp_coords)
        });

        let mut polygon_rgba = color;
        polygon_rgba.3 /= 2.0;

        draw_filled_polygon(cr, polygon_rgba, polygon);
        // Draw lines
        Self::draw_plume_parcel_profile(args, &profile_low, color);
        Self::draw_plume_parcel_profile(args, &profile_high, color);

        // Draw a sample point
        if config.show_active_readout_line {
            let point = TPCoords {
                temperature: parcel_low.temperature,
                pressure: parcel_low.pressure,
            };
            let point = ac.skew_t.convert_tp_to_screen(point);
            let rgba = config.active_readout_line_rgba;

            Self::draw_point(point, rgba, args);
        }
    }

    fn draw_plume_parcel_profile(
        args: DrawingArgs<'_, '_>,
        profile: &ParcelProfile,
        line_rgba: Rgba,
    ) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let pres_data = &profile.pressure;
        let temp_data = &profile.parcel_t;

        let line_width = config.profile_line_width;

        let profile_data = izip!(pres_data, temp_data).filter_map(|(&pressure, &temperature)| {
            if pressure > config::MINP {
                let tp_coords = TPCoords {
                    temperature,
                    pressure,
                };
                Some(ac.skew_t.convert_tp_to_screen(tp_coords))
            } else {
                None
            }
        });

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    }

    pub fn draw_sample_parcel_profile(
        args: DrawingArgs<'_, '_>,
        parcel_analysis: &Option<ParcelAscentAnalysis>,
    ) {
        if let Some(ref parcel_analysis) = parcel_analysis {
            let config = args.ac.config.borrow();

            // build the parcel profile
            let profile = parcel_analysis.profile();
            let color = config.sample_parcel_profile_color;
            Self::draw_parcel_profile(args, &profile, color);
        }
    }

    pub fn draw_sample_mix_down_profile(args: DrawingArgs<'_, '_>, sample_parcel: Parcel) {
        let ac = args.ac;
        let config = ac.config.borrow();

        let anal = if let Some(anal) = ac.get_sounding_for_display() {
            anal
        } else {
            return;
        };

        let anal = anal.borrow();
        let sndg = anal.sounding();

        // build the parcel profile
        let profile = if let Ok(profile) = sounding_analysis::mix_down(sample_parcel, sndg) {
            profile
        } else {
            return;
        };

        let color = config.sample_mix_down_rgba;

        Self::draw_parcel_profile(args, &profile, color);

        if let (Some(&pressure), Some(&temperature)) =
            (profile.pressure.get(0), profile.parcel_t.get(0))
        {
            let pos = ac.skew_t.convert_tp_to_screen(TPCoords {
                temperature,
                pressure,
            });
            let deg_f = format!(
                "{:.0}\u{00B0}F",
                Fahrenheit::from(temperature).unpack().round()
            );
            ac.skew_t.draw_tag(
                &format!("{}/{:.0}\u{00B0}C", deg_f, temperature.unpack().round()),
                pos,
                color,
                args,
            );
        }
    }
}
