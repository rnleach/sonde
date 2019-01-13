use crate::{
    app::{config, config::Rgba, AppContext, AppContextPointer},
    coords::{SDCoords, ScreenCoords, ScreenRect, XYCoords},
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, plot_curve_from_points},
        Drawable, DrawingArgs, MasterDrawable,
    },
};
use gtk::{prelude::*, DrawingArea};
use itertools::izip;
use metfor::{Knots, Quantity, WindSpdDir, WindUV};
use std::rc::Rc;

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
        let WindUV { u, v } = WindUV::<Knots>::from(coords.spd_dir);

        let x = u / (config::MAX_SPEED * 2.0) + 0.5;
        let y = v / (config::MAX_SPEED * 2.0) + 0.5;

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
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError> {
        let da: DrawingArea = acp.fetch_widget("hodograph_area")?;

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.hodo.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.hodo.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.hodo.button_press_event(ev, &ac));

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

        Ok(())
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    fn draw_background_fill(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let mut do_draw = true;
        let rgba = config.background_band_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        for pnts in config::ISO_SPEED_PNTS.iter() {
            let mut pnts = pnts
                .iter()
                .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));

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

    fn draw_background_lines(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        if config.show_iso_speed {
            for pnts in config::ISO_SPEED_PNTS.iter() {
                let pnts = pnts
                    .iter()
                    .map(|xy_coords| self.convert_xy_to_screen(*xy_coords));
                plot_curve_from_points(
                    cr,
                    config.background_line_width,
                    config.iso_speed_rgba,
                    pnts,
                );
            }

            let origin = self.convert_sd_to_screen(SDCoords {
                spd_dir: WindSpdDir {
                    speed: Knots(0.0),
                    direction: 360.0,
                },
            });
            for pnts in [
                30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0, 360.0,
            ]
            .iter()
            .map(|d| {
                let end_point = self.convert_sd_to_screen(SDCoords {
                    spd_dir: WindSpdDir {
                        speed: config::MAX_SPEED,
                        direction: *d,
                    },
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
    }

    fn collect_labels(&self, args: DrawingArgs<'_, '_>) -> Vec<(String, ScreenRect)> {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let mut labels = vec![];

        let screen_edges = self.calculate_plot_edges(cr, ac);

        if config.show_iso_speed {
            for &s in &config::ISO_SPEED {
                for direction in &[240.0] {
                    let label = format!("{:.0}", s.unpack());

                    let extents = cr.text_extents(&label);

                    let ScreenCoords {
                        x: mut screen_x,
                        y: mut screen_y,
                    } = self.convert_sd_to_screen(SDCoords {
                        spd_dir: WindSpdDir {
                            speed: s,
                            direction: *direction,
                        },
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

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        vec![("Hodograph".to_owned(), ac.config.borrow().label_rgba)]
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    fn draw_data(&self, args: DrawingArgs<'_, '_>) {
        draw_data(args);
    }

    /***********************************************************************************************
     * Overlays Drawing.
     **********************************************************************************************/
    fn draw_active_sample(&self, args: DrawingArgs<'_, '_>) {
        if !self.has_data() {
            return;
        }

        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let spd_dir = if let Some(sample) = ac.get_sample() {
            if let (Some(pressure), Some(wind)) =
                (sample.pressure.into_option(), sample.wind.into_option())
            {
                if pressure >= config.min_hodo_pressure {
                    wind
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
        let coords = ac.hodo.convert_sd_to_screen(SDCoords { spd_dir });

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

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
}

impl MasterDrawable for HodoContext {}

fn draw_data(args: DrawingArgs<'_, '_>) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(sndg) = ac.get_sounding_for_display() {
        let sndg = sndg.sounding();
        let pres_data = sndg.pressure_profile();
        let wind_data = sndg.wind_profile();

        let profile_data = izip!(pres_data, wind_data).filter_map(|(p, wind)| {
            if let (Some(p), Some(spd_dir)) = (p.into_option(), wind.into_option()) {
                if p >= config.min_hodo_pressure {
                    let sd_coords = SDCoords { spd_dir };
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
