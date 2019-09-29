use super::SkewTContext;
use crate::{
    analysis::Analysis,
    app::config::{Config, Rgba},
    coords::TPCoords,
    gui::{Drawable, DrawingArgs},
};
use metfor::{rh, Fahrenheit, Feet, Quantity};
use sounding_analysis::{
    self, experimental::fire::PlumeAscentAnalysis, DataRow, Parcel, ParcelAscentAnalysis,
    ParcelProfile,
};

impl SkewTContext {
    pub fn create_active_readout_text_sounding(
        data: &DataRow,
        anal: &Analysis,
        pcl_anal: &ParcelAscentAnalysis,
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

    pub fn create_active_readout_text_plume(
        parcel: &Parcel,
        anal: &Analysis,
        plume_anal: &PlumeAscentAnalysis,
        config: &Config,
        results: &mut Vec<(String, Rgba)>,
    ) {
        let default_color = config.label_rgba;

        let t_c = parcel.temperature;
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
            if let Some(cape) = plume_anal.net_cape {
                line.push_str(&format!("Net CAPE: {:.0} J/Kg\n", cape.unpack()));
            } else {
                line.push_str("Net CAPE: 0 J/Kg\n");
            }
            results.push((line, color));
            let mut line = String::with_capacity(32);
            if let Some(el) = plume_anal.el_height {
                line.push_str(&format!("EL: {:.0} m\n", el.unpack()));
            }
            results.push((line, default_color));
            let mut line = String::with_capacity(32);
            if let Some(mh) = plume_anal.max_height {
                line.push_str(&format!("Max Height: {:.0} m\n", mh.unpack()));
            }
            results.push((line, default_color));
        }
    }

    pub fn draw_plume_parcel_profile(
        args: DrawingArgs<'_, '_>,
        parcel: Parcel,
        profile: &ParcelProfile,
    ) {
        let (ac, config) = (args.ac, args.ac.config.borrow());

        let color = config.sample_parcel_profile_color;
        Self::draw_parcel_profile(args, &profile, color);

        // Draw a sample point
        let point = TPCoords {
            temperature: parcel.temperature,
            pressure: parcel.pressure,
        };
        let point = ac.skew_t.convert_tp_to_screen(point);
        let rgba = config.active_readout_line_rgba;

        Self::draw_point(point, rgba, args);
    }

    pub fn draw_sample_parcel_profile(
        args: DrawingArgs<'_, '_>,
        parcel_analysis: &ParcelAscentAnalysis,
    ) {
        let config = args.ac.config.borrow();

        // build the parcel profile
        let profile = parcel_analysis.profile();
        let color = config.sample_parcel_profile_color;
        Self::draw_parcel_profile(args, &profile, color);
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
