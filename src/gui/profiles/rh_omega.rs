use crate::{
    app::{
        config,
        config::Rgba,
        sample::{create_sample_sounding, Sample},
        AppContext, AppContextPointer,
    },
    coords::{
        convert_pressure_to_y, convert_y_to_pressure, DeviceCoords, Rect, ScreenCoords, ScreenRect,
        WPCoords, XYCoords,
    },
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, draw_horizontal_bars, plot_curve_from_points},
        Drawable, DrawingArgs, SlaveProfileDrawable,
    },
};
use gdk::EventMotion;
use gtk::{prelude::*, DrawingArea};
use itertools::izip;
use metfor::{PaPS, Quantity};
use sounding_analysis::{relative_humidity, relative_humidity_ice};
use std::{cell::Cell, rc::Rc};

#[derive(Debug)]
pub struct RHOmegaContext {
    x_zoom: Cell<f64>,
    generic: GenericContext,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            x_zoom: Cell::new(1.0),
            generic: GenericContext::new(),
        }
    }

    pub fn convert_wp_to_xy(coords: WPCoords) -> XYCoords {
        let y = convert_pressure_to_y(coords.p);

        // The + sign below looks weird, but is correct.
        let x = (coords.w + config::MAX_ABS_W) / (config::MAX_ABS_W * 2.0);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_wp(coords: XYCoords) -> WPCoords {
        let p = convert_y_to_pressure(coords.y);
        let w = (config::MAX_ABS_W * 2.0) * coords.x - config::MAX_ABS_W;

        WPCoords { w, p }
    }

    pub fn convert_screen_to_wp(&self, coords: ScreenCoords) -> WPCoords {
        let xy = self.convert_screen_to_xy(coords);
        RHOmegaContext::convert_xy_to_wp(xy)
    }

    pub fn convert_wp_to_screen(&self, coords: WPCoords) -> ScreenCoords {
        let xy = RHOmegaContext::convert_wp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_wp(&self, coords: DeviceCoords) -> WPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_wp(xy)
    }
}

impl HasGenericContext for RHOmegaContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for RHOmegaContext {
    fn bound_view(&self) {
        let device_rect = self.get_device_rect();

        let bounds = DeviceCoords {
            col: device_rect.width,
            row: device_rect.height,
        };
        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy(device_rect.upper_left);
        let height = upper_left.y - lower_right.y;

        let mut translate = self.get_translate();
        if height < 1.0 {
            if translate.y < 0.0 {
                translate.y = 0.0;
            }
            let max_y = 1.0 - height;
            if translate.y > max_y {
                translate.y = max_y;
            }
        } else {
            translate.y = -(height - 1.0) / 2.0;
        }
        self.set_translate(translate);
    }

    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        // Apply translation first
        let x = coords.x - self.get_translate().x;
        let y = coords.y - self.get_translate().y;

        // Apply scaling
        let x = x * self.x_zoom.get();
        let y = y * self.get_zoom_factor();

        ScreenCoords { x, y }
    }

    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x / self.x_zoom.get();
        let y = coords.y / self.get_zoom_factor();

        // Unapply translation
        let x = x + self.get_translate().x;
        let y = y + self.get_translate().y;

        XYCoords { x, y }
    }
}

impl Drawable for RHOmegaContext {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("rh_omega_area")?;

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.rh_omega.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.rh_omega.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.rh_omega.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| RHOmegaContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.rh_omega.configure_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.rh_omega.size_allocate_event(da));

        Ok(())
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/

    fn draw_background_fill(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        if config.show_background_bands {
            let rgba = config.background_band_rgba;
            cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

            let mut omegas = config::ISO_OMEGA.iter();
            let mut draw = true;
            let mut prev = omegas.next();
            while let Some(prev_val) = prev {
                let curr = omegas.next();
                if let Some(curr_val) = curr {
                    if draw {
                        let ll = WPCoords {
                            w: *prev_val,
                            p: config::MAXP,
                        };
                        let ur = WPCoords {
                            w: *curr_val,
                            p: config::MINP,
                        };
                        let ll = self.convert_wp_to_screen(ll);
                        let ur = self.convert_wp_to_screen(ur);
                        let ScreenCoords { x: xmin, y: ymin } = ll;
                        let ScreenCoords { x: xmax, y: ymax } = ur;
                        cr.rectangle(xmin, ymin, xmax - xmin, ymax - ymin);
                        cr.fill().unwrap();
                        draw = false;
                    } else {
                        draw = true;
                    }
                }
                prev = curr;
            }
        }
    }

    fn draw_background_lines(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        // Draw isobars
        if config.show_isobars {
            for pnts in config::ISOBAR_PNTS.iter() {
                let pnts = pnts
                    .iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
                plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
            }
        }

        // Draw w-lines
        for v_line in config::ISO_OMEGA_PNTS.iter() {
            plot_curve_from_points(
                cr,
                config.background_line_width,
                config.isobar_rgba,
                v_line
                    .iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords)),
            );
        }

        // Make a thicker zero line
        plot_curve_from_points(
            cr,
            config.background_line_width * 2.6,
            config.isobar_rgba,
            ([
                WPCoords {
                    w: PaPS(0.0),
                    p: config::MAXP,
                },
                WPCoords {
                    w: PaPS(0.0),
                    p: config::MINP,
                },
            ])
            .iter()
            .map(|wp_coords| self.convert_wp_to_screen(*wp_coords)),
        );
    }

    fn collect_labels(&self, args: DrawingArgs<'_, '_>) -> Vec<(String, ScreenRect)> {
        let (ac, cr) = (args.ac, args.cr);

        let mut labels = vec![];

        let screen_edges = ac.rh_omega.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        let WPCoords {
            p: screen_max_p, ..
        } = ac.rh_omega.convert_screen_to_wp(lower_left);

        for &w in [PaPS(0.0)].iter().chain(config::ISO_OMEGA.iter()) {
            let label = format!("{:.0}", w.unpack());

            let extents = cr.text_extents(&label).unwrap();

            let ScreenCoords {
                x: mut xpos,
                y: mut ypos,
            } = ac
                .rh_omega
                .convert_wp_to_screen(WPCoords { w, p: screen_max_p });
            xpos -= extents.width() / 2.0; // Center
            ypos -= extents.height() / 2.0; // Center
            ypos += extents.height(); // Move up off bottom axis.

            let ScreenRect {
                lower_left: ScreenCoords { x: xmin, .. },
                upper_right: ScreenCoords { x: xmax, .. },
            } = screen_edges;

            if xpos < xmin || xpos + extents.width() > xmax {
                continue;
            }

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

        labels
    }

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        let mut result = vec![];

        let config = ac.config.borrow();

        if config.show_rh {
            result.push(("RH (water)".to_owned(), config.rh_rgba));
        }

        if config.show_rh_ice {
            result.push(("RH (ice)".to_owned(), config.rh_ice_rgba));
        }

        if config.show_omega {
            result.push(("PVV".to_owned(), config.omega_rgba));
        }

        result
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs<'_, '_>) {
        self.draw_hail_growth_zone(args);
        self.draw_dendritic_snow_growth_zone(args);
        self.draw_warm_layer_aloft(args);

        self.draw_wet_bulb_zero_levels(args);
        self.draw_freezing_levels(args);

        let rh_ice_drawn = draw_rh_ice_profile(args);
        let rh_drawn = draw_rh_profile(args);
        let omega_drawn = draw_omega_profile(args);
        let has_data = rh_ice_drawn || rh_drawn || omega_drawn;
        self.set_has_data(has_data);
        if !has_data {
            self.draw_no_data(args);
        }
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn create_active_readout_text(vals: &Sample, ac: &AppContext) -> Vec<(String, Rgba)> {
        use metfor::rh;

        let config = ac.config.borrow();

        let mut results = vec![];

        let vals = match vals {
            Sample::Sounding { data, .. } => data,
            Sample::FirePlume { .. } | Sample::None => return results,
        };

        let t_c = vals.temperature;
        let dp_c = vals.dew_point;
        let omega = vals.pvv;

        if (t_c.is_some() && dp_c.is_some()) || omega.is_some() {
            if config.show_rh {
                if let (Some(t_c), Some(dp_c)) = (t_c.into_option(), dp_c.into_option()) {
                    if let Some(rh) = rh(t_c, dp_c) {
                        results.push((format!("{:.0}% (water)\n", 100.0 * rh), config.rh_rgba));
                    }
                }
            }

            if config.show_rh_ice {
                if let (Some(t_c), Some(dp_c)) = (t_c.into_option(), dp_c.into_option()) {
                    let vp_water = metfor::vapor_pressure_water(dp_c);
                    let sat_vp_ice = metfor::vapor_pressure_ice(t_c);
                    if let Some(rh) = vp_water.and_then(|vpw| sat_vp_ice.map(|vpi| vpw / vpi)) {
                        results.push((format!("{:.0}% (ice)\n", 100.0 * rh), config.rh_ice_rgba));
                    }
                }
            }

            if config.show_omega {
                if let Some(omega) = omega.into_option() {
                    results.push((format!("{:.1} Pa/s\n", omega.unpack()), config.omega_rgba));
                }
            }
        }

        results
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        event: &EventMotion,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if ac.plottable() {
            let position: DeviceCoords = event.position().into();

            self.set_last_cursor_position(Some(position));
            let wp_position = self.convert_device_to_wp(position);

            let sample = ac
                .get_sounding_for_display()
                .and_then(|anal| {
                    sounding_analysis::linear_interpolate_sounding(
                        anal.borrow().sounding(),
                        wp_position.p,
                    )
                    .ok()
                    .map(|data| create_sample_sounding(data, &anal.borrow()))
                })
                .unwrap_or(Sample::None);
            ac.set_sample(sample);
            ac.mark_overlay_dirty();
            crate::gui::draw_all(&ac);
            crate::gui::text_area::update_text_highlight(&ac);
        }
        Inhibit(false)
    }
}

impl SlaveProfileDrawable for RHOmegaContext {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64 {
        acp.skew_t.get_zoom_factor()
    }

    fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.set_translate(translate);
    }
}

fn draw_rh_profile(args: DrawingArgs<'_, '_>) -> bool {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if !config.show_rh {
        return false;
    }

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        let sndg = anal.sounding();

        ac.rh_omega.set_has_data(true);

        let pres_data = sndg.pressure_profile();
        let rh_data = relative_humidity(sndg);

        let bb = ac.rh_omega.get_plot_area();
        let x0 = bb.lower_left.x;
        let width = bb.width();

        let profile = izip!(pres_data, rh_data.iter())
            // Filter out levels with missing pressure and map missing RH to 0%
            .filter_map(|(p, rh)| p.map(|p| (p, rh.unwrap_or(0.0))))
            // Only take up to the highest plottable pressu
            .take_while(|(p, _)| *p > config::MINP)
            // Map into ScreenCoords for plotting
            .map(|(p, rh)| {
                let ScreenCoords { y, .. } = ac
                    .rh_omega
                    .convert_wp_to_screen(WPCoords { w: PaPS(0.0), p });
                let x = x0 + width * rh;
                ScreenCoords { x, y }
            });

        let line_width = config.bar_graph_line_width;
        let rgba = config.rh_rgba;

        draw_horizontal_bars(cr, line_width, rgba, profile);

        true
    } else {
        false
    }
}

fn draw_rh_ice_profile(args: DrawingArgs<'_, '_>) -> bool {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if !config.show_rh_ice {
        return false;
    }

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        let sndg = anal.sounding();

        ac.rh_omega.set_has_data(true);

        let pres_data = sndg.pressure_profile();
        let rh_data = relative_humidity_ice(sndg);

        let bb = ac.rh_omega.get_plot_area();
        let x0 = bb.lower_left.x;
        let width = bb.width();

        let profile = izip!(pres_data, rh_data.iter())
            // Filter out levels with missing pressure and map missing RH to 0%
            .filter_map(|(p, rh)| p.map(|p| (p, rh.unwrap_or(0.0))))
            // Only take up to the highest plottable pressu
            .take_while(|(p, _)| *p > config::MINP)
            // Map into ScreenCoords for plotting
            .map(|(p, rh)| {
                let ScreenCoords { y, .. } = ac
                    .rh_omega
                    .convert_wp_to_screen(WPCoords { w: PaPS(0.0), p });
                let x = x0 + width * rh;
                ScreenCoords { x, y }
            });

        let line_width = config.bar_graph_line_width;
        let rgba = config.rh_ice_rgba;

        draw_horizontal_bars(cr, line_width, rgba, profile);

        true
    } else {
        false
    }
}

fn draw_omega_profile(args: DrawingArgs<'_, '_>) -> bool {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if !config.show_omega {
        return false;
    }

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        let sndg = anal.sounding();

        let pres_data = sndg.pressure_profile();
        let omega_data = sndg.pvv_profile();
        let line_width = config.profile_line_width;
        let line_rgba = config.omega_rgba;

        let profile_data = izip!(pres_data, omega_data).filter_map(|(p, w)| {
            if let (Some(p), Some(w)) = (p.into(), w.into()) {
                if p > config::MINP {
                    let wp_coords = WPCoords { w, p };
                    Some(ac.rh_omega.convert_wp_to_screen(wp_coords))
                } else {
                    None
                }
            } else {
                None
            }
        });

        plot_curve_from_points(cr, line_width, line_rgba, profile_data);
    } else {
        return false;
    }

    true
}
