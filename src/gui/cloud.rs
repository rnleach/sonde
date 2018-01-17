use std::rc::Rc;

use gdk::{EventMask, EventMotion, EventScroll};
use gtk::prelude::*;
use gtk::DrawingArea;

use sounding_base::{DataRow, Sounding};

use app::{config, AppContextPointer, AppContext};
use coords::{convert_pressure_to_y, DeviceCoords, PPCoords, ScreenCoords, XYCoords};
use gui::{Drawable, SlaveProfileDrawable};
use gui::plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt};
use gui::utility::{plot_curve_from_points, DrawingArgs};

pub struct CloudContext {
    generic: GenericContext,
}

impl CloudContext {
    pub fn new() -> Self {
        CloudContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_pp_to_xy(coords: PPCoords) -> XYCoords {
        let y = convert_pressure_to_y(coords.press);

        // The + sign below looks weird, but is correct.
        let x = coords.pcnt;

        XYCoords { x, y }
    }

    pub fn convert_xy_to_pp(coords: XYCoords) -> PPCoords {
        use std::f64;

        let press = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP))
                + f64::log10(config::MAXP),
        );
        let pcnt = coords.x;

        PPCoords { pcnt, press }
    }

    pub fn convert_pp_to_screen(&self, coords: PPCoords) -> ScreenCoords {
        let xy = CloudContext::convert_pp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_pp(&self, coords: DeviceCoords) -> PPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_pp(xy)
    }
}

impl HasGenericContext for CloudContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for CloudContext {
    fn zoom_to_envelope(&self) {}

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

impl Drawable for CloudContext {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.cloud.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.cloud.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.cloud.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| CloudContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.cloud.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.cloud.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    fn scroll_event(&self, _event: &EventScroll, _ac: &AppContextPointer) -> Inhibit {
        Inhibit(false)
    }

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
            let pp_position = self.convert_device_to_pp(position);
            let sample = ::sounding_analysis::linear_interpolate(
                &ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
                pp_position.press,
            );
            ac.set_sample(Some(sample));
            ac.mark_overlay_dirty();
            ac.update_all_gui();
        }
        Inhibit(false)
    }

    fn draw_background(&self, args: DrawingArgs) {
        self.draw_dendtritic_snow_growth_zone(args);
        draw_background_lines(args);
        self.prepare_to_make_text(args);
        self.draw_legend(args);
    }

    fn draw_data(&self, args: DrawingArgs) {
        draw_cloud_profile(args);
    }

    fn create_active_readout_text(vals: &DataRow, _snd: &Sounding) -> Vec<String> {
        let mut results = vec![];

        if let Some(cloud) = vals.cloud_fraction {
            let cld = (cloud / 10.0).round() * 10.0;
            let line = format!("{:.0}%", cld);
            results.push(line);
        }

        results
    }

    fn build_legend_strings(_ac: &AppContext) -> Vec<String> {

        vec!["Cloud Cover".to_owned()]
    }
}

impl SlaveProfileDrawable for CloudContext {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64 {
        acp.skew_t.get_zoom_factor()
    }

    fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.set_translate(translate);
    }
}

fn draw_cloud_profile(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{CloudFraction, Pressure};

        let pres_data = sndg.get_profile(Pressure);
        let c_data = sndg.get_profile(CloudFraction);
        let mut profile = izip!(pres_data, c_data)
            .filter_map(|pair| {
                if let (Some(p), Some(c)) = (*pair.0, *pair.1) {
                    Some((p, c))
                } else {
                    None
                }
            })
            .filter_map(|pair| {
                let (press, pcnt) = pair;
                if press > config::MINP {
                    Some(ac.cloud.convert_pp_to_screen(PPCoords {
                        pcnt: pcnt / 100.0,
                        press,
                    }))
                } else {
                    None
                }
            });

        let line_width = config.omega_line_width; // FIXME: use another line width?
        let mut rgba = config.cloud_rgba;
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
            } else {
                // Impossible state
                println!("Unreachable");
                break; // FIXME: Should be unreachable, check for empty clouds
                unreachable!();
            }

            cr.rectangle(XMIN, ymin, xmax, ymax - ymin);
            cr.fill_preserve();
            cr.stroke();
        }
    }
}

fn draw_background_lines(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    // Draw isobars
    if config.show_isobars {
        for pnts in config::ISOBAR_PNTS.iter() {
            let pnts = pnts.iter()
                .map(|xy_coords| ac.cloud.convert_xy_to_screen(*xy_coords));
            plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
        }
    }
}
