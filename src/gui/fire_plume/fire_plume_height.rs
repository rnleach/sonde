use crate::{
    app::{
        config::{self, Rgba},
        sample::{create_sample_plume, Sample},
        AppContext, AppContextPointer, ZoomableDrawingAreas,
    },
    coords::{DeviceCoords, DtHCoords, ScreenCoords, ScreenRect, XYCoords},
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
use metfor::{CelsiusDiff, Meters, Quantity};
use std::rc::Rc;

pub struct FirePlumeContext {
    generic: GenericContext,
}

impl FirePlumeContext {
    pub fn new() -> Self {
        FirePlumeContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_dth_to_xy(coords: DtHCoords) -> XYCoords {
        let min_h = config::MIN_FIRE_PLUME_HEIGHT.unpack();
        let max_h = config::MAX_FIRE_PLUME_HEIGHT.unpack();

        let DtHCoords {
            dt,
            height: Meters(h),
        } = coords;

        let x = super::convert_dt_to_x(dt);
        let y = (h - min_h) / (max_h - min_h);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_dth(coords: XYCoords) -> DtHCoords {
        let min_h = config::MIN_FIRE_PLUME_HEIGHT.unpack();
        let max_h = config::MAX_FIRE_PLUME_HEIGHT.unpack();

        let XYCoords { x, y } = coords;

        let dt = super::convert_x_to_dt(x);
        let height = Meters(y * (max_h - min_h) + min_h);

        DtHCoords { dt, height }
    }

    pub fn convert_dth_to_screen(&self, coords: DtHCoords) -> ScreenCoords {
        let xy = FirePlumeContext::convert_dth_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_dth(&self, coords: DeviceCoords) -> DtHCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_dth(xy)
    }
}

impl HasGenericContext for FirePlumeContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for FirePlumeContext {
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        super::convert_xy_to_screen(self, coords)
    }

    /// Conversion from (x,y) coords to screen coords
    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        super::convert_screen_to_xy(self, coords)
    }
}

impl Drawable for FirePlumeContext {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("fire_plume_height_area")?;

        // Set up the drawing function.
        let ac = Rc::clone(acp);
        da.set_draw_func(move |_da, cr, _width, _height| {
            ac.fire_plume.draw_callback(cr, &ac);
        });

        // Set up the scroll (or zoom in/out) callbacks.
        let ac = Rc::clone(acp);
        let scroll_control = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        scroll_control.connect_scroll(move |_scroll_control, _dx, dy| {
            ac.mark_background_dirty();
            ac.fire_plume.scroll_event(dy, &ac);

            Propagation::Stop
        });
        da.add_controller(scroll_control);

        // Set up the button clicks.
        let left_mouse_button = GestureClick::builder().build();

        let ac = Rc::clone(acp);
        left_mouse_button.connect_pressed(move |_mouse_button, _n_pressed, x, y| {
            ac.fire_plume.left_button_press_event((x, y), &ac);
        });

        let ac = Rc::clone(acp);
        left_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.fire_plume.left_button_release_event((x, y), &ac);
        });

        da.add_controller(left_mouse_button);

        let right_mouse_button = GestureClick::builder().button(3).build();
        let ac = Rc::clone(acp);
        right_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.fire_plume.right_button_release_event((x, y), &ac);
        });
        da.add_controller(right_mouse_button);

        // Set up the mouse motion events
        let mouse_motion = EventControllerMotion::new();

        let ac = Rc::clone(acp);
        mouse_motion.connect_motion(move |mouse_motion, x, y| {
            ac.fire_plume.mouse_motion_event(mouse_motion, (x, y), &ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_enter(move |_mouse_motion, _x, _y| {
            ac.fire_plume.enter_event(&ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_leave(move |_mouse_motion| {
            ac.fire_plume.leave_event(&ac);
        });

        da.add_controller(mouse_motion);

        // Set up the key presses.
        let key_press = EventControllerKey::new();
        let ac = Rc::clone(acp);
        key_press.connect_key_pressed(move |_key_press, key, _code, _key_modifier| {
            FirePlumeContext::key_press_event(key, &ac)
        });
        da.add_controller(key_press);

        let ac = Rc::clone(acp);
        da.connect_resize(move |da, width, height| {
            // TODO merge below methods into one.
            ac.fire_plume.size_allocate_event(da);
            ac.fire_plume.resize_event(width, height, &ac);
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

        // Draw iso heights
        for pnts in config::FIRE_PLUME_HEIGHT_PNTS.iter() {
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
            } = self.convert_dth_to_screen(DtHCoords {
                dt,
                height: Meters(0.0),
            });
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

        for &h in config::FIRE_PLUME_HEIGHTS.iter().skip(1) {
            let label = format!("{:.0}", h.unpack() / 1_000.0);

            let extents = cr.text_extents(&label).unwrap();

            let ScreenCoords {
                y: mut screen_y, ..
            } = self.convert_dth_to_screen(DtHCoords {
                dt: CelsiusDiff(0.0),
                height: h,
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

        vec![
            (
                "Level of Max Int. Buoyancy (km)".to_owned(),
                config.fire_plume_lmib_color,
            ),
            (
                "Lifting Condensation Level (km)".to_owned(),
                config.fire_plume_lcl_color,
            ),
            (
                "Max Plume Height (km)".to_owned(),
                config.fire_plume_maxh_color,
            ),
        ]
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
            let lmib_rgba = config.fire_plume_lmib_color;
            let mut lmib_polygon_color = lmib_rgba;
            lmib_polygon_color.3 /= 2.0;

            let max_hgt_rgba = config.fire_plume_maxh_color;
            let mut max_hgt_polygon_color = max_hgt_rgba;
            max_hgt_polygon_color.3 /= 2.0;

            let lcl_rgba = config.fire_plume_lcl_color;
            let mut lcl_polygon_color = lcl_rgba;
            lcl_polygon_color.3 /= 2.0;

            let els_low = izip!(&vals_low.dts, &vals_low.el_heights)
                .filter_map(|(&dt, height)| height.map(|h| (dt, h)))
                .map(|(dt, height)| DtHCoords { dt, height })
                .map(|dt_coord| ac.fire_plume.convert_dth_to_screen(dt_coord));

            let els_high = izip!(&vals_high.dts, &vals_high.el_heights)
                .filter_map(|(&dt, height)| height.map(|h| (dt, h)))
                .map(|(dt, height)| DtHCoords { dt, height })
                .map(|dt_coord| ac.fire_plume.convert_dth_to_screen(dt_coord));

            let polygon = els_low.clone().chain(els_high.clone().rev());
            draw_filled_polygon(cr, lmib_polygon_color, polygon);

            plot_curve_from_points(cr, line_width, lmib_rgba, els_low);
            plot_curve_from_points(cr, line_width, lmib_rgba, els_high);

            let lcls_low = izip!(&vals_low.dts, &vals_low.lcl_heights)
                .filter_map(|(&dt, height)| height.map(|h| (dt, h)))
                .map(|(dt, height)| DtHCoords { dt, height })
                .map(|dt_coord| ac.fire_plume.convert_dth_to_screen(dt_coord));

            let lcls_high = izip!(&vals_high.dts, &vals_high.lcl_heights)
                .filter_map(|(&dt, height)| height.map(|h| (dt, h)))
                .map(|(dt, height)| DtHCoords { dt, height })
                .map(|dt_coord| ac.fire_plume.convert_dth_to_screen(dt_coord));

            let polygon = lcls_low.clone().chain(lcls_high.clone().rev());
            draw_filled_polygon(cr, lcl_polygon_color, polygon);

            plot_curve_from_points(cr, line_width, lcl_rgba, lcls_low);
            plot_curve_from_points(cr, line_width, lcl_rgba, lcls_high);

            let maxhs_low = izip!(&vals_low.dts, &vals_low.max_heights)
                .filter_map(|(&dt, height)| height.map(|h| (dt, h)))
                .map(|(dt, height)| DtHCoords { dt, height })
                .map(|dt_coord| ac.fire_plume.convert_dth_to_screen(dt_coord));

            let maxhs_high = izip!(&vals_high.dts, &vals_high.max_heights)
                .filter_map(|(&dt, height)| height.map(|h| (dt, h)))
                .map(|(dt, height)| DtHCoords { dt, height })
                .map(|dt_coord| ac.fire_plume.convert_dth_to_screen(dt_coord));

            let polygon = maxhs_low.clone().chain(maxhs_high.clone().rev());
            draw_filled_polygon(cr, max_hgt_polygon_color, polygon);

            plot_curve_from_points(cr, line_width, max_hgt_rgba, maxhs_low);
            plot_curve_from_points(cr, line_width, max_hgt_rgba, maxhs_high);
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

        let vals = ac.get_sample();

        if let Sample::FirePlume {
            plume_anal_low,
            plume_anal_high,
            ..
        } = *vals
        {
            let t0 = match ac.get_sounding_for_display() {
                Some(anal) => match anal.borrow().starting_parcel_for_blow_up_anal() {
                    Some(pcl) => pcl.temperature,
                    None => return,
                },
                None => return,
            };

            let pnt_color = config.active_readout_line_rgba;

            let dt_low = plume_anal_low.parcel.temperature - t0;
            let dt_high = plume_anal_high.parcel.temperature - t0;

            if let Some(lmib_low) = plume_anal_low.el_height.into_option() {
                let lmib_pnt = DtHCoords {
                    dt: dt_low,
                    height: lmib_low,
                };
                let screen_coords_el = ac.fire_plume.convert_dth_to_screen(lmib_pnt);
                Self::draw_point(screen_coords_el, pnt_color, args);
            }

            if let Some(lmib_high) = plume_anal_high.el_height.into_option() {
                let lmib_pnt = DtHCoords {
                    dt: dt_high,
                    height: lmib_high,
                };
                let screen_coords_el = ac.fire_plume.convert_dth_to_screen(lmib_pnt);
                Self::draw_point(screen_coords_el, pnt_color, args);
            }

            if let Some(maxh_low) = plume_anal_low.max_height.into_option() {
                let maxh_pnt = DtHCoords {
                    dt: dt_low,
                    height: maxh_low,
                };
                let screen_coords_maxh = ac.fire_plume.convert_dth_to_screen(maxh_pnt);
                Self::draw_point(screen_coords_maxh, pnt_color, args);
            }

            if let Some(maxh_high) = plume_anal_high.max_height.into_option() {
                let maxh_pnt = DtHCoords {
                    dt: dt_high,
                    height: maxh_high,
                };
                let screen_coords_maxh = ac.fire_plume.convert_dth_to_screen(maxh_pnt);
                Self::draw_point(screen_coords_maxh, pnt_color, args);
            }

            if let Some(lcl_low) = plume_anal_low.lcl_height.into_option() {
                let lcl_pnt = DtHCoords {
                    dt: dt_low,
                    height: lcl_low,
                };
                let screen_coords_lcl = ac.fire_plume.convert_dth_to_screen(lcl_pnt);
                Self::draw_point(screen_coords_lcl, pnt_color, args);
            }

            if let Some(lcl_high) = plume_anal_high.lcl_height.into_option() {
                let lcl_pnt = DtHCoords {
                    dt: dt_high,
                    height: lcl_high,
                };
                let screen_coords_lcl = ac.fire_plume.convert_dth_to_screen(lcl_pnt);
                Self::draw_point(screen_coords_lcl, pnt_color, args);
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
        let da: DrawingArea = controller.widget().unwrap().downcast().unwrap();
        da.grab_focus();

        let position = DeviceCoords::from(new_position);

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
                    let DtHCoords { dt, .. } = self.convert_device_to_dth(position);
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
        ac.set_last_focus(ZoomableDrawingAreas::FirePlume);
    }
}

impl MasterDrawable for FirePlumeContext {}
