use crate::{
    app::{
        config::{self, Rgba},
        sample::{create_sample_plume, create_sample_sounding, Sample},
        AppContext, AppContextPointer, ZoomableDrawingAreas,
    },
    coords::{
        convert_pressure_to_y, convert_y_to_pressure, DeviceCoords, ScreenCoords, ScreenRect,
        TPCoords, XYCoords,
    },
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext},
        utility::{check_overlap_then_add, plot_curve_from_points, plot_dashed_curve_from_points},
        Drawable, DrawingArgs, MasterDrawable, PlotContext, PlotContextExt,
    },
};
use gtk::{
    //Menu,
    gdk::{ButtonEvent, MotionEvent},
    prelude::*,
    DrawingArea,
    Inhibit,
};
use itertools::izip;
use metfor::{Celsius, Feet, Quantity};
use sounding_analysis::{self, Parcel, ParcelProfile};
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
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("skew_t")?;

        let ac = Rc::clone(acp);
        da.set_draw_func(move |_da, cr, _width, _height| {
            ac.skew_t.draw_callback(cr, &ac);
        });

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_scroll_event(move |_da, ev| {
            //ac.mark_background_dirty();
            //ac.skew_t.scroll_event(ev, &ac)
        //});

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_button_press_event(move |_da, ev| ac.skew_t.button_press_event(ev, &ac));

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_button_release_event(move |_da, ev| ac.skew_t.button_release_event(ev));

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_motion_notify_event(move |da, ev| ac.skew_t.mouse_motion_event(da, ev, &ac));

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_enter_notify_event(move |_da, _ev| ac.skew_t.enter_event(&ac));

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_leave_notify_event(move |_da, _ev| ac.skew_t.leave_event(&ac));

        // FIXME
        //let ac = Rc::clone(acp);
        //da.connect_key_press_event(move |_da, ev| SkewTContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_resize(move |da, width, height| {
            // TODO merge below methods into one.
            ac.skew_t.size_allocate_event(da);
            ac.skew_t.resize_event(width, height, &ac);
        });

        // FIXME
        // Self::build_sounding_area_context_menu(acp)?;

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

                let extents = cr.text_extents(&label).unwrap();

                let ScreenCoords { y: screen_y, .. } = self.convert_tp_to_screen(TPCoords {
                    temperature: Celsius(0.0),
                    pressure: p,
                });
                let screen_y = screen_y - extents.height() / 2.0;

                let label_lower_left = ScreenCoords {
                    x: lower_left.x,
                    y: screen_y,
                };
                let label_upper_right = ScreenCoords {
                    x: lower_left.x + extents.width(),
                    y: screen_y + extents.height(),
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

                let extents = cr.text_extents(&label).unwrap();

                let ScreenCoords {
                    x: mut xpos,
                    y: mut ypos,
                } = self.convert_tp_to_screen(TPCoords {
                    temperature: t,
                    pressure: screen_max_p,
                });
                xpos -= extents.width() / 2.0; // Center
                ypos -= extents.height() / 2.0; // Center
                ypos += extents.height(); // Move up off bottom axis.
                xpos += extents.height(); // Move right for 45 degree angle from move up

                let label_lower_left = ScreenCoords { x: xpos, y: ypos };
                let label_upper_right = ScreenCoords {
                    x: xpos + extents.width(),
                    y: ypos + extents.height(),
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
        Self::draw_temperature_profiles(args);
        Self::draw_wind_profile(args);
        Self::draw_data_overlays(args);
        // Drawing the precip icon requires self because it draws relative to the window (like the
        // legend) and not just in data or X-Y coordinates.
        self.draw_precip_icons(args);
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn create_active_readout_text(vals: &Sample, ac: &AppContext) -> Vec<(String, Rgba)> {
        let mut results = vec![];

        let anal = if let Some(anal) = ac.get_sounding_for_display() {
            anal
        } else {
            return results;
        };

        let anal = anal.borrow();
        let config = ac.config.borrow();

        match vals {
            Sample::Sounding {
                ref data,
                ref pcl_anal,
            } => {
                Self::create_active_readout_text_sounding(
                    data,
                    &anal,
                    pcl_anal,
                    &config,
                    &mut results,
                );
            }
            Sample::FirePlume {
                parcel_low,
                plume_anal_low,
                plume_anal_high,
                ..
            } => Self::create_active_readout_text_plume(
                &parcel_low,
                &anal,
                &plume_anal_low,
                &plume_anal_high,
                &config,
                &mut results,
            ),
            Sample::None => {}
        }

        results
    }

    fn draw_active_readout(&self, args: DrawingArgs<'_, '_>) {
        let config = args.ac.config.borrow();

        if config.show_active_readout {
            self.draw_active_sample(args);

            match *args.ac.get_sample() {
                Sample::Sounding { data, ref pcl_anal } => {
                    if let Some(sample_parcel) = Parcel::from_datarow(data) {
                        if config.show_sample_mix_down {
                            Self::draw_sample_mix_down_profile(args, sample_parcel);
                        }
                    }

                    if config.show_sample_parcel_profile {
                        Self::draw_sample_parcel_profile(args, &pcl_anal);
                    }
                }

                Sample::FirePlume {
                    parcel_low,
                    ref profile_low,
                    ref profile_high,
                    ..
                } => {
                    if config.show_sample_parcel_profile {
                        Self::draw_plume_parcel_profiles(
                            args,
                            parcel_low,
                            profile_low,
                            profile_high,
                        );
                    }
                }
                Sample::None => {}
            }
        }
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
    fn enter_event(&self, ac: &AppContextPointer) -> Inhibit {
        ac.set_last_focus(ZoomableDrawingAreas::SkewT);
        Inhibit(false)
    }

    fn button_press_event(&self, event: &ButtonEvent, _ac: &AppContextPointer) -> Inhibit {
        // Left mouse button
        if event.button() == 1 {
            self.set_last_cursor_position(event.position().map(|coords| coords.into()));
            self.set_left_button_pressed(true);
            Inhibit(true)
        } else if event.button() == 3 {
            // FIXME: Get menu working.
            //            if let Ok(menu) = ac.fetch_widget::<Menu>("sounding_context_menu") {
            //                // waiting for version 3.22...
            //                // let ev: &::gdk::Event = evt;
            //                // menu.popup_at_pointer(ev);
            //                menu.popup_easy(3, 0)
            //            }
            Inhibit(false)
        } else {
            Inhibit(false)
        }
    }

    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        event: &MotionEvent,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if self.get_left_button_pressed() {
            if let (Some(last_position), Some(new_position)) =
                (self.get_last_cursor_position(), event.position())
            {
                let old_position = self.convert_device_to_xy(last_position);
                let new_position = DeviceCoords::from(new_position);
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
                // FIXME
                // crate::gui::text_area::update_text_highlight(&ac);

                ac.set_sample(Sample::None);
            }
        } else if ac.plottable() {
            let position: DeviceCoords = event.position().unwrap_or((0.0, 0.0)).into();

            self.set_last_cursor_position(Some(position));
            let tp_position = self.convert_device_to_tp(position);

            let sample = if let Some(max_p) = ac
                .get_sounding_for_display()
                .map(|anal| anal.borrow().max_pressure())
            {
                if tp_position.pressure <= max_p {
                    // This is a sample from some level in the sounding.
                    ac.get_sounding_for_display()
                        .and_then(|anal| {
                            sounding_analysis::linear_interpolate_sounding(
                                anal.borrow().sounding(),
                                tp_position.pressure,
                            )
                            .ok()
                            .map(|data| create_sample_sounding(data, &anal.borrow()))
                        })
                        .unwrap_or(Sample::None)
                } else {
                    // We are below the lowest level in the sounding, so lets generate a plume
                    // parcel!
                    ac.get_sounding_for_display()
                        .and_then(|anal| {
                            let anal = anal.borrow();

                            anal.starting_parcel_for_blow_up_anal()
                                .filter(|pcl| pcl.temperature < tp_position.temperature)
                                .map(|parcel| {
                                    create_sample_plume(parcel, tp_position.temperature, &anal)
                                })
                        })
                        .unwrap_or(Sample::None)
                }
            } else {
                Sample::None
            };

            ac.set_sample(sample);
            ac.mark_overlay_dirty();
            crate::gui::draw_all(&ac);
            // FIXME:
            // crate::gui::text_area::update_text_highlight(&ac);
        }
        Inhibit(false)
    }
}

impl MasterDrawable for SkewTContext {}

mod active_readout;
mod background;
mod data_layer;
// FIXME
//mod menu;
mod wind;

impl SkewTContext {
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
}
