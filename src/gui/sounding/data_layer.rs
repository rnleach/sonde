use super::SkewTContext;
use crate::{
    analysis::Analysis,
    app::config::{self},
    coords::{DeviceCoords, Rect, ScreenCoords, ScreenRect, TPCoords, XYCoords},
    gui::{
        utility::{draw_filled_polygon, plot_curve_from_points},
        Drawable, DrawingArgs, PlotContext, PlotContextExt,
    },
};
use itertools::izip;
use log::warn;
use metfor::{Celsius, Fahrenheit, JpKg, Mm, Quantity};
use optional::Optioned;
use sounding_analysis::{self, Parcel, ParcelAscentAnalysis};

const PRECIP_BOX_SIZE: f64 = 0.07;

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
            .and_then(|p_analysis| {
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
                    p_analysis
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
                        .and_then(|pos| {
                            ac.skew_t.draw_tag("LCL", pos, config.parcel_rgba, args);
                            Some(())
                        });

                    // LFC
                    p_analysis
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
                        .and_then(|pos| {
                            ac.skew_t.draw_tag("LFC", pos, config.parcel_rgba, args);
                            Some(())
                        });

                    // EL
                    p_analysis
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
                        .and_then(|pos| {
                            ac.skew_t.draw_tag("EL", pos, config.parcel_rgba, args);
                            Some(())
                        });
                }

                Some(())
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
            sounding_analysis::sfc_based_inversion(sndg)
                .ok()
                .and_then(|lyr| lyr) // unwrap a layer of options
                .map(|lyr| lyr.top)
                .and_then(Parcel::from_datarow)
                .and_then(|parcel| sounding_analysis::mix_down(parcel, sndg).ok())
                .and_then(|parcel_profile| {
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

                    Some(())
                });
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

                    let screen_bounds = ac.skew_t.bounding_box_in_screen_coords();
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

            bottom.and_then(|&bottom| {
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

                Some(())
            });
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

    pub fn draw_precip_icon(&self, args: DrawingArgs<'_, '_>) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let wx_symbol_code = if let Some(code) = ac.get_sounding_for_display().map(|anal| {
            let anal = anal.borrow();
            let code = anal.provider_wx_symbol_code();
            let conv_precip = anal.provider_1hr_convective_precip();
            let total_precip = anal.provider_1hr_precip();
            derived_wx_code(code, conv_precip, total_precip)
        }) {
            code
        } else {
            return;
        };

        if wx_symbol_code == 0 {
            return;
        }

        let screen = self.device_rect_to_screen_rect();

        let mut box_area = screen;
        let padding = cr.device_to_user_distance(config.edge_padding, 0.0).0;
        box_area.lower_left.x += padding + PRECIP_BOX_SIZE;
        box_area.upper_right.x = box_area.lower_left.x + PRECIP_BOX_SIZE;
        box_area.lower_left.y += PRECIP_BOX_SIZE;
        box_area.upper_right.y = box_area.lower_left.y + PRECIP_BOX_SIZE;

        Self::draw_legend_rectangle(args, &box_area);

        let box_center = ScreenCoords {
            x: box_area.lower_left.x + PRECIP_BOX_SIZE / 2.0,
            y: box_area.lower_left.y + PRECIP_BOX_SIZE / 2.0,
        };

        cr.move_to(box_center.x, box_center.y);
        match wx_symbol_code {
            60 => draw_point_symbol(cr, Mode::Convective, Intensity::Light, draw_rain_dot),
            61 => draw_point_symbol(cr, Mode::Stratiform, Intensity::Light, draw_rain_dot),
            62 => draw_point_symbol(cr, Mode::Convective, Intensity::Moderate, draw_rain_dot),
            63 => draw_point_symbol(cr, Mode::Stratiform, Intensity::Moderate, draw_rain_dot),
            64 => draw_point_symbol(cr, Mode::Convective, Intensity::Heavy, draw_rain_dot),
            65 => draw_point_symbol(cr, Mode::Stratiform, Intensity::Heavy, draw_rain_dot),

            // FIXME: draw_freezing_rain(cr, Intensity)
            66 => draw_freezing_rain(cr, Intensity::Light),
            67 => draw_freezing_rain(cr, Intensity::Moderate),

            // Add light moderate, heavy, and all showers.
            70 => draw_point_symbol(cr, Mode::Convective, Intensity::Light, draw_snowflake),
            71 => draw_point_symbol(cr, Mode::Stratiform, Intensity::Light, draw_snowflake),
            72 => draw_point_symbol(cr, Mode::Convective, Intensity::Moderate, draw_snowflake),
            73 => draw_point_symbol(cr, Mode::Stratiform, Intensity::Moderate, draw_snowflake),
            74 => draw_point_symbol(cr, Mode::Convective, Intensity::Heavy, draw_snowflake),
            75 => draw_point_symbol(cr, Mode::Stratiform, Intensity::Heavy, draw_snowflake),

            79 => draw_ice_pellets(cr),
            _ => draw_red_x(cr),
        }
    }

    // FIXME: Should this be part of PlotContext for drawing things on the screen?
    fn device_rect_to_screen_rect(&self) -> ScreenRect {
        let device_rect = self.get_device_rect();

        let mut lower_left = self.convert_device_to_screen(DeviceCoords {
            row: device_rect.max_y(),
            ..device_rect.upper_left
        });

        let mut upper_right = self.convert_device_to_screen(DeviceCoords {
            col: device_rect.max_x(),
            ..device_rect.upper_left
        });

        // Make sure we stay on the x-y coords domain
        let ScreenCoords { x: xmin, y: ymin } =
            self.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
        let ScreenCoords { x: xmax, y: ymax } =
            self.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });

        lower_left.x = lower_left.x.max(xmin);
        lower_left.y = lower_left.y.max(ymin);
        upper_right.x = upper_right.x.min(xmax);
        upper_right.y = upper_right.y.min(ymax);

        ScreenRect {
            lower_left,
            upper_right,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Stratiform,
    Convective,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Intensity {
    Light,
    Moderate,
    Heavy,
}

fn derived_wx_code(wx_code: u8, conv_precip: Optioned<Mm>, total_precip: Optioned<Mm>) -> u8 {
    let total_precip = if let Some(total_precip) = total_precip.into_option() {
        total_precip
    } else {
        return wx_code;
    };

    let mode = if let Some(conv_precip) = conv_precip.into_option() {
        if conv_precip > (total_precip - conv_precip) {
            Mode::Convective
        } else {
            Mode::Stratiform
        }
    } else {
        Mode::Stratiform
    };

    let intensity = if total_precip <= Mm(2.5) {
        Intensity::Light
    } else if total_precip <= Mm(7.6) {
        Intensity::Moderate
    } else {
        Intensity::Heavy
    };

    match wx_code {
        60 => {
            // Rain
            match mode {
                Mode::Convective => {
                    match intensity {
                        Intensity::Light => 60,    // -SHRA
                        Intensity::Moderate => 62, // SHRA
                        Intensity::Heavy => 64,    // +SHRA
                    }
                }
                Mode::Stratiform => {
                    match intensity {
                        Intensity::Light => 61,    // -RA
                        Intensity::Moderate => 63, // RA
                        Intensity::Heavy => 65,    // +RA
                    }
                }
            }
        }
        66 => {
            // Freezing rain
            match intensity {
                Intensity::Light => 66, // -FZRA
                _ => 67,                // FZRA or +FZRA
            }
        }
        70 => {
            // Snow
            match mode {
                Mode::Convective => {
                    match intensity {
                        Intensity::Light => 70,    // -SHSN
                        Intensity::Moderate => 72, // SHSN
                        Intensity::Heavy => 74,    // +SHSN
                    }
                }
                Mode::Stratiform => {
                    match intensity {
                        Intensity::Light => 71,    // -SN
                        Intensity::Moderate => 73, // SN
                        Intensity::Heavy => 75,    // +SN
                    }
                }
            }
        }
        code => code, // All other codes just get passed through, including IP
    }
}

fn draw_point_symbol<F: Fn(&cairo::Context, f64)>(
    cr: &cairo::Context,
    mode: Mode,
    inten: Intensity,
    draw_func: F,
) {
    const GRID_SIZE: f64 = PRECIP_BOX_SIZE / 5.0;
    const PNT_SIZE: f64 = GRID_SIZE / 2.0; // divide by 2.0 for radius
    const A: f64 = std::f64::consts::SQRT_2 * GRID_SIZE;

    let (x, y) = cr.get_current_point();

    if mode == Mode::Stratiform {
        match inten {
            Intensity::Light => {
                cr.move_to(x - A / 2.0, y);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x + A / 2.0, y);
                draw_func(cr, PNT_SIZE);
            }
            Intensity::Moderate => {
                const H_SQ: f64 = 3.0 * A * A / 4.0;
                let h = H_SQ.sqrt();
                let yt = (A * A + 4.0 * H_SQ) / (8.0 * h);

                cr.move_to(x, y + yt);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x + A / 2.0, y - h + yt);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x - A / 2.0, y - h + yt);
                draw_func(cr, PNT_SIZE);
            }
            Intensity::Heavy => {
                cr.move_to(x, y + GRID_SIZE);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x, y - GRID_SIZE);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x + GRID_SIZE, y);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x - GRID_SIZE, y);
                draw_func(cr, PNT_SIZE);
            }
        }
    } else {
        // Mode::Convective
        match inten {
            Intensity::Light => {
                draw_func(cr, PNT_SIZE);
            }
            Intensity::Moderate => {
                cr.move_to(x, y + A / 2.0);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x, y - A / 2.0);
                draw_func(cr, PNT_SIZE);
            }
            Intensity::Heavy => {
                draw_func(cr, PNT_SIZE);
                cr.move_to(x, y + A);
                draw_func(cr, PNT_SIZE);
                cr.move_to(x, y - A);
                draw_func(cr, PNT_SIZE);
            }
        }
    }
}

fn draw_freezing_rain(cr: &cairo::Context, intensity: Intensity) {
    use std::f64::consts::PI;

    const PNT_SIZE: f64 = PRECIP_BOX_SIZE / 7.0 / 2.0; // divide by 2.0 for radius

    cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
    cr.rel_move_to(-PRECIP_BOX_SIZE / 5.0, 0.0);
    let (x, y) = cr.get_current_point();
    cr.arc(x, y, PNT_SIZE, 0.0, 2.0 * PI);
    cr.fill();

    let radius = PRECIP_BOX_SIZE / 5.0 / 1.2;
    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);
    cr.arc_negative(x, y, radius, 5.0 * PI / 4.0, 9.0 * PI / 4.0);
    let x = x + PRECIP_BOX_SIZE / 5.0 * 2.0;
    cr.arc(x, y, radius, 5.0 * PI / 4.0, 9.0 * PI / 4.0);
    cr.stroke();

    if intensity == Intensity::Moderate {
        cr.arc(x, y, PNT_SIZE, 0.0, 2.0 * PI);
        cr.fill();
    }
}

fn draw_ice_pellets(cr: &cairo::Context) {
    use std::f64::consts::PI;
    const PNT_SIZE: f64 = PRECIP_BOX_SIZE / 7.0 / 2.0; // divide by 2.0 for radius
    #[allow(non_snake_case)]
    let TRIANGLE_HEIGHT: f64 = PRECIP_BOX_SIZE * 3.0 * 3.0f64.sqrt() / 10.0;
    const TRIANGLE_WIDTH: f64 = 3.0 * PRECIP_BOX_SIZE / 5.0;
    #[allow(non_snake_case)]
    let Y: f64 = (TRIANGLE_WIDTH * TRIANGLE_WIDTH / 4.0 + TRIANGLE_HEIGHT * TRIANGLE_HEIGHT)
        / (2.0 * TRIANGLE_HEIGHT);

    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);

    cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
    cr.rel_move_to(0.0, TRIANGLE_HEIGHT / 2.0 - Y);
    let (x, y) = cr.get_current_point();
    cr.arc(x, y, PNT_SIZE, 0.0, 2.0 * PI);
    cr.fill();

    cr.move_to(x, y);
    cr.rel_move_to(0.0, Y);
    cr.rel_line_to(TRIANGLE_WIDTH / 2.0, -TRIANGLE_HEIGHT);
    cr.rel_line_to(-TRIANGLE_WIDTH, 0.0);
    cr.close_path();
    cr.stroke();
}

fn draw_red_x(cr: &cairo::Context) {
    cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
    cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
    cr.rel_move_to(-PRECIP_BOX_SIZE / 2.0, -PRECIP_BOX_SIZE / 2.0);
    cr.rel_line_to(PRECIP_BOX_SIZE, PRECIP_BOX_SIZE);
    cr.rel_move_to(-PRECIP_BOX_SIZE, 0.0);
    cr.rel_line_to(PRECIP_BOX_SIZE, -PRECIP_BOX_SIZE);
    cr.stroke();
}

fn draw_rain_dot(cr: &cairo::Context, pnt_size: f64) {
    use std::f64::consts::PI;

    cr.set_source_rgba(0.0, 0.8, 0.0, 1.0);
    let (x, y) = cr.get_current_point();
    cr.arc(x, y, pnt_size, 0.0, 2.0 * PI);
    cr.fill();
}

fn draw_snowflake(cr: &cairo::Context, _pnt_size: f64) {
    const ANGLE: f64 = std::f64::consts::PI * 2.0 / 5.0;
    const A: f64 = PRECIP_BOX_SIZE / 5.0 / 2.0;

    cr.set_source_rgba(0.0, 0.0, 1.0, 1.0);
    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);

    cr.save();

    let (x, y) = cr.get_current_point();
    cr.translate(x, y);

    cr.rel_line_to(0.0, A);
    for _ in 0..5 {
        cr.rel_move_to(0.0, -A);
        cr.rotate(ANGLE);
        cr.rel_line_to(0.0, A);
    }

    cr.stroke();
    cr.restore();
}
