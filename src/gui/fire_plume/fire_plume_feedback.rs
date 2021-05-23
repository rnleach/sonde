use crate::{
    app::{
        config::{self, Rgba},
        sample::{create_sample_plume, Sample},
        AppContext, AppContextPointer, ZoomableDrawingAreas,
    },
    coords::{DeviceCoords, DtPCoords, ScreenCoords, ScreenRect, XYCoords},
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, draw_filled_polygon, plot_curve_from_points},
        Drawable, DrawingArgs, MasterDrawable,
    },
};
use gdk::EventMotion;
use gtk::{prelude::*, DrawingArea};
use itertools::{izip, Itertools};
use metfor::{CelsiusDiff, JpKg, Quantity};
use std::rc::Rc;

pub struct FirePlumeFeedbackContext {
    generic: GenericContext,
}

impl FirePlumeFeedbackContext {
    pub fn new() -> Self {
        FirePlumeFeedbackContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_dtp_to_xy(coords: DtPCoords) -> XYCoords {
        let min_p = config::MIN_FIRE_PLUME_PCT;
        let max_p = config::MAX_FIRE_PLUME_PCT;

        let DtPCoords { dt, percent } = coords;

        let x = super::convert_dt_to_x(dt);
        let y = (percent - min_p) / (max_p - min_p);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_dtp(coords: XYCoords) -> DtPCoords {
        let min_p = config::MIN_FIRE_PLUME_PCT;
        let max_p = config::MAX_FIRE_PLUME_PCT;

        let XYCoords { x, y } = coords;

        let dt = super::convert_x_to_dt(x);
        let percent = y * (max_p - min_p) + min_p;

        DtPCoords { dt, percent }
    }

    pub fn convert_dtp_to_screen(&self, coords: DtPCoords) -> ScreenCoords {
        let xy = FirePlumeFeedbackContext::convert_dtp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_dtp(&self, coords: DeviceCoords) -> DtPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_dtp(xy)
    }
}

impl HasGenericContext for FirePlumeFeedbackContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for FirePlumeFeedbackContext {
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        super::convert_xy_to_screen(self, coords)
    }

    /// Conversion from (x,y) coords to screen coords
    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        super::convert_screen_to_xy(self, coords)
    }
}

impl Drawable for FirePlumeFeedbackContext {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("fire_plume_feedback_area")?;

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.fire_plume_feedback.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.fire_plume_feedback.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| {
            ac.fire_plume_feedback.button_press_event(ev, &ac)
        });

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| {
            ac.fire_plume_feedback.button_release_event(ev)
        });

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| {
            ac.fire_plume_feedback.mouse_motion_event(da, ev, &ac)
        });

        let ac = Rc::clone(acp);
        da.connect_enter_notify_event(move |_da, _ev| ac.fire_plume_feedback.enter_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.fire_plume_feedback.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| {
            FirePlumeFeedbackContext::key_press_event(ev, &ac)
        });

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.fire_plume_feedback.configure_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.fire_plume_feedback.size_allocate_event(da));

        Ok(())
    }

    // Shouldn't override, but need to set font size larger. Failure to DRY.
    fn prepare_to_make_text(&self, args: DrawingArgs<'_, '_>) {
        super::prepare_to_make_text(self, args)
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    fn draw_background_fill(&self, _args: DrawingArgs<'_, '_>) {}

    fn draw_background_lines(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        // Draw iso percents
        for pnts in config::FIRE_PLUME_PCTS_PNTS.iter() {
            let pnts = pnts
                .iter()
                .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
            plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
        }

        super::draw_iso_dts(self, &config, cr);
    }

    fn collect_labels(&self, args: DrawingArgs<'_, '_>) -> Vec<(String, ScreenRect)> {
        let (ac, cr) = (args.ac, args.cr);

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        for &dt in config::FIRE_PLUME_DTS.iter().skip(1) {
            let label = format!("{:.0}\u{00B0}C", dt.unpack());

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                x: mut screen_x, ..
            } = self.convert_dtp_to_screen(DtPCoords { dt, percent: 0.0 });
            screen_x -= extents.width / 2.0;

            let label_lower_left = ScreenCoords {
                x: screen_x,
                y: lower_left.y,
            };
            let label_upper_right = ScreenCoords {
                x: screen_x + extents.width,
                y: lower_left.y + extents.height,
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

        for &percent in config::FIRE_PLUME_PCTS.iter().skip(1) {
            let label = format!("{:.0}%", percent);

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                y: mut screen_y, ..
            } = self.convert_dtp_to_screen(DtPCoords {
                dt: CelsiusDiff(0.0),
                percent,
            });
            screen_y -= extents.height / 2.0;

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

        labels
    }

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        let config = ac.config.borrow();

        let mut lines = Vec::with_capacity(2);

        lines.push((
            "Plume Growth Efficiency (%)".to_owned(),
            config.fire_plume_lmib_color,
        ));
        lines.push((
            "Plume Efficiency (%)".to_owned(),
            config.fire_plume_maxh_color,
        ));

        lines
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs<'_, '_>) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let anal = match ac.get_sounding_for_display() {
            Some(anal) => anal,
            None => return,
        };

        let anal = anal.borrow();

        if let (Some(vals_low), Some(vals_high)) =
            (anal.plume_heating_low(), anal.plume_heating_high())
        {
            let line_width = config.profile_line_width;
            let plume_growth_efficiency_rgba = config.fire_plume_lmib_color;
            let mut plume_growth_efficiency_polygon_rgba = plume_growth_efficiency_rgba;
            plume_growth_efficiency_polygon_rgba.3 /= 2.0;

            let eff_rgba = config.fire_plume_maxh_color;
            let mut eff_polygon_rgba = eff_rgba;
            eff_polygon_rgba.3 /= 2.0;

            let plume_growth_efficiency_vals_low = vals_low
                .plume_growth_efficiencies
                .iter()
                .map(|&(dt, ratio)| DtPCoords {
                    dt,
                    percent: ratio * 100.0,
                })
                .map(|dt_coord| ac.fire_plume_feedback.convert_dtp_to_screen(dt_coord));

            let plume_growth_efficiency_vals_high = vals_high
                .plume_growth_efficiencies
                .iter()
                .map(|&(dt, ratio)| DtPCoords {
                    dt,
                    percent: ratio * 100.0,
                })
                .map(|dt_coord| ac.fire_plume_feedback.convert_dtp_to_screen(dt_coord));

            let plume_growth_efficiency_polygon = plume_growth_efficiency_vals_low
                .clone()
                .chain(plume_growth_efficiency_vals_high.clone().rev());

            draw_filled_polygon(
                cr,
                plume_growth_efficiency_polygon_rgba,
                plume_growth_efficiency_polygon,
            );
            plot_curve_from_points(
                cr,
                line_width,
                plume_growth_efficiency_rgba,
                plume_growth_efficiency_vals_low,
            );
            plot_curve_from_points(
                cr,
                line_width,
                plume_growth_efficiency_rgba,
                plume_growth_efficiency_vals_high,
            );

            let eff_vals_low = izip!(&vals_low.dts, &vals_low.fire_efficiencies)
                .filter_map(|(&dt, &ratio)| ratio.map(|r| (dt, r * 100.0)))
                .map(|(dt, percent)| DtPCoords { dt, percent })
                .map(|dt_coord| ac.fire_plume_feedback.convert_dtp_to_screen(dt_coord));

            let eff_vals_high = izip!(&vals_high.dts, &vals_high.fire_efficiencies)
                .filter_map(|(&dt, &ratio)| ratio.map(|r| (dt, r * 100.0)))
                .map(|(dt, percent)| DtPCoords { dt, percent })
                .map(|dt_coord| ac.fire_plume_feedback.convert_dtp_to_screen(dt_coord));

            let eff_polygon = eff_vals_low.clone().chain(eff_vals_high.clone().rev());

            draw_filled_polygon(cr, eff_polygon_rgba, eff_polygon);
            plot_curve_from_points(cr, line_width, eff_rgba, eff_vals_low);
            plot_curve_from_points(cr, line_width, eff_rgba, eff_vals_high);
        }
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn draw_active_sample(&self, args: DrawingArgs<'_, '_>) {
        if !self.has_data() {
            return;
        }

        let (ac, config) = (args.ac, args.ac.config.borrow());

        let anal = match ac.get_sounding_for_display() {
            Some(anal) => anal,
            None => return,
        };
        let anal = anal.borrow();

        let t0 = match anal.starting_parcel_for_blow_up_anal() {
            Some(pcl) => pcl.temperature,
            None => return,
        };

        let vals = ac.get_sample();
        let pnt_color = config.active_readout_line_rgba;

        if let Sample::FirePlume {
            plume_anal_low,
            plume_anal_high,
            ..
        } = *vals
        {
            let dt = plume_anal_low.parcel.temperature - t0;

            if let Some(max_int_b_low) = plume_anal_low.max_int_buoyancy.into_option() {
                let percent = if max_int_b_low > JpKg(0.0) {
                    max_int_b_low.unpack() / (dt.unpack() * metfor::cp.unpack()) * 100.0
                } else {
                    0.0
                };

                let pct_pnt = DtPCoords { dt, percent };
                let screen_coords_cape = ac.fire_plume_feedback.convert_dtp_to_screen(pct_pnt);
                Self::draw_point(screen_coords_cape, pnt_color, args);
            }

            if let Some(max_int_b_high) = plume_anal_high.max_int_buoyancy.into_option() {
                let percent = if max_int_b_high > JpKg(0.0) {
                    max_int_b_high.unpack() / (dt.unpack() * metfor::cp.unpack()) * 100.0
                } else {
                    0.0
                };

                let pct_pnt = DtPCoords { dt, percent };
                let screen_coords_cape = ac.fire_plume_feedback.convert_dtp_to_screen(pct_pnt);
                Self::draw_point(screen_coords_cape, pnt_color, args);
            }

            if let (Some(vals_low), Some(vals_high)) =
                (anal.plume_heating_low(), anal.plume_heating_high())
            {
                if let Some(percent) = vals_low
                    .plume_growth_efficiencies
                    .iter()
                    .tuple_windows::<(_, _)>()
                    .find(|(&(dt0, _r0), &(dt1, _r1))| dt0 <= dt && dt <= dt1)
                    .map(|(&(dt0, r0), &(dt1, r1))| ((dt0.unpack(), r0), (dt1.unpack(), r1)))
                    .map(|((dt0, r0), (dt1, r1))| {
                        (r1 - r0) / (dt1 - dt0) * (dt.unpack() - dt0) + r0
                    })
                    .map(|ratio| 100.0 * ratio)
                {
                    let pct_pnt = DtPCoords { dt, percent };
                    let screen_coords_cape = ac.fire_plume_feedback.convert_dtp_to_screen(pct_pnt);
                    Self::draw_point(screen_coords_cape, pnt_color, args);
                }

                if let Some(percent) = vals_high
                    .plume_growth_efficiencies
                    .iter()
                    .tuple_windows::<(_, _)>()
                    .find(|(&(dt0, _r0), &(dt1, _r1))| dt0 <= dt && dt <= dt1)
                    .map(|(&(dt0, r0), &(dt1, r1))| ((dt0.unpack(), r0), (dt1.unpack(), r1)))
                    .map(|((dt0, r0), (dt1, r1))| {
                        (r1 - r0) / (dt1 - dt0) * (dt.unpack() - dt0) + r0
                    })
                    .map(|ratio| 100.0 * ratio)
                {
                    let pct_pnt = DtPCoords { dt, percent };
                    let screen_coords_cape = ac.fire_plume_feedback.convert_dtp_to_screen(pct_pnt);
                    Self::draw_point(screen_coords_cape, pnt_color, args);
                }
            }
        }
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
                self.mark_background_dirty();

                crate::gui::draw_all(ac);
                ac.set_sample(Sample::None);
            }
        } else if ac.plottable() {
            let position: DeviceCoords = event.get_position().into();
            self.set_last_cursor_position(Some(position));

            let sample = ac
                .get_sounding_for_display()
                .and_then(|anal| {
                    let DtPCoords { dt, .. } = self.convert_device_to_dtp(position);
                    let pcl = anal.borrow().starting_parcel_for_blow_up_anal();
                    pcl.map(|pcl| (anal, pcl, dt))
                })
                .map(|(anal, pcl, dt)| {
                    create_sample_plume(pcl, pcl.temperature + dt, &anal.borrow())
                })
                .unwrap_or(Sample::None);

            ac.set_sample(sample);
            ac.mark_overlay_dirty();
            crate::gui::draw_all(&ac);
        }

        Inhibit(false)
    }

    fn enter_event(&self, ac: &AppContextPointer) -> Inhibit {
        ac.set_last_focus(ZoomableDrawingAreas::FirePlumeEnergy);
        Inhibit(false)
    }
}

impl MasterDrawable for FirePlumeFeedbackContext {}