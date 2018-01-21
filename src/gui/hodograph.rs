use std::rc::Rc;

use gdk::EventMask;
use gtk::prelude::*;
use gtk::DrawingArea;

use app::{config, AppContext, AppContextPointer};
use coords::{Rect, SDCoords, ScreenCoords, ScreenRect, XYCoords};
use gui::{Drawable, DrawingArgs, Labels, MasterDrawable};
use gui::plot_context::{GenericContext, HasGenericContext, PlotContextExt};
use gui::utility::{check_overlap_then_add, plot_curve_from_points};

pub struct HodoContext {
    generic: GenericContext,
}

impl HodoContext {
    pub fn new() -> Self {
        HodoContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_sd_to_xy(coords: SDCoords) -> XYCoords {
        let radius = coords.speed / 2.0 / config::MAX_SPEED;
        let angle = (270.0 - coords.dir).to_radians();

        let x = radius * angle.cos() + 0.5;
        let y = radius * angle.sin() + 0.5;
        XYCoords { x, y }
    }

    pub fn convert_sd_to_screen(&self, coords: SDCoords) -> ScreenCoords {
        let xy = HodoContext::convert_sd_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }
}

impl HasGenericContext for HodoContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for HodoContext {}

impl Drawable for HodoContext {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.hodo.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.hodo.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.hodo.button_press_event(ev));

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| ac.hodo.button_release_event(ev));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.hodo.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.hodo.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| HodoContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.hodo.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.hodo.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    fn draw_background(&self, args: DrawingArgs) {
        let config = args.ac.config.borrow();

        if config.show_background_bands {
            draw_background_fill(args);
        }

        if config.show_iso_speed {
            draw_background_lines(args);
        }

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
        draw_data(args);
    }

    fn draw_overlays(&self, args: DrawingArgs) {
        draw_overlays(args);
    }
}

impl MasterDrawable for HodoContext {}

impl Labels for HodoContext {
    fn collect_labels(&self, args: DrawingArgs) -> Vec<(String, ScreenRect)> {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);

        if config.show_iso_speed {
            for &s in &config::ISO_SPEED {
                for direction in &[240.0] {
                    let label = format!("{:.0}", s);

                    let extents = cr.text_extents(&label);

                    let ScreenCoords {
                        x: mut screen_x,
                        y: mut screen_y,
                    } = self.convert_sd_to_screen(SDCoords {
                        speed: s,
                        dir: *direction,
                    });
                    screen_y -= extents.height / 2.0;
                    screen_x -= extents.width / 2.0;

                    let label_lower_left = ScreenCoords {
                        x: screen_x,
                        y: screen_y,
                    };
                    let label_upper_right = ScreenCoords {
                        x: screen_x + extents.width,
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
        }

        labels
    }

    fn build_legend_strings(_ac: &AppContext) -> Vec<String> {
        vec!["Hodograph".to_owned()]
    }
}

fn draw_background_fill(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);

    let mut rgba = ac.config.borrow().background_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    let rect = ac.hodo.bounding_box_in_screen_coords();
    cr.rectangle(
        rect.lower_left.x,
        rect.lower_left.y,
        rect.width(),
        rect.height(),
    );
    cr.fill();

    let mut do_draw = true;
    rgba = ac.config.borrow().background_band_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

    for pnts in config::ISO_SPEED_PNTS.iter() {
        let mut pnts = pnts.iter()
            .map(|xy_coords| ac.hodo.convert_xy_to_screen(*xy_coords));

        if let Some(pnt) = pnts.by_ref().next() {
            cr.move_to(pnt.x, pnt.y);
        }
        if do_draw {
            for pnt in pnts {
                cr.line_to(pnt.x, pnt.y);
            }
        } else {
            for pnt in pnts.rev() {
                cr.line_to(pnt.x, pnt.y);
            }
        }
        cr.close_path();
        if do_draw {
            cr.fill();
        }
        do_draw = !do_draw;
    }
}

fn draw_background_lines(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);

    let config = ac.config.borrow();

    for pnts in config::ISO_SPEED_PNTS.iter() {
        let pnts = pnts.iter()
            .map(|xy_coords| ac.hodo.convert_xy_to_screen(*xy_coords));
        plot_curve_from_points(
            cr,
            config.background_line_width,
            config.iso_speed_rgba,
            pnts,
        );
    }

    let origin = ac.hodo.convert_sd_to_screen(SDCoords {
        speed: 0.0,
        dir: 360.0,
    });
    for pnts in [
        30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0, 360.0
    ].iter()
        .map(|d| {
            let end_point = ac.hodo.convert_sd_to_screen(SDCoords {
                speed: config::MAX_SPEED,
                dir: *d,
            });
            [origin, end_point]
        }) {
        plot_curve_from_points(
            cr,
            config.background_line_width,
            config.iso_speed_rgba,
            pnts.iter().cloned(),
        );
    }
}

fn draw_data(args: DrawingArgs) {
    use sounding_base::Profile::{Pressure, WindDirection, WindSpeed};

    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        let pres_data = sndg.get_profile(Pressure);
        let speed_data = sndg.get_profile(WindSpeed);
        let dir_data = sndg.get_profile(WindDirection);

        let profile_data = izip!(pres_data, speed_data, dir_data).filter_map(|triplet| {
            if let (Some(p), Some(speed), Some(dir)) = (*triplet.0, *triplet.1, *triplet.2) {
                if p >= config.min_hodo_pressure {
                    let sd_coords = SDCoords { speed, dir };
                    Some(ac.hodo.convert_sd_to_screen(sd_coords))
                } else {
                    None
                }
            } else {
                None
            }
        });

        plot_curve_from_points(
            cr,
            config.velocity_line_width,
            config.veclocity_rgba,
            profile_data,
        );
    }
}

fn draw_overlays(args: DrawingArgs) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if !config.show_active_readout {
        return;
    }

    let (speed, dir) = if let Some(sample) = ac.get_sample() {
        if let (Some(pressure), Some(speed), Some(dir)) =
            (sample.pressure, sample.speed, sample.direction)
        {
            if pressure >= config.min_hodo_pressure {
                (speed, dir)
            } else {
                return;
            }
        } else {
            return;
        }
    } else {
        return;
    };

    let pnt_size = cr.device_to_user_distance(5.0, 0.0).0;
    let coords = ac.hodo.convert_sd_to_screen(SDCoords { speed, dir });

    let rgba = config.active_readout_line_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.arc(
        coords.x,
        coords.y,
        pnt_size,
        0.0,
        2.0 * ::std::f64::consts::PI,
    );
    cr.fill();
}
