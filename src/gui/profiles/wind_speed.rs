use crate::{
    app::{
        config,
        config::Rgba,
        sample::{create_sample_sounding, Sample},
        AppContext, AppContextPointer,
    },
    coords::{
        convert_pressure_to_y, convert_y_to_pressure, DeviceCoords, SPCoords, ScreenCoords,
        ScreenRect, XYCoords,
    },
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, plot_curve_from_points, DrawingArgs},
        Drawable, SlaveProfileDrawable,
    },
};
use gtk::{prelude::*, DrawingArea, EventControllerKey, EventControllerMotion, GestureClick};
use itertools::izip;
use metfor::{Knots, Quantity, WindSpdDir};
use std::rc::Rc;

pub struct WindSpeedContext {
    generic: GenericContext,
}

impl WindSpeedContext {
    pub fn new() -> Self {
        WindSpeedContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_sp_to_xy(coords: SPCoords) -> XYCoords {
        let y = convert_pressure_to_y(coords.press);

        let mut x = coords.spd.unpack();
        // Avoid infinity
        if x <= 0.0 {
            x = 0.00001;
        }
        x = (f64::log10(x) - f64::log10(1.0))
            / (f64::log10(config::MAX_PROFILE_SPEED.unpack()) - f64::log10(1.0));

        XYCoords { x, y }
    }

    pub fn convert_xy_to_sp(coords: XYCoords) -> SPCoords {
        let press = convert_y_to_pressure(coords.y);

        let spd = Knots(10.0f64.powf(
            coords.x * (f64::log10(config::MAX_PROFILE_SPEED.unpack()) - f64::log10(1.0))
                + f64::log10(1.0),
        ));

        SPCoords { spd, press }
    }

    pub fn convert_sp_to_screen(&self, coords: SPCoords) -> ScreenCoords {
        let xy = Self::convert_sp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_screen_to_sp(&self, coords: ScreenCoords) -> SPCoords {
        let xy = self.convert_screen_to_xy(coords);
        Self::convert_xy_to_sp(xy)
    }

    pub fn convert_device_to_sp(&self, coords: DeviceCoords) -> SPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_sp(xy)
    }
}

impl HasGenericContext for WindSpeedContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for WindSpeedContext {
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
        let x = coords.x;
        let y = coords.y - self.get_translate().y;

        // Apply scaling
        let x = x;
        let y = y * self.get_zoom_factor();

        ScreenCoords { x, y }
    }

    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x;
        let y = coords.y / self.get_zoom_factor();

        // Unapply translation
        let x = x;
        let y = y + self.get_translate().y;

        XYCoords { x, y }
    }
}

impl Drawable for WindSpeedContext {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("wind_speed_area")?;

        // Set up the drawing function.
        let ac = Rc::clone(acp);
        da.set_draw_func(move |_da, cr, _width, _height| {
            ac.wind_speed.draw_callback(cr, &ac);
        });

        // Set up the button clicks.
        let left_mouse_button = GestureClick::builder().build();

        let ac = Rc::clone(acp);
        left_mouse_button.connect_pressed(move |_mouse_button, _n_pressed, x, y| {
            ac.wind_speed.left_button_press_event((x, y), &ac);
        });

        let ac = Rc::clone(acp);
        left_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.wind_speed.left_button_release_event((x, y), &ac);
        });

        da.add_controller(left_mouse_button);

        let right_mouse_button = GestureClick::builder().button(3).build();
        let ac = Rc::clone(acp);
        right_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.wind_speed.right_button_release_event((x, y), &ac);
        });
        da.add_controller(right_mouse_button);

        // Set up the mouse motion events
        let mouse_motion = EventControllerMotion::new();

        let ac = Rc::clone(acp);
        mouse_motion.connect_motion(move |mouse_motion, x, y| {
            ac.wind_speed.mouse_motion_event(mouse_motion, (x, y), &ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_enter(move |_mouse_motion, _x, _y| {
            ac.wind_speed.enter_event(&ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_leave(move |_mouse_motion| {
            ac.wind_speed.leave_event(&ac);
        });

        da.add_controller(mouse_motion);

        // Set up the key presses.
        let key_press = EventControllerKey::new();
        let ac = Rc::clone(acp);
        key_press.connect_key_pressed(move |_key_press, key, _code, _key_modifier| {
            WindSpeedContext::key_press_event(key, &ac)
        });
        da.add_controller(key_press);

        let ac = Rc::clone(acp);
        da.connect_resize(move |da, width, height| {
            // TODO merge below methods into one.
            ac.wind_speed.size_allocate_event(da);
            ac.wind_speed.resize_event(width, height, &ac);
        });

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

            let mut lines = config::PROFILE_SPEED_PNTS.iter();
            let mut draw = true;
            let mut prev = lines.next();
            while let Some(prev_val) = prev {
                let curr = lines.next();
                if let Some(curr_val) = curr {
                    if draw {
                        let ll = self.convert_xy_to_screen(prev_val[0]);
                        let ur = self.convert_xy_to_screen(curr_val[1]);
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

        for line in config::PROFILE_SPEED_PNTS.iter() {
            let pnts = line
                .iter()
                .map(|xy_coord| self.convert_xy_to_screen(*xy_coord));
            plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
        }
    }

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        vec![("Wind speed".to_owned(), ac.config.borrow().wind_rgba)]
    }

    fn collect_labels(&self, args: DrawingArgs<'_, '_>) -> Vec<(String, ScreenRect)> {
        let (ac, cr) = (args.ac, args.cr);

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        let SPCoords {
            press: screen_max_p,
            ..
        } = self.convert_screen_to_sp(lower_left);

        for spd in &config::PROFILE_SPEEDS {
            let label = format!("{:.0}", spd.unpack());

            let extents = cr.text_extents(&label).unwrap();

            let ScreenCoords {
                x: mut xpos,
                y: mut ypos,
            } = self.convert_sp_to_screen(SPCoords {
                spd: *spd,
                press: screen_max_p,
            });
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

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs<'_, '_>) {
        self.draw_hail_growth_zone(args);
        self.draw_dendritic_snow_growth_zone(args);
        self.draw_warm_layer_aloft(args);

        draw_wind_speed_profile(args);
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn create_active_readout_text(vals: &Sample, ac: &AppContext) -> Vec<(String, Rgba)> {
        let mut results = vec![];

        if let Sample::Sounding { data, .. } = vals {
            if let Some(WindSpdDir { speed, .. }) = data.wind.into_option() {
                let line = format!("{:.0}KT\n", speed.unpack());
                results.push((line, ac.config.borrow().wind_rgba));
            }
        }

        results
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

        let position = DeviceCoords::from(new_position);

        if ac.plottable() && self.has_data() {
            let sp_position = self.convert_device_to_sp(position);

            let sample = ac
                .get_sounding_for_display()
                .and_then(|anal| {
                    sounding_analysis::linear_interpolate_sounding(
                        anal.borrow().sounding(),
                        sp_position.press,
                    )
                    .ok()
                    .map(|data| create_sample_sounding(data, &anal.borrow()))
                })
                .unwrap_or(Sample::None);
            ac.set_sample(sample);
            ac.mark_overlay_dirty();
            crate::gui::draw_all(ac);
            crate::gui::text_area::update_text_highlight(ac);
        }

        self.set_last_cursor_position(Some(position));
    }
}

impl SlaveProfileDrawable for WindSpeedContext {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64 {
        acp.skew_t.get_zoom_factor()
    }

    fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.set_translate(translate);
    }
}

fn draw_wind_speed_profile(args: DrawingArgs<'_, '_>) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        let sndg = anal.sounding();

        let pres_data = sndg.pressure_profile();
        let spd_data = sndg.wind_profile();

        if spd_data.iter().any(|opt| opt.is_some()) {
            ac.wind_speed.set_has_data(true);
        } else {
            ac.wind_speed.set_has_data(false);
        }

        let profile = izip!(pres_data, spd_data)
            .filter_map(|(p, spd)| {
                if let (Some(p), Some(WindSpdDir { speed: s, .. })) =
                    (p.into_option(), spd.into_option())
                {
                    Some((p, s))
                } else {
                    None
                }
            })
            .filter_map(|pair| {
                let (press, spd) = pair;
                if press > config::MINP {
                    Some(ac.wind_speed.convert_sp_to_screen(SPCoords { spd, press }))
                } else {
                    None
                }
            });

        plot_curve_from_points(cr, config.profile_line_width, config.wind_rgba, profile);
    } else {
        ac.wind_speed.set_has_data(false);
    }

    if !ac.wind_speed.has_data() {
        ac.wind_speed.draw_no_data(args);
    }
}
