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
use gtk::{
    glib::Propagation, prelude::*, DrawingArea, EventControllerKey, EventControllerMotion,
    EventControllerScroll, EventControllerScrollFlags, GestureClick,
};
use itertools::izip;
use metfor::{CelsiusDiff, JpKg, Quantity};
use std::rc::Rc;

pub struct FirePlumeEnergyContext {
    generic: GenericContext,
}

impl FirePlumeEnergyContext {
    pub fn new() -> Self {
        FirePlumeEnergyContext {
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
        let xy = FirePlumeEnergyContext::convert_dtp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_dtp(&self, coords: DeviceCoords) -> DtPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_dtp(xy)
    }
}

impl HasGenericContext for FirePlumeEnergyContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for FirePlumeEnergyContext {
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        super::convert_xy_to_screen(self, coords)
    }

    /// Conversion from (x,y) coords to screen coords
    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        super::convert_screen_to_xy(self, coords)
    }
}

impl Drawable for FirePlumeEnergyContext {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("fire_plume_energy_area")?;
        //
        // Set up the drawing function.
        let ac = Rc::clone(acp);
        da.set_draw_func(move |_da, cr, _width, _height| {
            ac.fire_plume_energy.draw_callback(cr, &ac);
        });

        // Set up the scroll (or zoom in/out) callbacks.
        let ac = Rc::clone(acp);
        let scroll_control = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        scroll_control.connect_scroll(move |_scroll_control, _dx, dy| {
            ac.mark_background_dirty();
            ac.fire_plume_energy.scroll_event(dy, &ac);

            Propagation::Stop
        });
        da.add_controller(scroll_control);

        // Set up the button clicks.
        let left_mouse_button = GestureClick::builder().build();

        let ac = Rc::clone(acp);
        left_mouse_button.connect_pressed(move |_mouse_button, _n_pressed, x, y| {
            ac.fire_plume_energy.left_button_press_event((x, y), &ac);
        });

        let ac = Rc::clone(acp);
        left_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.fire_plume_energy.left_button_release_event((x, y), &ac);
        });

        da.add_controller(left_mouse_button);

        let right_mouse_button = GestureClick::builder().button(3).build();
        let ac = Rc::clone(acp);
        right_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.fire_plume_energy.right_button_release_event((x, y), &ac);
        });
        da.add_controller(right_mouse_button);

        // Set up the mouse motion events
        let mouse_motion = EventControllerMotion::new();

        let ac = Rc::clone(acp);
        mouse_motion.connect_motion(move |mouse_motion, x, y| {
            ac.fire_plume_energy
                .mouse_motion_event(mouse_motion, (x, y), &ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_enter(move |_mouse_motion, _x, _y| {
            ac.fire_plume_energy.enter_event(&ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_leave(move |_mouse_motion| {
            ac.fire_plume_energy.leave_event(&ac);
        });

        da.add_controller(mouse_motion);

        // Set up the key presses.
        let key_press = EventControllerKey::new();
        let ac = Rc::clone(acp);
        key_press.connect_key_pressed(move |_key_press, key, _code, _key_modifier| {
            FirePlumeEnergyContext::key_press_event(key, &ac)
        });
        da.add_controller(key_press);

        let ac = Rc::clone(acp);
        da.connect_resize(move |da, width, height| {
            // TODO merge below methods into one.
            ac.fire_plume_energy.size_allocate_event(da);
            ac.fire_plume_energy.resize_event(width, height, &ac);
        });

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

        // Draw iso capes
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

            let extents = cr.text_extents(&label).unwrap();

            let ScreenCoords {
                x: mut screen_x, ..
            } = self.convert_dtp_to_screen(DtPCoords { dt, percent: 0.0 });
            screen_x -= extents.width() / 2.0;

            let label_lower_left = ScreenCoords {
                x: screen_x,
                y: lower_left.y,
            };
            let label_upper_right = ScreenCoords {
                x: screen_x + extents.width(),
                y: lower_left.y + extents.height(),
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

            let extents = cr.text_extents(&label).unwrap();

            let ScreenCoords {
                y: mut screen_y, ..
            } = self.convert_dtp_to_screen(DtPCoords {
                dt: CelsiusDiff(0.0),
                percent,
            });
            screen_y -= extents.height() / 2.0;

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

        labels
    }

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        let config = ac.config.borrow();

        let mut lines = Vec::with_capacity(2);

        lines.push((
            "Percent Wet Integrated Bouyancy".to_owned(),
            config.fire_plume_pct_wet_cape_color,
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
            let pct_wet_rgba = config.fire_plume_pct_wet_cape_color;
            let mut pct_wet_polygon_color = pct_wet_rgba;
            pct_wet_polygon_color.3 /= 2.0;

            let vals_low = izip!(&vals_low.dts, &vals_low.wet_ratio)
                .filter_map(|(&dt, &ratio)| ratio.map(|r| (dt, r * 100.0)))
                .map(|(dt, percent)| DtPCoords { dt, percent })
                .map(|dt_coord| ac.fire_plume_energy.convert_dtp_to_screen(dt_coord));

            let vals_high = izip!(&vals_high.dts, &vals_high.wet_ratio)
                .filter_map(|(&dt, &ratio)| ratio.map(|r| (dt, r * 100.0)))
                .map(|(dt, percent)| DtPCoords { dt, percent })
                .map(|dt_coord| ac.fire_plume_energy.convert_dtp_to_screen(dt_coord));

            let polygon = vals_low.clone().chain(vals_high.clone().rev());

            draw_filled_polygon(cr, pct_wet_polygon_color, polygon);
            plot_curve_from_points(cr, line_width, pct_wet_rgba, vals_low);
            plot_curve_from_points(cr, line_width, pct_wet_rgba, vals_high);
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
        let t0 = match ac.get_sounding_for_display() {
            Some(anal) => match anal.borrow().starting_parcel_for_blow_up_anal() {
                Some(pcl) => pcl.temperature,
                None => return,
            },
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

            if let (Some(max_int_b_low), Some(max_dry_int_b_low)) = (
                plume_anal_low.max_int_buoyancy.into_option(),
                plume_anal_low.max_dry_int_buoyancy.into_option(),
            ) {
                let percent = if max_int_b_low > JpKg(0.0) {
                    (max_int_b_low - max_dry_int_b_low) / max_int_b_low * 100.0
                } else {
                    0.0
                };

                let pct_pnt = DtPCoords { dt, percent };
                let screen_coords_cape = ac.fire_plume_energy.convert_dtp_to_screen(pct_pnt);
                Self::draw_point(screen_coords_cape, pnt_color, args);
            }

            if let (Some(max_int_b_high), Some(max_dry_int_b_high)) = (
                plume_anal_high.max_int_buoyancy.into_option(),
                plume_anal_high.max_dry_int_buoyancy.into_option(),
            ) {
                let percent = if max_int_b_high > JpKg(0.0) {
                    (max_int_b_high - max_dry_int_b_high) / max_int_b_high * 100.0
                } else {
                    0.0
                };

                let pct_pnt = DtPCoords { dt, percent };
                let screen_coords_cape = ac.fire_plume_energy.convert_dtp_to_screen(pct_pnt);
                Self::draw_point(screen_coords_cape, pnt_color, args);
            }
        }
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
    fn mouse_motion_event(
        &self,
        controller: &EventControllerMotion,
        new_position: (f64, f64),
        ac: &AppContextPointer,
    ) {
        let da: DrawingArea = controller.widget().downcast().unwrap();
        da.grab_focus();

        let position: DeviceCoords = DeviceCoords::from(new_position);

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);

                let new_position = self.convert_device_to_xy(position);
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
            crate::gui::draw_all(ac);
        }
        self.set_last_cursor_position(Some(position));
    }

    fn enter_event(&self, ac: &AppContextPointer) {
        ac.set_last_focus(ZoomableDrawingAreas::FirePlumeEnergy);
    }
}

impl MasterDrawable for FirePlumeEnergyContext {}
