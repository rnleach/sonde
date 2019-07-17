use crate::{
    app::{
        config::{self, ParcelType, Rgba},
        AppContext, AppContextPointer,
    },
    coords::{
        convert_pressure_to_y, convert_y_to_pressure, DeviceCoords, Rect, ScreenCoords, ScreenRect,
        TPCoords, XYCoords,
    },
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext},
        utility::{
            check_overlap_then_add, draw_filled_polygon, plot_curve_from_points,
            plot_dashed_curve_from_points,
        },
        Drawable, DrawingArgs, MasterDrawable, PlotContext, PlotContextExt,
    },
};
use cairo::Context;
use gdk::{EventButton, EventMotion, EventScroll, ScrollDirection};
use gtk::{
    prelude::*, CheckMenuItem, DrawingArea, Menu, MenuItem, RadioMenuItem, SeparatorMenuItem,
};
use itertools::izip;
use log::warn;
use metfor::{
    Celsius, CelsiusDiff, Fahrenheit, Feet, HectoPascal, JpKg, Knots, Quantity, WindSpdDir,
};
use sounding_analysis::{self, Analysis, Parcel, ParcelAnalysis, ParcelProfile};
use sounding_base::DataRow;
use std::rc::Rc;

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

        let t = config::MINT + (config::MAXT - config::MINT) * x;
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
    fn draw_clear_background(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.background_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        const MINT: Celsius = Celsius(-160.0);
        const MAXT: Celsius = Celsius(100.0);
        self.draw_temperature_band(MINT, MAXT, args);
    }

    fn draw_temperature_banding(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.background_band_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        let mut start_line = -160i32;
        while start_line < 100 {
            let t1 = Celsius(f64::from(start_line));
            let t2 = t1 + CelsiusDiff(10.0);

            self.draw_temperature_band(t1, t2, args);

            start_line += 20;
        }
    }

    fn draw_hail_growth_zone(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.hail_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        self.draw_temperature_band(Celsius(-30.0), Celsius(-10.0), args);
    }

    fn draw_dendtritic_growth_zone(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.dendritic_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        self.draw_temperature_band(Celsius(-18.0), Celsius(-12.0), args);
    }

    fn draw_temperature_band(&self, cold_t: Celsius, warm_t: Celsius, args: DrawingArgs<'_, '_>) {
        let cr = args.cr;

        // Assume color has already been set up for us.

        const MAXP: HectoPascal = config::MAXP;
        const MINP: HectoPascal = config::MINP;

        let mut coords = [
            (warm_t.unpack(), MAXP.unpack()),
            (warm_t.unpack(), MINP.unpack()),
            (cold_t.unpack(), MINP.unpack()),
            (cold_t.unpack(), MAXP.unpack()),
        ];

        // Convert points to screen coords
        for coord in &mut coords {
            let screen_coords = self.convert_tp_to_screen(TPCoords {
                temperature: Celsius(coord.0),
                pressure: HectoPascal(coord.1),
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
    /********************** *************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("skew_t")?;

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.skew_t.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.skew_t.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.skew_t.button_press_event(ev, &ac));

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

        build_sounding_area_context_menu(acp)?;

        Ok(())
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    fn draw_background_fill(&self, args: DrawingArgs<'_, '_>) {
        let config = args.ac.config.borrow();

        self.draw_clear_background(args);

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

    fn draw_background_lines(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        // Draws background lines from the bottom up.

        // Draw isentrops
        if config.show_isentrops {
            for pnts in config::ISENTROP_PNTS.iter() {
                let pnts = pnts
                    .iter()
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
                let pnts = pnts
                    .iter()
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
                let pnts = pnts
                    .iter()
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
                let pnts = pnts
                    .iter()
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
                let pnts = pnts
                    .iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));

                plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
            }
        }

        // Draw the freezing line
        if config.show_freezing_line {
            let pnts = &[
                TPCoords {
                    temperature: Celsius(0.0),
                    pressure: config::MAXP,
                },
                TPCoords {
                    temperature: Celsius(0.0),
                    pressure: config::MINP,
                },
            ];
            let pnts = pnts
                .iter()
                .map(|tp_coords| self.convert_tp_to_screen(*tp_coords));
            plot_curve_from_points(
                cr,
                config.freezing_line_width,
                config.freezing_line_color,
                pnts,
            );
        }
    }

    fn collect_labels(&self, args: DrawingArgs<'_, '_>) -> Vec<(String, ScreenRect)> {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        if config.show_isobars {
            for &p in &config::ISOBARS {
                let label = format!("{:.0}", p.unpack());

                let extents = cr.text_extents(&label);

                let ScreenCoords { y: screen_y, .. } = self.convert_tp_to_screen(TPCoords {
                    temperature: Celsius(0.0),
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
                let label = format!("{:.0}", t.unpack());

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

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        use chrono::Weekday::*;

        let color = ac.config.borrow().label_rgba;

        let mut result = vec![];

        if let Some(anal) = ac.get_sounding_for_display() {
            if let Some(src_desc) = anal.borrow().sounding().source_description() {
                result.push((src_desc.to_owned(), color));
            }
        }

        if let Some(anal) = ac.get_sounding_for_display() {
            let anal = anal.borrow();
            let snd = anal.sounding();
            // Build the valid time part
            if let Some(vt) = snd.valid_time() {
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

                if let Some(lt) = snd.lead_time().into_option() {
                    temp_string.push_str(&format!(" F{:03}", lt));
                }

                result.push((temp_string, color));
            }

            // Build location part.
            let coords = snd.station_info().location();
            let elevation = snd.station_info().elevation();
            if coords.is_some() || elevation.is_some() {
                let mut location = "".to_owned();

                if let Some((lat, lon)) = coords {
                    location.push_str(&format!("{:.2}, {:.2}", lat, lon));
                    if elevation.is_some() {
                        location.push_str(", ");
                    }
                }
                if let Some(el) = elevation.into_option() {
                    location.push_str(&format!(
                        "{:.0}m ({:.0}ft)",
                        el.unpack(),
                        Feet::from(el).unpack()
                    ));
                }

                result.push((location, color));
            }
        }

        result
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs<'_, '_>) {
        draw_temperature_profiles(args);
        draw_wind_profile(args);
        draw_data_overlays(args);
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn create_active_readout_text(vals: &DataRow, ac: &AppContext) -> Vec<(String, Rgba)> {
        use metfor::rh;

        let mut results = vec![];

        let anal = if let Some(anal) = ac.get_sounding_for_display() {
            anal
        } else {
            return results;
        };

        let anal = anal.borrow();

        let config = ac.config.borrow();

        let default_color = config.label_rgba;

        let t_c = vals.temperature;
        let dp_c = vals.dew_point;
        let pres = vals.pressure;
        let wind = vals.wind;
        let hgt_asl = vals.height;
        let omega = vals.pvv;
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
            if let Some(pcl) = Parcel::from_datarow(*vals) {
                if let Ok(pcl_anal) = sounding_analysis::lift_parcel(pcl, anal.sounding()) {
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
            };
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

    fn draw_active_readout(&self, args: DrawingArgs<'_, '_>) {
        let config = args.ac.config.borrow();

        if config.show_active_readout {
            self.draw_active_sample(args);

            if let Some(sample_parcel) = get_sample_parcel(args) {
                if config.show_sample_parcel_profile {
                    draw_sample_parcel_profile(args, sample_parcel);
                }

                if config.show_sample_mix_down {
                    draw_sample_mix_down_profile(args, sample_parcel);
                }
            }
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

        crate::gui::draw_all(&ac);
        crate::gui::text_area::update_text_highlight(&ac);

        Inhibit(true)
    }

    fn button_press_event(&self, event: &EventButton, ac: &AppContextPointer) -> Inhibit {
        // Left mouse button
        if event.get_button() == 1 {
            self.set_last_cursor_position(Some(event.get_position().into()));
            self.set_left_button_pressed(true);
            Inhibit(true)
        } else if event.get_button() == 3 {
            if let Ok(menu) = ac.fetch_widget::<Menu>("sounding_context_menu") {
                // waiting for version 3.22...
                // let ev: &::gdk::Event = evt;
                // menu.popup_at_pointer(ev);
                menu.popup_easy(3, 0)
            }
            Inhibit(false)
        } else {
            Inhibit(false)
        }
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
                crate::gui::draw_all(&ac);
                crate::gui::text_area::update_text_highlight(&ac);

                ac.set_sample(None);
            }
        } else if ac.plottable() {
            let position: DeviceCoords = event.get_position().into();

            self.set_last_cursor_position(Some(position));
            let tp_position = self.convert_device_to_tp(position);

            let sample = ac.get_sounding_for_display().and_then(|anal| {
                sounding_analysis::linear_interpolate_sounding(
                    anal.borrow().sounding(),
                    tp_position.pressure,
                )
                .ok()
            });

            ac.set_sample(sample);
            ac.mark_overlay_dirty();
            crate::gui::draw_all(&ac);
            crate::gui::text_area::update_text_highlight(&ac);
        }
        Inhibit(false)
    }
}

impl MasterDrawable for SkewTContext {}

/**************************************************************************************************
 *                                   DrawingArea set up
 **************************************************************************************************/
fn build_sounding_area_context_menu(acp: &AppContextPointer) -> Result<(), SondeError> {
    let menu: Menu = acp.fetch_widget("sounding_context_menu")?;

    build_active_readout_section_of_context_menu(&menu, acp);
    menu.append(&SeparatorMenuItem::new());
    build_overlays_section_of_context_menu(&menu, acp);
    menu.append(&SeparatorMenuItem::new());
    build_profiles_section_of_context_menu(&menu, acp);

    menu.show_all();

    Ok(())
}

macro_rules! make_heading {
    ($menu:ident, $label:expr) => {
        let heading = MenuItem::new_with_label($label);
        heading.set_sensitive(false);
        $menu.append(&heading);
    };
}

macro_rules! make_check_item {
    ($menu:ident, $label:expr, $acp:ident, $check_val:ident) => {
        let check_menu_item = CheckMenuItem::new_with_label($label);
        check_menu_item.set_active($acp.config.borrow().$check_val);

        let ac = Rc::clone($acp);
        check_menu_item.connect_toggled(move |button| {
            ac.config.borrow_mut().$check_val = button.get_active();
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
        });

        $menu.append(&check_menu_item);
    };
}

fn build_active_readout_section_of_context_menu(menu: &Menu, acp: &AppContextPointer) {
    make_heading!(menu, "Active readout");
    make_check_item!(menu, "Show active readout", acp, show_active_readout);
    make_check_item!(menu, "Draw sample parcel", acp, show_sample_parcel_profile);
    make_check_item!(menu, "Draw sample mix down", acp, show_sample_mix_down);
}

fn build_overlays_section_of_context_menu(menu: &Menu, acp: &AppContextPointer) {
    use crate::app::config::ParcelType::*;

    make_heading!(menu, "Parcel Type");

    let sfc = RadioMenuItem::new_with_label("Surface");
    let mxd = RadioMenuItem::new_with_label_from_widget(&sfc, Some("Mixed Layer"));
    let mu = RadioMenuItem::new_with_label_from_widget(&sfc, Some("Most Unstable"));
    let con = RadioMenuItem::new_with_label_from_widget(&sfc, Some("Convective"));

    let p_type = acp.config.borrow().parcel_type;
    match p_type {
        Surface => sfc.set_active(true),
        MixedLayer => mxd.set_active(true),
        MostUnstable => mu.set_active(true),
        Convective => con.set_active(true),
    }

    fn handle_toggle(button: &RadioMenuItem, parcel_type: ParcelType, ac: &AppContextPointer) {
        if button.get_active() {
            ac.config.borrow_mut().parcel_type = parcel_type;
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
        }
    }

    let ac = Rc::clone(acp);
    sfc.connect_toggled(move |button| {
        handle_toggle(button, Surface, &ac);
    });

    let ac = Rc::clone(acp);
    mxd.connect_toggled(move |button| {
        handle_toggle(button, MixedLayer, &ac);
    });

    let ac = Rc::clone(acp);
    mu.connect_toggled(move |button| {
        handle_toggle(button, MostUnstable, &ac);
    });

    let ac = Rc::clone(acp);
    con.connect_toggled(move |button| {
        handle_toggle(button, Convective, &ac);
    });

    menu.append(&sfc);
    menu.append(&mxd);
    menu.append(&mu);
    menu.append(&con);

    menu.append(&SeparatorMenuItem::new());

    make_heading!(menu, "Parcel Options");
    make_check_item!(menu, "Show profile", acp, show_parcel_profile);
    make_check_item!(menu, "Fill CAPE/CIN", acp, fill_parcel_areas);
    make_check_item!(menu, "Show downburst", acp, show_downburst);
    make_check_item!(menu, "Fill downburst", acp, fill_dcape_area);
    make_check_item!(menu, "Show inflow layer", acp, show_inflow_layer);

    menu.append(&SeparatorMenuItem::new());

    make_heading!(menu, "Inversions");
    make_check_item!(menu, "Show inv. mix-down", acp, show_inversion_mix_down);
}

fn build_profiles_section_of_context_menu(menu: &Menu, acp: &AppContextPointer) {
    make_heading!(menu, "Profiles");
    make_check_item!(menu, "Temperature", acp, show_temperature);
    make_check_item!(menu, "Wet bulb", acp, show_wet_bulb);
    make_check_item!(menu, "Dew point", acp, show_dew_point);
    make_check_item!(menu, "Wind", acp, show_wind_profile);
}

/**************************************************************************************************
 *                                   Data Layer Drawing
 **************************************************************************************************/
fn draw_temperature_profiles(args: DrawingArgs<'_, '_>) {
    let config = args.ac.config.borrow();

    if config.show_wet_bulb {
        draw_temperature_profile(TemperatureType::WetBulb, args);
    }

    if config.show_dew_point {
        draw_temperature_profile(TemperatureType::DewPoint, args);
    }

    if config.show_temperature {
        draw_temperature_profile(TemperatureType::DryBulb, args);
    }
}

#[derive(Clone, Copy, Debug)]
enum TemperatureType {
    DryBulb,
    WetBulb,
    DewPoint,
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

fn draw_wind_profile(args: DrawingArgs<'_, '_>) {
    if args.ac.config.borrow().show_wind_profile {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let anal = if let Some(anal) = ac.get_sounding_for_display() {
            anal
        } else {
            return;
        };

        let anal = anal.borrow();
        let snd = anal.sounding();

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

fn draw_data_overlays(args: DrawingArgs<'_, '_>) {
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
        }
        .and_then(|p_analysis| {
            let color = config.parcel_rgba;
            let p_profile = p_analysis.profile();

            draw_parcel_profile(args, &p_profile, color);

            if config.fill_parcel_areas {
                draw_cape_cin_fill(args, &p_analysis);
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
        draw_downburst(args, &anal);
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
                draw_parcel_profile(args, &parcel_profile, color);

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
                let yb = get_wind_barb_center(bottom_p, xcoord, args);
                let yt = get_wind_barb_center(top_p, xcoord, args);

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

fn draw_cape_cin_fill(args: DrawingArgs<'_, '_>, parcel_analysis: &ParcelAnalysis) {
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
    draw_parcel_profile(args, parcel_profile, color);

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

fn gather_wind_data(
    snd: &::sounding_base::Sounding,
    barb_config: &WindBarbConfig,
    args: DrawingArgs<'_, '_>,
) -> Vec<WindBarbData> {
    let wind = snd.wind_profile();
    let pres = snd.pressure_profile();

    izip!(pres, wind)
        .filter_map(|tuple| {
            let (p, w) = (*tuple.0, *tuple.1);
            if let (Some(p), Some(w)) = (p.into(), w.into()) {
                if p > config::MINP {
                    Some((p, w))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|tuple| {
            let (p, w) = tuple;
            WindBarbData::create(p, w, barb_config, args)
        })
        .collect()
}

fn filter_wind_data(args: DrawingArgs<'_, '_>, barb_data: Vec<WindBarbData>) -> Vec<WindBarbData> {
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
    fn init(args: DrawingArgs<'_, '_>) -> Self {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let (shaft_length, barb_length) = cr
            .device_to_user_distance(config.wind_barb_shaft_length, -config.wind_barb_barb_length);

        let (dot_size, pennant_width) = cr
            .device_to_user_distance(config.wind_barb_dot_radius, -config.wind_barb_pennant_width);
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
        pressure: HectoPascal,
        wind: WindSpdDir<Knots>,
        barb_config: &WindBarbConfig,
        args: DrawingArgs<'_, '_>,
    ) -> Self {
        let center = get_wind_barb_center(pressure, barb_config.xcoord, args);

        let WindSpdDir {
            speed: Knots(speed),
            direction,
        } = wind;

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

fn get_wind_barb_center(
    pressure: HectoPascal,
    xcenter: f64,
    args: DrawingArgs<'_, '_>,
) -> ScreenCoords {
    let ac = args.ac;

    let ScreenCoords { y: yc, .. } = ac.skew_t.convert_tp_to_screen(TPCoords {
        temperature: Celsius(0.0),
        pressure,
    });

    ScreenCoords { x: xcenter, y: yc }
}

/**************************************************************************************************
 *                              Active Readout Layer Drawing
 **************************************************************************************************/
fn draw_sample_parcel_profile(args: DrawingArgs<'_, '_>, sample_parcel: Parcel) {
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
    let parcel_analysis = sounding_analysis::lift_parcel(sample_parcel, sndg);
    let profile = if let Ok(ref parcel_analysis) = parcel_analysis {
        parcel_analysis.profile()
    } else {
        return;
    };

    let color = config.sample_parcel_profile_color;

    draw_parcel_profile(args, &profile, color);
}

fn draw_sample_mix_down_profile(args: DrawingArgs<'_, '_>, sample_parcel: Parcel) {
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

    draw_parcel_profile(args, &profile, color);

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

fn draw_parcel_profile(args: DrawingArgs<'_, '_>, profile: &ParcelProfile, line_rgba: Rgba) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    let pres_data = &profile.pressure;
    let temp_data = &profile.parcel_t;

    let line_width = config.temperature_line_width;

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

    plot_dashed_curve_from_points(cr, line_width, line_rgba, profile_data);
}

fn get_sample_parcel(args: DrawingArgs<'_, '_>) -> Option<Parcel> {
    args.ac.get_sample().and_then(Parcel::from_datarow)
}
