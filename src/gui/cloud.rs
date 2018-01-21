use std::rc::Rc;

use gdk::{EventMask, EventMotion, EventScroll};
use gtk::prelude::*;
use gtk::DrawingArea;

use sounding_base::{DataRow, Sounding};

use app::{config, AppContext, AppContextPointer};
use coords::{convert_pressure_to_y, DeviceCoords, PPCoords, ScreenCoords, ScreenRect, XYCoords};
use gui::{Drawable, Labels, SampleReadout, SlaveProfileDrawable};
use gui::plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt};
use gui::utility::{check_overlap_then_add, plot_curve_from_points, DrawingArgs};

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

    pub fn convert_screen_to_pp(&self, coords: ScreenCoords) -> PPCoords {
        let xy = self.convert_screen_to_xy(coords);
        CloudContext::convert_xy_to_pp(xy)
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

        if ac.plottable() && self.has_data() {
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
        let config = args.ac.config.borrow();

        draw_background_fill(args);
        draw_background_lines(args);

        if config.show_labels || config.show_legend {
            self.prepare_to_make_text(args);
        }

        if config.show_labels {
            self.draw_background_labels(args);
        }

        if config.show_legend {
            self.draw_legend(args);
        }
    }

    fn draw_data(&self, args: DrawingArgs) {
        draw_cloud_profile(args);
    }

    fn draw_overlays(&self, args: DrawingArgs) {
        if args.ac.config.borrow().show_active_readout {
            self.draw_active_sample(args);
        }
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

impl SampleReadout for CloudContext {
    fn create_active_readout_text(vals: &DataRow, _snd: &Sounding) -> Vec<String> {
        let mut results = vec![];

        if let Some(cloud) = vals.cloud_fraction {
            let cld = (cloud).round();
            let line = format!("{:.0}%", cld);
            results.push(line);
        }

        results
    }
}

impl Labels for CloudContext {
    fn build_legend_strings(_ac: &AppContext) -> Vec<String> {
        vec!["Cloud Cover".to_owned()]
    }

    fn collect_labels(&self, args: DrawingArgs) -> Vec<(String, ScreenRect)> {
        let (ac, cr) = (args.ac, args.cr);

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);
        let ScreenRect { lower_left, .. } = screen_edges;

        let PPCoords {
            press: screen_max_p,
            ..
        } = ac.cloud.convert_screen_to_pp(lower_left);

        for pcnt in &config::PERCENTS {
            let label = format!("{:.0}%", *pcnt);

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                x: mut xpos,
                y: mut ypos,
            } = ac.cloud.convert_pp_to_screen(PPCoords {
                pcnt: *pcnt / 100.0,
                press: screen_max_p,
            });
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

        labels
    }
}

fn draw_cloud_profile(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        use sounding_base::Profile::{CloudFraction, Pressure};

        ac.cloud.set_has_data(true);

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

        let line_width = config.bar_graph_line_width;
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
            } else if let (None, None, None) = (previous, curr, next) {
                // There is no data plot the no data and leave!
                ac.cloud.set_has_data(false);
                break;
            } else {
                // Impossible state
                unreachable!();
            }

            cr.rectangle(XMIN, ymin, xmax, ymax - ymin);
            cr.fill_preserve();
            cr.stroke();
        }
    } else {
        ac.cloud.set_has_data(false);
    }

    if !ac.cloud.has_data() {
        ac.cloud.draw_no_data(args);
    }
}

fn draw_background_fill(args: DrawingArgs) {
    let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

    if config.show_background_bands {
        let rgba = config.background_band_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        let mut lines = config::CLOUD_PERCENT_PNTS.iter();
        let mut draw = true;
        let mut prev = lines.next();
        while let Some(prev_val) = prev {
            let curr = lines.next();
            if let Some(curr_val) = curr {
                if draw {
                    let ll = ac.cloud.convert_xy_to_screen(prev_val[0]);
                    let ur = ac.cloud.convert_xy_to_screen(curr_val[1]);
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

    ac.cloud.draw_dendtritic_snow_growth_zone(args);
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

    // Draw percent values
    for line in config::CLOUD_PERCENT_PNTS.iter() {
        let pnts = line.iter()
            .map(|xy_coord| ac.cloud.convert_xy_to_screen(*xy_coord));
        plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
    }
}
