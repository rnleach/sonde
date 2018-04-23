use std::rc::Rc;

use cairo::Context;
use gdk::{EventMask, EventMotion, EventScroll, ScrollDirection};
use gtk::prelude::*;
use gtk::DrawingArea;

use sounding_base::{DataRow, Sounding};

use app::{config, AppContext, AppContextPointer};
use coords::{convert_pressure_to_y, convert_y_to_pressure, DeviceCoords, Rect, ScreenCoords,
             ScreenRect, TPCoords, XYCoords};
use gui::{Drawable, DrawingArgs, MasterDrawable, PlotContext, PlotContextExt};
use gui::plot_context::{GenericContext, HasGenericContext};
use gui::utility::{check_overlap_then_add, plot_curve_from_points, plot_dashed_curve_from_points};

pub struct SkewTContext {
    generic: GenericContext,
}

impl SkewTContext {
    pub fn new() -> Self {
        SkewTContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        let y = convert_pressure_to_y(coords.pressure);
        let x = (coords.temperature - config::MINT) / (config::MAXT - config::MINT);

        // do the skew
        let x = x + y;
        XYCoords { x, y }
    }

    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        // undo the skew
        let x = coords.x - coords.y;
        let y = coords.y;

        let t = x * (config::MAXT - config::MINT) + config::MINT;
        let p = convert_y_to_pressure(y);

        TPCoords {
            temperature: t,
            pressure: p,
        }
    }

    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = Self::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        let xy = self.convert_screen_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }

    pub fn convert_device_to_tp(&self, coords: DeviceCoords) -> TPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }

    /***********************************************************************************************
     * Support methods for drawing the background..
     **********************************************************************************************/
    fn draw_temperature_banding(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.background_band_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        let mut start_line = -160i32;
        while start_line < 100 {
            let t1 = f64::from(start_line);
            let t2 = t1 + 10.0;

            self.draw_temperature_band(t1, t2, args);

            start_line += 20;
        }
    }

    fn draw_hail_growth_zone(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.hail_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        self.draw_temperature_band(-30.0, -10.0, args);
    }

    fn draw_dendtritic_growth_zone(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.dendritic_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        self.draw_temperature_band(-18.0, -12.0, args);
    }

    fn draw_temperature_band(&self, cold_t: f64, warm_t: f64, args: DrawingArgs) {
        let cr = args.cr;

        // Assume color has already been set up for us.

        const MAXP: f64 = config::MAXP;
        const MINP: f64 = config::MINP;

        let mut coords = [
            (warm_t, MAXP),
            (warm_t, MINP),
            (cold_t, MINP),
            (cold_t, MAXP),
        ];

        // Convert points to screen coords
        for coord in &mut coords {
            let screen_coords = self.convert_tp_to_screen(TPCoords {
                temperature: coord.0,
                pressure: coord.1,
            });
            coord.0 = screen_coords.x;
            coord.1 = screen_coords.y;
        }

        let mut coord_iter = coords.iter();
        for coord in coord_iter.by_ref().take(1) {
            cr.move_to(coord.0, coord.1);
        }
        for coord in coord_iter {
            cr.line_to(coord.0, coord.1);
        }

        cr.close_path();
        cr.fill();
    }
}

impl HasGenericContext for SkewTContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for SkewTContext {}

impl Drawable for SkewTContext {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.skew_t.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.skew_t.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.skew_t.button_press_event(ev));

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| ac.skew_t.button_release_event(ev));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.skew_t.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.skew_t.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| SkewTContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.skew_t.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.skew_t.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    fn draw_background_fill(&self, args: DrawingArgs) {
        let config = args.ac.config.borrow();

        if config.show_background_bands {
            self.draw_temperature_banding(args);
        }

        if config.show_hail_zone {
            self.draw_hail_growth_zone(args);
        }

        if config.show_dendritic_zone {
            self.draw_dendtritic_growth_zone(args);
        }
    }

    fn draw_background_lines(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        // Draws background lines from the bottom up.

        // Draw isentrops
        if config.show_isentrops {
            for pnts in config::ISENTROP_PNTS.iter() {
                let pnts = pnts.iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
                plot_curve_from_points(
                    cr,
                    config.background_line_width,
                    config.isentrop_rgba,
                    pnts,
                );
            }
        }

        // Draw theta-e lines
        if config.show_iso_theta_e {
            for pnts in config::ISO_THETA_E_PNTS.iter() {
                let pnts = pnts.iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
                plot_curve_from_points(
                    cr,
                    config.background_line_width,
                    config.iso_theta_e_rgba,
                    pnts,
                );
            }
        }

        // Draw mixing ratio lines
        if config.show_iso_mixing_ratio {
            for pnts in config::ISO_MIXING_RATIO_PNTS.iter() {
                let pnts = pnts.iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
                plot_dashed_curve_from_points(
                    cr,
                    config.background_line_width,
                    config.iso_mixing_ratio_rgba,
                    pnts,
                );
            }
        }

        // Draw isotherms
        if config.show_isotherms {
            for pnts in config::ISOTHERM_PNTS.iter() {
                let pnts = pnts.iter()
                    .map(|tp_coords| self.convert_xy_to_screen(*tp_coords));
                plot_curve_from_points(
                    cr,
                    config.background_line_width,
                    config.isotherm_rgba,
                    pnts,
                );
            }
        }

        // Draw isobars
        if config.show_isobars {
            for pnts in config::ISOBAR_PNTS.iter() {
                let pnts = pnts.iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));

                plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
            }
        }

        // Draw the freezing line
        if config.show_freezing_line {
            let pnts = &[
                TPCoords {
                    temperature: 0.0,
                    pressure: config::MAXP,
                },
                TPCoords {
                    temperature: 0.0,
                    pressure: config::MINP,
                },
            ];
            let pnts = pnts.iter()
                .map(|tp_coords| self.convert_tp_to_screen(*tp_coords));
            plot_curve_from_points(
                cr,
                config.freezing_line_width,
                config.freezing_line_color,
                pnts,
            );
        }
    }

    fn collect_labels(&self, args: DrawingArgs) -> Vec<(String, ScreenRect)> {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        if config.show_isobars {
            for &p in &config::ISOBARS {
                let label = format!("{}", p);

                let extents = cr.text_extents(&label);

                let ScreenCoords { y: screen_y, .. } = self.convert_tp_to_screen(TPCoords {
                    temperature: 0.0,
                    pressure: p,
                });
                let screen_y = screen_y - extents.height / 2.0;

                let label_lower_left = ScreenCoords {
                    x: lower_left.x,
                    y: screen_y,
                };
                let label_upper_right = ScreenCoords {
                    x: lower_left.x + extents.width,
                    y: screen_y + extents.height,
                };

                let pair = (
                    label,
                    ScreenRect {
                        lower_left: label_lower_left,
                        upper_right: label_upper_right,
                    },
                );

                check_overlap_then_add(cr, ac, &mut labels, &screen_edges, pair);
            }
        }

        if config.show_isotherms {
            let TPCoords {
                pressure: screen_max_p,
                ..
            } = self.convert_screen_to_tp(lower_left);
            for &t in &config::ISOTHERMS {
                let label = format!("{}", t);

                let extents = cr.text_extents(&label);

                let ScreenCoords {
                    x: mut xpos,
                    y: mut ypos,
                } = self.convert_tp_to_screen(TPCoords {
                    temperature: t,
                    pressure: screen_max_p,
                });
                xpos -= extents.width / 2.0; // Center
                ypos -= extents.height / 2.0; // Center
                ypos += extents.height; // Move up off bottom axis.
                xpos += extents.height; // Move right for 45 degree angle from move up

                let label_lower_left = ScreenCoords { x: xpos, y: ypos };
                let label_upper_right = ScreenCoords {
                    x: xpos + extents.width,
                    y: ypos + extents.height,
                };

                let pair = (
                    label,
                    ScreenRect {
                        lower_left: label_lower_left,
                        upper_right: label_upper_right,
                    },
                );
                check_overlap_then_add(cr, ac, &mut labels, &screen_edges, pair);
            }
        }

        labels
    }

    fn build_legend_strings(ac: &AppContext) -> Vec<String> {
        use chrono::Weekday::*;

        let mut result = vec![];

        if let Some(src_desc) = ac.get_source_description() {
            result.push(src_desc);
        }

        if let Some(snd) = ac.get_sounding_for_display() {
            let snd = snd.sounding();
            // Build the valid time part
            if let Some(vt) = snd.get_valid_time() {
                use chrono::{Datelike, Timelike};
                let mut temp_string = format!(
                    "Valid: {} {:02}/{:02}/{:04} {:02}Z",
                    match vt.weekday() {
                        Sun => "Sunday",
                        Mon => "Monday",
                        Tue => "Tuesday",
                        Wed => "Wednesday",
                        Thu => "Thursday",
                        Fri => "Friday",
                        Sat => "Saturday",
                    },
                    vt.month(),
                    vt.day(),
                    vt.year(),
                    vt.hour()
                );

                if let Some(lt) = snd.get_lead_time() {
                    temp_string.push_str(&format!(" F{:03}", lt));
                }

                result.push(temp_string);
            }

            // Build location part.
            let coords = snd.get_station_info().location();
            let elevation = snd.get_station_info().elevation();
            if coords.is_some() || elevation.is_some() {
                let mut location = "".to_owned();

                if let Some((lat, lon)) = coords {
                    location.push_str(&format!("{:.2}, {:.2}", lat, lon));
                    if elevation.is_some() {
                        location.push_str(", ");
                    }
                }
                if let Some(el) = elevation {
                    location.push_str(&format!("{:.0}m ({:.0}ft)", el, el * 3.28084));
                }

                result.push(location);
            }
        }

        result
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs) {
        draw_temperature_profiles(args);
        draw_wind_profile(args);
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn create_active_readout_text(vals: &DataRow, snd: &Sounding) -> Vec<String> {
        use metfor::rh;

        let mut results = vec![];

        let t_c = vals.temperature;
        let dp_c = vals.dew_point;
        let pres = vals.pressure;
        let dir = vals.direction;
        let spd = vals.speed;
        let hgt_asl = vals.height;
        let omega = vals.omega;
        let elevation = snd.get_station_info().elevation();

        if t_c.is_some() || dp_c.is_some() || omega.is_some() {
            let mut line = String::with_capacity(128);
            if let Some(t_c) = t_c {
                line.push_str(&format!("{:.0}C", t_c));
            }
            if let Some(dp_c) = dp_c {
                if t_c.is_some() {
                    line.push('/');
                }
                line.push_str(&format!("{:.0}C", dp_c));
            }
            if let (Some(t_c), Some(dp_c)) = (t_c, dp_c) {
                if let Ok(rh) = rh(t_c, dp_c) {
                    line.push_str(&format!(" {:.0}%", 100.0 * rh));
                }
            }
            if let Some(omega) = omega {
                line.push_str(&format!(" {:.1} hPa/s", omega * 10.0));
            }
            results.push(line);
        }

        if pres.is_some() || dir.is_some() || spd.is_some() {
            let mut line = String::with_capacity(128);
            if let Some(pres) = pres {
                line.push_str(&format!("{:.0}hPa", pres));
            }
            if let Some(dir) = dir {
                if pres.is_some() {
                    line.push(' ');
                }
                let dir = (dir / 10.0).round() * 10.0;
                line.push_str(&format!("{:03.0}", dir));
            }
            if let Some(spd) = spd {
                if pres.is_some() && dir.is_none() {
                    line.push(' ');
                }
                line.push_str(&format!("{:02.0}KT", spd));
            }
            results.push(line);
        }

        if let Some(hgt) = hgt_asl {
            results.push(format!("ASL: {:5.0}m ({:5.0}ft)", hgt, 3.28084 * hgt));
        }

        if elevation.is_some() && hgt_asl.is_some() {
            if let (Some(elev), Some(hgt)) = (elevation, hgt_asl) {
                let mut line = String::with_capacity(128);
                line.push_str(&format!(
                    "AGL: {:5.0}m ({:5.0}ft)",
                    hgt - elev,
                    3.28084 * (hgt - elev)
                ));
                results.push(line);
            }
        }

        // Sample the screen coords. Leave these commented out for debugging later possibly.
        // {
        //     use app::PlotContext;
        //     if let Some(pnt) = _ac.skew_t.last_cursor_position_skew_t {
        //         let mut line = String::with_capacity(128);
        //         line.push_str(&format!(
        //             "col: {:3.0} row: {:3.0}",
        //             pnt.col,
        //             pnt.row
        //         ));
        //         results.push(line);
        //         let mut line = String::with_capacity(128);
        //         let pnt = _ac.skew_t.convert_device_to_screen(pnt);
        //         line.push_str(&format!(
        //             "screen x: {:.3} y: {:.3}",
        //             pnt.x,
        //             pnt.y
        //         ));
        //         results.push(line);
        //         let mut line = String::with_capacity(128);
        //         let pnt2 = _ac.skew_t.convert_screen_to_xy(pnt);
        //         line.push_str(&format!(
        //             "x: {:.3} y: {:.3}",
        //             pnt2.x,
        //             pnt2.y
        //         ));
        //         results.push(line);
        //         let mut line = String::with_capacity(128);
        //         let pnt = _ac.skew_t.convert_screen_to_tp(pnt);
        //         line.push_str(&format!(
        //             "t: {:3.0} p: {:3.0}",
        //             pnt.temperature,
        //             pnt.pressure
        //         ));
        //         results.push(line);
        //     }
        // }

        results
    }

    fn draw_active_readout(&self, args: DrawingArgs) {
        if args.ac.config.borrow().show_sample_parcel_profile {
            draw_sample_parcel_profile(args);
        }

        if args.ac.config.borrow().show_active_readout {
            self.draw_active_sample(args);
        }
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
    /// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
    fn scroll_event(&self, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
        const DELTA_SCALE: f64 = 1.05;
        const MIN_ZOOM: f64 = 1.0;
        const MAX_ZOOM: f64 = 10.0;

        let pos = self.convert_device_to_xy(DeviceCoords::from(event.get_position()));
        let dir = event.get_direction();

        let old_zoom = self.get_zoom_factor();
        let mut new_zoom = old_zoom;

        match dir {
            ScrollDirection::Up => {
                new_zoom *= DELTA_SCALE;
            }
            ScrollDirection::Down => {
                new_zoom /= DELTA_SCALE;
            }
            _ => {}
        }

        if new_zoom < MIN_ZOOM {
            new_zoom = MIN_ZOOM;
        } else if new_zoom > MAX_ZOOM {
            new_zoom = MAX_ZOOM;
        }
        self.set_zoom_factor(new_zoom);

        let mut translate = self.get_translate();
        translate = XYCoords {
            x: pos.x - old_zoom / new_zoom * (pos.x - translate.x),
            y: pos.y - old_zoom / new_zoom * (pos.y - translate.y),
        };
        self.set_translate(translate);
        self.bound_view();
        ac.mark_background_dirty();

        ac.update_all_gui();

        Inhibit(true)
    }

    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        event: &EventMotion,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);
                let new_position = DeviceCoords::from(event.get_position());
                self.set_last_cursor_position(Some(new_position));

                let new_position = self.convert_device_to_xy(new_position);
                let delta = (
                    new_position.x - old_position.x,
                    new_position.y - old_position.y,
                );
                let mut translate = self.get_translate();
                translate.x -= delta.0;
                translate.y -= delta.1;
                self.set_translate(translate);
                self.bound_view();
                ac.mark_background_dirty();
                ac.update_all_gui();

                ac.set_sample(None);
            }
        } else if ac.plottable() {
            let position: DeviceCoords = event.get_position().into();

            self.set_last_cursor_position(Some(position));
            let tp_position = self.convert_device_to_tp(position);
            let sample = ::sounding_analysis::linear_interpolate(
                &ac.get_sounding_for_display().unwrap().sounding(), // ac.plottable() call ensures this won't panic
                tp_position.pressure,
            );
            ac.set_sample(sample.ok());
            ac.mark_overlay_dirty();
            ac.update_all_gui();
        }
        Inhibit(false)
    }
}

impl MasterDrawable for SkewTContext {}

/**************************************************************************************************
 *                                   Data Layer Drawing
 **************************************************************************************************/
fn draw_temperature_profiles(args: DrawingArgs) {
    let config = args.ac.config.borrow();

    use self::TemperatureType::{DewPoint, DryBulb, WetBulb};

    if config.show_wet_bulb {
        draw_temperature_profile(WetBulb, args);
    }

    if config.show_dew_point {
        draw_temperature_profile(DewPoint, args);
    }

    if config.show_temperature {
        draw_temperature_profile(DryBulb, args);
    }
}

#[derive(Clone, Copy, Debug)]
enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
}

fn draw_temperature_profile(t_type: TemperatureType, args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    use sounding_base::Profile::{DewPoint, Pressure, Temperature, WetBulb};

    if let Some(sndg) = ac.get_sounding_for_display() {
        let sndg = sndg.sounding();
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

        let profile_data = pres_data
            .iter()
            .zip(temp_data.iter())
            .filter_map(|val_pair| {
                if let (Some(pressure), Some(temperature)) = (*val_pair.0, *val_pair.1) {
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
}

fn draw_wind_profile(args: DrawingArgs) {
    if args.ac.config.borrow().show_wind_profile {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let snd = if let Some(snd) = ac.get_sounding_for_display() {
            snd
        } else {
            return;
        };
        let snd = snd.sounding();

        let barb_config = WindBarbConfig::init(args);
        let barb_data = gather_wind_data(&snd, &barb_config, args);
        let barb_data = filter_wind_data(args, barb_data);

        let rgba = config.wind_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(
            cr.device_to_user_distance(config.wind_barb_line_width, 0.0)
                .0,
        );

        for bdata in &barb_data {
            bdata.draw(cr);
        }
    }
}

fn gather_wind_data(
    snd: &::sounding_base::Sounding,
    barb_config: &WindBarbConfig,
    args: DrawingArgs,
) -> Vec<WindBarbData> {
    use sounding_base::Profile::{Pressure, WindDirection, WindSpeed};

    let dir = snd.get_profile(WindDirection);
    let spd = snd.get_profile(WindSpeed);
    let pres = snd.get_profile(Pressure);

    izip!(pres, dir, spd)
        .filter_map(|tuple| {
            let (p, d, s) = (*tuple.0, *tuple.1, *tuple.2);
            if let (Some(p), Some(d), Some(s)) = (p, d, s) {
                if p > config::MINP {
                    Some((p, d, s))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|tuple| {
            let (p, d, s) = tuple;
            WindBarbData::create(p, d, s, barb_config, args)
        })
        .collect()
}

fn filter_wind_data(args: DrawingArgs, barb_data: Vec<WindBarbData>) -> Vec<WindBarbData> {
    let ac = args.ac;

    // Remove overlapping barbs, or barbs not on the screen
    let mut keepers: Vec<WindBarbData> = vec![];
    let screen_box = ac.skew_t.bounding_box_in_screen_coords();
    let mut last_added_bbox: ScreenRect = ScreenRect {
        lower_left: ScreenCoords {
            x: ::std::f64::MAX,
            y: ::std::f64::MAX,
        },
        upper_right: ScreenCoords {
            x: ::std::f64::MAX,
            y: ::std::f64::MAX,
        },
    };
    for bdata in barb_data {
        let bbox = bdata.bounding_box();
        if !bbox.inside(&screen_box) || bbox.overlaps(&last_added_bbox) {
            continue;
        }
        last_added_bbox = bbox;
        keepers.push(bdata);
    }

    keepers
}

struct WindBarbConfig {
    shaft_length: f64,
    barb_length: f64,
    pennant_width: f64,
    xcoord: f64,
    dot_size: f64,
}

impl WindBarbConfig {
    fn init(args: DrawingArgs) -> Self {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let (shaft_length, barb_length) = cr.device_to_user_distance(
            config.wind_barb_shaft_length,
            -config.wind_barb_barb_length,
        );

        let (dot_size, pennant_width) = cr.device_to_user_distance(
            config.wind_barb_dot_radius,
            -config.wind_barb_pennant_width,
        );
        let padding = cr.device_to_user_distance(config.edge_padding, 0.0).0;

        let screen_bounds = ac.skew_t.bounding_box_in_screen_coords();
        let XYCoords { x: mut xmax, .. } =
            ac.skew_t.convert_screen_to_xy(screen_bounds.upper_right);

        if xmax > 1.0 {
            xmax = 1.0;
        }

        let ScreenCoords { x: xmax, .. } =
            ac.skew_t.convert_xy_to_screen(XYCoords { x: xmax, y: 0.0 });

        let xcoord = xmax - padding - shaft_length;

        WindBarbConfig {
            shaft_length,
            barb_length,
            pennant_width,
            xcoord,
            dot_size,
        }
    }
}

struct WindBarbData {
    center: ScreenCoords,
    shaft_end: ScreenCoords,
    num_pennants: usize,
    pennant_coords: [(ScreenCoords, ScreenCoords, ScreenCoords); 5],
    num_barbs: usize,
    barb_coords: [(ScreenCoords, ScreenCoords); 5],
    point_radius: f64,
}

impl WindBarbData {
    fn create(
        pressure: f64,
        direction: f64,
        speed: f64,
        barb_config: &WindBarbConfig,
        args: DrawingArgs,
    ) -> Self {
        let center = get_wind_barb_center(pressure, barb_config.xcoord, args);

        // Convert angle to traditional XY coordinate plane
        let direction_radians = ::std::f64::consts::FRAC_PI_2 - direction.to_radians();

        let dx = barb_config.shaft_length * direction_radians.cos();
        let dy = barb_config.shaft_length * direction_radians.sin();

        let shaft_end = ScreenCoords {
            x: center.x + dx,
            y: center.y + dy,
        };

        let mut rounded_speed = (speed / 10.0 * 2.0).round() / 2.0 * 10.0;
        let mut num_pennants = 0;
        while rounded_speed >= 50.0 {
            num_pennants += 1;
            rounded_speed -= 50.0;
        }

        let mut num_barbs = 0;
        while rounded_speed >= 10.0 {
            num_barbs += 1;
            rounded_speed -= 10.0;
        }

        let mut pennant_coords = [(
            ScreenCoords::origin(),
            ScreenCoords::origin(),
            ScreenCoords::origin(),
        ); 5];

        for i in 0..num_pennants {
            if i >= pennant_coords.len() {
                break;
            }

            let mut pos = shaft_end;
            pos.x -= (i + 1) as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= (i + 1) as f64 * barb_config.pennant_width * direction_radians.sin();
            let pnt1 = pos;

            pos.x += barb_config.pennant_width * direction_radians.cos();
            pos.y += barb_config.pennant_width * direction_radians.sin();
            let pnt2 = pos;

            let point_angle = direction_radians - ::std::f64::consts::FRAC_PI_2;
            pos.x += barb_config.barb_length * point_angle.cos();
            pos.y += barb_config.barb_length * point_angle.sin();
            let pnt3 = pos;

            pennant_coords[i] = (pnt1, pnt2, pnt3);
        }

        let mut barb_coords = [(ScreenCoords::origin(), ScreenCoords::origin()); 5];

        for i in 0..num_barbs {
            if i >= barb_coords.len() {
                break;
            }

            let mut pos = shaft_end;
            pos.x -= num_pennants as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= num_pennants as f64 * barb_config.pennant_width * direction_radians.sin();

            pos.x -= i as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= i as f64 * barb_config.pennant_width * direction_radians.sin();
            let pnt1 = pos;

            let point_angle = direction_radians - ::std::f64::consts::FRAC_PI_2;
            pos.x += barb_config.barb_length * point_angle.cos();
            pos.y += barb_config.barb_length * point_angle.sin();
            let pnt2 = pos;

            barb_coords[i] = (pnt1, pnt2);
        }

        // Add half barb if needed
        if rounded_speed >= 5.0 && num_barbs < barb_coords.len() {
            let mut pos = shaft_end;
            pos.x -= num_pennants as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= num_pennants as f64 * barb_config.pennant_width * direction_radians.sin();

            pos.x -= num_barbs as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= num_barbs as f64 * barb_config.pennant_width * direction_radians.sin();
            let pnt1 = pos;

            let point_angle = direction_radians - ::std::f64::consts::FRAC_PI_2;
            pos.x += barb_config.barb_length * point_angle.cos() / 2.0;
            pos.y += barb_config.barb_length * point_angle.sin() / 2.0;
            let pnt2 = pos;

            barb_coords[num_barbs] = (pnt1, pnt2);

            num_barbs += 1;
        }

        let point_radius = barb_config.dot_size;

        WindBarbData {
            center,
            shaft_end,
            num_pennants,
            pennant_coords,
            num_barbs,
            barb_coords,
            point_radius,
        }
    }

    fn bounding_box(&self) -> ScreenRect {
        let mut bbox = ScreenRect {
            lower_left: ScreenCoords {
                x: self.center.x - self.point_radius,
                y: self.center.y - self.point_radius,
            },
            upper_right: ScreenCoords {
                x: self.center.x + self.point_radius,
                y: self.center.y + self.point_radius,
            },
        };

        bbox.expand_to_fit(self.shaft_end);
        for i in 0..self.num_pennants {
            if i >= self.pennant_coords.len() {
                break;
            }
            bbox.expand_to_fit(self.pennant_coords[i].2);
        }
        for i in 0..self.num_barbs {
            if i >= self.barb_coords.len() {
                break;
            }
            bbox.expand_to_fit(self.barb_coords[i].1);
        }

        bbox
    }

    fn draw(&self, cr: &Context) {
        // Assume color and line width are already taken care of.
        cr.arc(
            self.center.x,
            self.center.y,
            self.point_radius,
            0.0,
            2.0 * ::std::f64::consts::PI,
        );
        cr.fill();

        cr.move_to(self.center.x, self.center.y);
        cr.line_to(self.shaft_end.x, self.shaft_end.y);
        cr.stroke();

        for (i, &(pnt1, pnt2, pnt3)) in self.pennant_coords.iter().enumerate() {
            if i >= self.num_pennants {
                break;
            }
            cr.move_to(pnt1.x, pnt1.y);
            cr.line_to(pnt2.x, pnt2.y);
            cr.line_to(pnt3.x, pnt3.y);
            cr.close_path();
            cr.fill();
        }

        for (i, &(pnt1, pnt2)) in self.barb_coords.iter().enumerate() {
            if i >= self.num_barbs {
                break;
            }
            cr.move_to(pnt1.x, pnt1.y);
            cr.line_to(pnt2.x, pnt2.y);
            cr.stroke();
        }
    }
}

fn get_wind_barb_center(pressure: f64, xcenter: f64, args: DrawingArgs) -> ScreenCoords {
    let ac = args.ac;

    let ScreenCoords { y: yc, .. } = ac.skew_t.convert_tp_to_screen(TPCoords {
        temperature: 0.0,
        pressure,
    });

    ScreenCoords { x: xcenter, y: yc }
}

/**************************************************************************************************
 *                                Overlay Layer Drawing
 **************************************************************************************************/
fn draw_sample_parcel_profile(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    use sounding_analysis::profile::lift_parcel;
    use sounding_analysis::parcel::Parcel;

    if let Some(sndg) = ac.get_sounding_for_display() {
        let sample_parcel = if let Some(vals) = ac.get_sample() {
            let sample_parcel_opt = vals.pressure.and_then(|p| {
                vals.temperature.and_then(|t| {
                    vals.dew_point.and_then(|dp| {
                        Some(Parcel {
                            temperature: t,
                            pressure: p,
                            dew_point: dp,
                        })
                    })
                })
            });

            match sample_parcel_opt {
                Some(parcel) => parcel,
                None => return,
            }
        } else {
            return;
        };

        let sndg = sndg.sounding();

        let profile = if let Ok(profile) = lift_parcel(sample_parcel, sndg) {
            profile
        } else {
            return;
        };

        let pres_data = profile.pressure;
        let temp_data = profile.parcel_t;

        let line_width = config.temperature_line_width;
        let line_rgba = config.sample_parcel_profile_color;

        let profile_data = pres_data
            .iter()
            .zip(temp_data.iter())
            .filter_map(|val_pair| {
                let (pressure, temperature) = (*val_pair.0, *val_pair.1);

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
}
