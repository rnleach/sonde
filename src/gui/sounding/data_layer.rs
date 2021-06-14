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
use metfor::{Celsius, Fahrenheit, HectoPascal, JpKg, Quantity};
use sounding_analysis::{self, Parcel, ParcelAscentAnalysis};
use std::iter::once;

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

        let anal = if let Some(anal) = ac.get_sounding_for_display() {
            anal
        } else {
            return;
        };

        let anal = anal.borrow();

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

        let line_rgba = match t_type {
            TemperatureType::DryBulb => config.temperature_rgba,
            TemperatureType::WetBulb => config.wet_bulb_rgba,
            TemperatureType::DewPoint => config.dew_point_rgba,
        };

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
        use crate::app::config::ParcelType::*;

        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let anal = if let Some(anal) = ac.get_sounding_for_display() {
            anal
        } else {
            return;
        };

        let anal = anal.borrow();
        let sndg = anal.sounding();

        if config.show_parcel_profile {
            match config.parcel_type {
                Surface => anal.surface_parcel_analysis(),
                MixedLayer => anal.mixed_layer_parcel_analysis(),
                MostUnstable => anal.most_unstable_parcel_analysis(),
                Convective => anal.convective_parcel_analysis(),
                Effective => anal.effective_parcel_analysis(),
            }
            .map(|p_analysis| {
                let color = config.parcel_rgba;
                let p_profile = p_analysis.profile();

                Self::draw_parcel_profile(args, &p_profile, color);

                if config.fill_parcel_areas {
                    Self::draw_cape_cin_fill(args, &p_analysis);
                }

                // Draw overlay tags
                if p_analysis
                    .cape()
                    .map(|cape| cape > JpKg(0.0))
                    .unwrap_or(false)
                {
                    // LCL
                    if let Some(pos) = p_analysis
                        .lcl_pressure()
                        .into_option()
                        .and_then(|p| p_analysis.lcl_temperature().map(|t| (p, t)))
                        .map(|(p, t)| {
                            let vt = metfor::virtual_temperature(t, t, p)
                                .map(Celsius::from)
                                .unwrap_or(t);
                            (p, vt)
                        })
                        .map(|(p, t)| TPCoords {
                            temperature: t,
                            pressure: p,
                        })
                        .map(|coords| {
                            let mut coords = ac.skew_t.convert_tp_to_screen(coords);
                            coords.x += 0.025;
                            coords
                        })
                    {
                        ac.skew_t.draw_tag("LCL", pos, config.parcel_rgba, args);
                    }

                    // LFC
                    if let Some(pos) = p_analysis
                        .lfc_pressure()
                        .into_option()
                        .and_then(|p| p_analysis.lfc_virt_temperature().map(|t| (p, t)))
                        .map(|(p, t)| TPCoords {
                            temperature: t,
                            pressure: p,
                        })
                        .map(|coords| {
                            let mut coords = ac.skew_t.convert_tp_to_screen(coords);
                            coords.x += 0.025;
                            coords
                        })
                    {
                        ac.skew_t.draw_tag("LFC", pos, config.parcel_rgba, args);
                    }

                    // EL
                    if let Some(pos) = p_analysis
                        .el_pressure()
                        .into_option()
                        .and_then(|p| p_analysis.el_temperature().map(|t| (p, t)))
                        .map(|(p, t)| {
                            let vt = metfor::virtual_temperature(t, t, p)
                                .map(Celsius::from)
                                .unwrap_or(t);
                            (p, vt)
                        })
                        .map(|(p, t)| TPCoords {
                            temperature: t,
                            pressure: p,
                        })
                        .map(|coords| {
                            let mut coords = ac.skew_t.convert_tp_to_screen(coords);
                            coords.x += 0.025;
                            coords
                        })
                    {
                        ac.skew_t.draw_tag("EL", pos, config.parcel_rgba, args);
                    }
                }
            })
            .or_else(|| {
                warn!("Parcel analysis returned None.");
                Some(())
            });
        }

        if config.show_downburst {
            Self::draw_downburst(args, &anal);
        }

        if config.show_inversion_mix_down {
            if let Some(parcel_profile) = sounding_analysis::sfc_based_inversion(sndg)
                .ok()
                .and_then(|lyr| lyr) // unwrap a layer of options
                .map(|lyr| lyr.top)
                .and_then(Parcel::from_datarow)
                .and_then(|parcel| sounding_analysis::mix_down(parcel, sndg).ok())
            {
                let color = config.inversion_mix_down_rgba;
                Self::draw_parcel_profile(args, &parcel_profile, color);

                if let (Some(&pressure), Some(&temperature)) = (
                    parcel_profile.pressure.get(0),
                    parcel_profile.parcel_t.get(0),
                ) {
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

        if config.show_inflow_layer {
            if let Some(lyr) = anal.effective_inflow_layer() {
                if let (Some(bottom_p), Some(top_p)) = (
                    lyr.bottom.pressure.into_option(),
                    lyr.top.pressure.into_option(),
                ) {
                    // Values from wind barbs, make this to the left of the wind barbs
                    let (shaft_length, _) = cr.device_to_user_distance(
                        config.wind_barb_shaft_length,
                        -config.wind_barb_barb_length,
                    );
                    let padding = cr.device_to_user_distance(config.edge_padding, 0.0).0;

                    let screen_bounds = ac.skew_t.get_plot_area();
                    let XYCoords { x: mut xmax, .. } =
                        ac.skew_t.convert_screen_to_xy(screen_bounds.upper_right);

                    xmax = xmax.min(1.0);

                    let ScreenCoords { x: xmax, .. } =
                        ac.skew_t.convert_xy_to_screen(XYCoords { x: xmax, y: 0.0 });

                    let xcoord = xmax - 2.0 * padding - 2.0 * shaft_length;
                    let yb = SkewTContext::get_wind_barb_center(bottom_p, xcoord, args);
                    let yt = SkewTContext::get_wind_barb_center(top_p, xcoord, args);

                    const WIDTH: f64 = 0.02;

                    let rgba = config.inflow_layer_rgba;
                    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
                    cr.set_line_width(cr.device_to_user_distance(4.0, 0.0).0);
                    cr.move_to(yt.x + WIDTH, yt.y);
                    cr.line_to(yt.x - WIDTH, yt.y);
                    cr.move_to(yt.x, yt.y);
                    cr.line_to(yb.x, yb.y);
                    cr.move_to(yb.x + WIDTH, yb.y);
                    cr.line_to(yb.x - WIDTH, yb.y);
                    cr.stroke();
                }
            }
        }

        if config.show_pft {
            Self::draw_pft_overlays(args, &anal);
        }
    }

    fn draw_pft_overlays(args: DrawingArgs<'_, '_>, sounding_anal: &Analysis) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let sp_color = config.pft_sp_curve_color;
        let mean_q_color = config.pft_mean_q_color;
        let mean_theta_color = config.pft_mean_theta_color;
        let cloud_parcel_color = config.pft_cloud_parcel_color;
        let line_width = config.pft_line_width;

        // This plots an SP-curve and the cloud parcel above the LFC.
        let plot_single_sp_curve_cloud_parcel = |pft_anal: &sounding_analysis::PFTAnalysis| {
            let sp_curve = pft_anal.sp_curve.iter().map(|(p, t)| {
                let tp_coords = TPCoords {
                    temperature: *t,
                    pressure: *p,
                };
                ac.skew_t.convert_tp_to_screen(tp_coords)
            });

            plot_curve_from_points(cr, line_width, sp_color, sp_curve);

            let theta_e_pnts = (0..100)
                .map(|i| pft_anal.p_fc - HectoPascal((i * 10) as f64))
                .take_while(|p| *p > HectoPascal(100.0))
                .map(|p| {
                    metfor::find_root(
                        &|t| {
                            Some(
                                (metfor::equiv_pot_temperature(Celsius(t), Celsius(t), p)?
                                    - pft_anal.theta_e_fc)
                                    .unpack(),
                            )
                        },
                        Celsius(-80.0).unpack(),
                        Celsius(50.0).unpack(),
                    )
                    .map(Celsius)
                    .map(|t| (p, t))
                })
                .take_while(|opt| opt.is_some())
                .flatten()
                .map(|(pressure, temperature)| {
                    let pt_coords = TPCoords {
                        temperature,
                        pressure,
                    };
                    ac.skew_t.convert_tp_to_screen(pt_coords)
                });

            plot_curve_from_points(cr, line_width, cloud_parcel_color, theta_e_pnts);
        };

        if let Some(pft_anal) = sounding_anal.pft() {
            plot_single_sp_curve_cloud_parcel(pft_anal);

            let sfc_p = match sounding_anal
                .sounding()
                .pressure_profile()
                .iter()
                .map(|p| p.into_option())
                .flatten()
                .next()
            {
                Some(p) => p,
                None => return,
            };

            let p_top = match pft_anal.sp_curve.iter().next() {
                Some((p, _)) => *p,
                None => return,
            };

            let q_iter = once(sfc_p)
                .chain(once(p_top))
                .filter_map(|p| {
                    metfor::dew_point_from_p_and_specific_humidity(p, pft_anal.q_ml)
                        .map(|dp| (dp, p))
                })
                .map(|(temperature, pressure)| TPCoords {
                    temperature,
                    pressure,
                })
                .map(|coords| ac.skew_t.convert_tp_to_screen(coords));

            plot_curve_from_points(cr, line_width, mean_q_color, q_iter);

            let theta_iter = sounding_anal
                .sounding()
                .pressure_profile()
                .iter()
                .map(|p| p.into_option())
                .flatten()
                .take_while(|p| *p > p_top)
                .chain(once(p_top))
                .map(|p| (metfor::temperature_from_pot_temp(pft_anal.theta_ml, p), p))
                .map(|(t, p)| (Celsius::from(t), p))
                .map(|(temperature, pressure)| TPCoords {
                    temperature,
                    pressure,
                })
                .map(|coords| ac.skew_t.convert_tp_to_screen(coords));

            plot_curve_from_points(cr, line_width, mean_theta_color, theta_iter);
        }
    }

    fn draw_cape_cin_fill(args: DrawingArgs<'_, '_>, parcel_analysis: &ParcelAscentAnalysis) {
        let cape = match parcel_analysis.cape().into_option() {
            Some(cape) => cape,
            None => return,
        };

        let cin = match parcel_analysis.cin().into_option() {
            Some(cin) => cin,
            None => return,
        };

        if cape <= JpKg(0.0) {
            return;
        }

        if parcel_analysis.lcl_pressure().is_none() {
            // No moist convection.
            return;
        };

        let lfc = match parcel_analysis.lfc_pressure().into_option() {
            Some(lfc) => lfc,
            None => return,
        };

        let el = match parcel_analysis.el_pressure().into_option() {
            Some(el) => el,
            None => return,
        };

        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let parcel_profile = parcel_analysis.profile();

        let pres_data = &parcel_profile.pressure;
        let parcel_t = &parcel_profile.parcel_t;
        let env_t = &parcel_profile.environment_t;

        if cin < JpKg(0.0) {
            let bottom = izip!(pres_data, parcel_t, env_t)
                // Top down
                .rev()
                .skip_while(|&(&p, _, _)| p < lfc)
                .take_while(|&(_, &p_t, &e_t)| p_t <= e_t)
                .map(|(p, _, _)| p)
                .last();

            if let Some(&bottom) = bottom {
                let up_side = izip!(pres_data, parcel_t, env_t)
                    .skip_while(|&(&p, _, _)| p > bottom)
                    .take_while(|&(&p, _, _)| p >= lfc)
                    .map(|(p, _, e_t)| (*p, *e_t));

                let down_side = izip!(pres_data, parcel_t, env_t)
                    // Top down
                    .rev()
                    // Skip above top.
                    .skip_while(|&(&p, _, _)| p < lfc)
                    // Now we're in the CIN area!
                    .take_while(|&(&p, _, _)| p < bottom)
                    .map(|(p, p_t, _)| (*p, *p_t));

                let negative_polygon = up_side.chain(down_side);

                let negative_polygon = negative_polygon.map(|(pressure, temperature)| {
                    let tp_coords = TPCoords {
                        temperature,
                        pressure,
                    };
                    ac.skew_t.convert_tp_to_screen(tp_coords)
                });

                let negative_polygon_rgba = config.parcel_negative_rgba;

                draw_filled_polygon(cr, negative_polygon_rgba, negative_polygon);
            }
        }

        let up_side = izip!(pres_data, parcel_t, env_t)
            .skip_while(|&(p, _, _)| *p > lfc)
            .take_while(|&(p, _, _)| *p >= el)
            .map(|(p, _, e_t)| (*p, *e_t));

        let down_side = izip!(pres_data, parcel_t, env_t)
            // Top down
            .rev()
            // Skip above top.
            .skip_while(|&(p, _, _)| *p < el)
            // Now we're in the CAPE area!
            .take_while(|&(p, _, _)| *p <= lfc)
            .map(|(p, p_t, _)| (*p, *p_t));

        let polygon = up_side.chain(down_side);

        let polygon = polygon.map(|(pressure, temperature)| {
            let tp_coords = TPCoords {
                temperature,
                pressure,
            };
            ac.skew_t.convert_tp_to_screen(tp_coords)
        });

        let polygon_rgba = config.parcel_positive_rgba;

        draw_filled_polygon(cr, polygon_rgba, polygon);
    }

    fn draw_downburst(args: DrawingArgs<'_, '_>, sounding_analysis: &Analysis) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let parcel_profile = if let Some(pp) = sounding_analysis.downburst_profile() {
            pp
        } else {
            return;
        };

        let color = config.downburst_rgba;
        Self::draw_parcel_profile(args, parcel_profile, color);

        if config.fill_dcape_area {
            let pres_data = &parcel_profile.pressure;
            let parcel_t = &parcel_profile.parcel_t;
            let env_t = &parcel_profile.environment_t;

            let up_side = izip!(pres_data, env_t);
            let down_side = izip!(pres_data, parcel_t).rev();

            let polygon = up_side.chain(down_side);

            let polygon = polygon.map(|(&pressure, &temperature)| {
                let tp_coords = TPCoords {
                    temperature,
                    pressure,
                };
                ac.skew_t.convert_tp_to_screen(tp_coords)
            });

            let polygon_rgba = config.dcape_area_color;

            draw_filled_polygon(cr, polygon_rgba, polygon);
        }
    }

    pub fn draw_precip_icons(&self, args: DrawingArgs<'_, '_>) {
        use PrecipTypeAlgorithm::*;

        // FIXME add options for which boxes to show.
        self.draw_precip_icon(Model, 0, args);
        self.draw_precip_icon(Bourgouin, 1, args);
        self.draw_precip_icon(NSSL, 2, args);
    }
}
