use std::cell::Cell;
use std::rc::Rc;

use gdk::{EventMask, EventMotion};
use gtk::prelude::*;
use gtk::DrawingArea;

use sounding_base::DataRow;

use app::{config, AppContext, AppContextPointer};
use coords::{convert_pressure_to_y, convert_y_to_pressure, DeviceCoords, Rect, ScreenCoords,
             ScreenRect, WPCoords, XYCoords};
use gui::{Drawable, DrawingArgs, SlaveProfileDrawable};
use gui::plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt};
use gui::utility::{check_overlap_then_add, plot_curve_from_points};

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
        let x = (coords.w + config::MAX_ABS_W) / (2.0 * config::MAX_ABS_W);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_wp(coords: XYCoords) -> WPCoords {
        let p = convert_y_to_pressure(coords.y);
        let w = coords.x * (2.0 * config::MAX_ABS_W) - config::MAX_ABS_W;

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
    fn zoom_to_envelope(&self) {
        let xy_envelope = &self.get_xy_envelope();
        self.set_translate(xy_envelope.lower_left);

        let width = xy_envelope.width();
        let width_scale = 1.0 / width;

        self.x_zoom.set(width_scale);
        self.bound_view();
    }

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
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.rh_omega.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.rh_omega.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.rh_omega.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| RHOmegaContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.rh_omega.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.rh_omega.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);

        da.set_no_show_all(true);
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/

    fn draw_background_fill(&self, args: DrawingArgs) {
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
                        cr.fill();
                        draw = false;
                    } else {
                        draw = true;
                    }
                }
                prev = curr;
            }
        }

        self.draw_hail_growth_zone(args);
        self.draw_dendtritic_snow_growth_zone(args);
        self.draw_warm_layer_aloft(args);
    }

    fn draw_background_lines(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        // Draw isobars
        if config.show_isobars {
            for pnts in config::ISOBAR_PNTS.iter() {
                let pnts = pnts.iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
                plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
            }
        }

        // Draw w-lines
        if config.show_iso_omega_lines {
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
                        w: 0.0,
                        p: config::MAXP,
                    },
                    WPCoords {
                        w: 0.0,
                        p: config::MINP,
                    },
                ]).iter()
                    .map(|wp_coords| self.convert_wp_to_screen(*wp_coords)),
            );
        }
    }

    fn collect_labels(&self, args: DrawingArgs) -> Vec<(String, ScreenRect)> {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let mut labels = vec![];

        let screen_edges = ac.rh_omega.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        if config.show_iso_omega_lines {
            let WPCoords {
                p: screen_max_p, ..
            } = ac.rh_omega.convert_screen_to_wp(lower_left);

            for &w in [0.0].iter().chain(config::ISO_OMEGA.iter()) {
                let label = format!("{:.0}", w * 10.0);

                let extents = cr.text_extents(&label);

                let ScreenCoords {
                    x: mut xpos,
                    y: mut ypos,
                } = ac.rh_omega
                    .convert_wp_to_screen(WPCoords { w, p: screen_max_p });
                xpos -= extents.width / 2.0; // Center
                ypos -= extents.height / 2.0; // Center
                ypos += extents.height; // Move up off bottom axis.

                let ScreenRect {
                    lower_left: ScreenCoords { x: xmin, .. },
                    upper_right: ScreenCoords { x: xmax, .. },
                } = screen_edges;

                if xpos < xmin || xpos + extents.width > xmax {
                    continue;
                }

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

    fn build_legend_strings(_ac: &AppContext) -> Vec<String> {
        vec!["RH & PVV".to_owned()]
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs) {
        let rh_drawn = draw_rh_profile(args);
        let omega_drawn = draw_omega_profile(args);
        let has_data = rh_drawn || omega_drawn;
        self.set_has_data(has_data);
        if !has_data {
            self.draw_no_data(args);
        }
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn create_active_readout_text(vals: &DataRow, _ac: &AppContext) -> Vec<String> {
        use metfor::rh;

        let mut results = vec![];

        let t_c = vals.temperature;
        let dp_c = vals.dew_point;
        let omega = vals.omega;

        if t_c.is_some() || dp_c.is_some() || omega.is_some() {
            let mut line = String::with_capacity(128);

            if let (Some(t_c), Some(dp_c)) = (t_c, dp_c) {
                if let Ok(rh) = rh(t_c, dp_c) {
                    line.push_str(&format!("{:.0}%", 100.0 * rh));
                }
            }
            if t_c.is_some() && dp_c.is_some() && omega.is_some() {
                line.push(' ');
            }
            if let Some(omega) = omega {
                line.push_str(&format!("{:.1} hPa/s", omega * 10.0));
            }
            results.push(line);
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
            let position: DeviceCoords = event.get_position().into();

            self.set_last_cursor_position(Some(position));
            let wp_position = self.convert_device_to_wp(position);
            let sample = ::sounding_analysis::linear_interpolate_sounding(
                // will not panic due to ac.plottable
                &ac.get_sounding_for_display().expect(file!()).sounding(),
                wp_position.p,
            );
            ac.set_sample(sample.ok());
            ac.mark_overlay_dirty();
            ac.update_all_gui();
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

fn draw_rh_profile(args: DrawingArgs) -> bool {
    use metfor::rh;

    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{DewPoint, Pressure, Temperature};

        let pres_data = sndg.sounding().get_profile(Pressure);
        let t_data = sndg.sounding().get_profile(Temperature);
        let td_data = sndg.sounding().get_profile(DewPoint);
        let mut profile = izip!(pres_data, t_data, td_data)
            .filter_map(|triplet| {
                if let (Some(p), Some(t), Some(td)) = (*triplet.0, *triplet.1, *triplet.2) {
                    match rh(t, td) {
                        Ok(rh) => Some((p, rh)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .filter_map(|pair| {
                let (p, rh) = pair;
                if p > config::MINP {
                    let ScreenCoords { y, .. } =
                        ac.rh_omega.convert_wp_to_screen(WPCoords { w: 0.0, p });
                    let bb = ac.rh_omega.bounding_box_in_screen_coords();
                    let x = bb.lower_left.x + bb.width() * rh;

                    Some(ScreenCoords { x, y })
                } else {
                    None
                }
            });

        let line_width = config.bar_graph_line_width;
        let mut rgba = config.rh_rgba;
        rgba.3 *= 0.75;

        cr.set_line_width(cr.device_to_user_distance(line_width, 0.0).0);
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        let mut previous: Option<ScreenCoords>;
        let mut curr: Option<ScreenCoords> = None;
        let mut next: Option<ScreenCoords> = None;
        loop {
            previous = curr;
            curr = next;
            next = profile.next();

            const XMIN: f64 = 0.0;
            let xmax: f64;
            let ymin: f64;
            let ymax: f64;
            if let (Some(p), Some(c), Some(n)) = (previous, curr, next) {
                // In the middle - most common
                xmax = c.x;
                let down = (c.y - p.y) / 2.0;
                let up = (n.y - c.y) / 2.0;
                ymin = c.y - down;
                ymax = c.y + up;
            } else if let (Some(p), Some(c), None) = (previous, curr, next) {
                // Last point
                xmax = c.x;
                let down = (c.y - p.y) / 2.0;
                let up = down;
                ymin = c.y - down;
                ymax = c.y + up;
            } else if let (None, Some(c), Some(n)) = (previous, curr, next) {
                // First point
                xmax = c.x;
                let up = (n.y - c.y) / 2.0;
                let down = up;
                ymin = c.y - down;
                ymax = c.y + up;
            } else if let (Some(_), None, None) = (previous, curr, next) {
                // Done - get out of here
                break;
            } else if let (None, None, Some(_)) = (previous, curr, next) {
                // Just getting into the loop - do nothing
                continue;
            } else if let (None, None, None) = (previous, curr, next) {
                return false;
            } else {
                // Impossible state
                unreachable!();
            }

            cr.rectangle(XMIN, ymin, xmax, ymax - ymin);
            cr.fill_preserve();
            cr.stroke();
        }
    } else {
        return false;
    }
    true
}

fn draw_omega_profile(args: DrawingArgs) -> bool {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{Pressure, PressureVerticalVelocity};

        let pres_data = sndg.sounding().get_profile(Pressure);
        let omega_data = sndg.sounding().get_profile(PressureVerticalVelocity);
        let line_width = config.profile_line_width;
        let line_rgba = config.omega_rgba;

        let profile_data = pres_data
            .iter()
            .zip(omega_data.iter())
            .filter_map(|val_pair| {
                if let (Some(p), Some(w)) = (*val_pair.0, *val_pair.1) {
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
