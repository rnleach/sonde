use crate::{
    app::{
        config::{self, HelicityType, Rgba, StormMotionType},
        AppContext, AppContextPointer,
    },
    coords::{SDCoords, ScreenCoords, ScreenRect, XYCoords},
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, draw_filled_polygon, plot_curve_from_points},
        Drawable, DrawingArgs, MasterDrawable,
    },
};
use gdk::EventButton;
use gtk::{prelude::*, DrawingArea, Menu, MenuItem, RadioMenuItem, SeparatorMenuItem};
use itertools::izip;
use metfor::{Knots, Meters, Quantity, WindSpdDir, WindUV};
use std::{iter::once, rc::Rc};

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

        build_hodograph_area_context_menu(acp)?;

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
        draw_data_overlays(args);
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
    fn button_press_event(&self, event: &EventButton, ac: &AppContextPointer) -> Inhibit {
        // Left mouse button
        if event.get_button() == 1 {
            self.set_last_cursor_position(Some(event.get_position().into()));
            self.set_left_button_pressed(true);
            Inhibit(true)
        } else if event.get_button() == 3 {
            if let Ok(menu) = ac.fetch_widget::<Menu>("hodograph_context_menu") {
                // waiting for version 3.22...
                // let ev: &::gdk::Event = evt;
                // menu.popup_at_pointer(ev);
                menu.popup_easy(3, 0)
            }
            Inhibit(false)
        } else {
            Inhibit(false)
        }
    }
}

impl MasterDrawable for HodoContext {}

/**************************************************************************************************
 *                                   DrawingArea set up
 **************************************************************************************************/
macro_rules! make_heading {
    ($menu:ident, $label:expr) => {
        let heading = MenuItem::new_with_label($label);
        heading.set_sensitive(false);
        $menu.append(&heading);
    };
}

fn build_hodograph_area_context_menu(acp: &AppContextPointer) -> Result<(), SondeError> {
    let menu: Menu = acp.fetch_widget("hodograph_context_menu")?;
    let config = acp.config.borrow();

    make_heading!(menu, "Helicity");
    let sfc_to_3km = RadioMenuItem::new_with_label("Surface to 3km");
    let effective = RadioMenuItem::new_with_label_from_widget(&sfc_to_3km, "Effective Inflow");

    match config.helicity_layer {
        HelicityType::SurfaceTo3km => sfc_to_3km.set_active(true),
        HelicityType::Effective => effective.set_active(true),
    }

    fn handle_helicity_type_toggle(
        button: &RadioMenuItem,
        htype: HelicityType,
        ac: &AppContextPointer,
    ) {
        if button.get_active() {
            ac.config.borrow_mut().helicity_layer = htype;
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
        }
    }

    let ac = Rc::clone(acp);
    sfc_to_3km.connect_toggled(move |button| {
        handle_helicity_type_toggle(button, HelicityType::SurfaceTo3km, &ac);
    });

    let ac = Rc::clone(acp);
    effective.connect_toggled(move |button| {
        handle_helicity_type_toggle(button, HelicityType::Effective, &ac);
    });

    menu.append(&sfc_to_3km);
    menu.append(&effective);

    menu.append(&SeparatorMenuItem::new());

    make_heading!(menu, "Helicity Storm");
    let right_mover = RadioMenuItem::new_with_label("Right Mover");
    let left_mover = RadioMenuItem::new_with_label_from_widget(&right_mover, "Left Mover");

    match config.helicity_storm_motion {
        StormMotionType::RightMover => right_mover.set_active(true),
        StormMotionType::LeftMover => left_mover.set_active(true),
    }

    fn handle_storm_motion_toggle(
        button: &RadioMenuItem,
        stype: StormMotionType,
        ac: &AppContextPointer,
    ) {
        if button.get_active() {
            ac.config.borrow_mut().helicity_storm_motion = stype;
            ac.mark_data_dirty();
            crate::gui::draw_all(&ac);
        }
    }

    let ac = Rc::clone(acp);
    right_mover.connect_toggled(move |button| {
        handle_storm_motion_toggle(button, StormMotionType::RightMover, &ac);
    });

    let ac = Rc::clone(acp);
    left_mover.connect_toggled(move |button| {
        handle_storm_motion_toggle(button, StormMotionType::LeftMover, &ac);
    });

    menu.append(&right_mover);
    menu.append(&left_mover);

    menu.show_all();

    Ok(())
}

/**************************************************************************************************
 *                                   Data Layer Drawing
 **************************************************************************************************/
fn draw_data(args: DrawingArgs<'_, '_>) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        let sndg = anal.sounding();
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

fn draw_data_overlays(args: DrawingArgs<'_, '_>) {
    draw_helicity_fill(args);
    draw_storm_motion_and_mean_wind(args);
}

fn draw_helicity_fill(args: DrawingArgs<'_, '_>) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        // Get the storm motion
        let motion = {
            let motion = match config.helicity_storm_motion {
                StormMotionType::RightMover => anal.right_mover(),
                StormMotionType::LeftMover => anal.left_mover(),
            }
            .map(WindSpdDir::<Knots>::from);

            if let Some(motion) = motion {
                motion
            } else {
                return;
            }
        };

        let pnts = {
            let layer = match config.helicity_layer {
                HelicityType::SurfaceTo3km => {
                    sounding_analysis::layer_agl(anal.sounding(), Meters(3000.0)).ok()
                }
                HelicityType::Effective => anal.effective_inflow_layer(),
            };

            if let Some((bottom_p, top_p)) = layer.and_then(|lyr| {
                lyr.bottom
                    .pressure
                    .into_option()
                    .and_then(|bp| lyr.top.pressure.into_option().map(|tp| (bp, tp)))
            }) {
                let pnts = izip!(
                    anal.sounding().pressure_profile(),
                    anal.sounding().wind_profile()
                )
                .filter_map(|(p_opt, w_opt)| {
                    p_opt.into_option().and_then(|p| w_opt.map(|w| (p, w)))
                })
                .skip_while(move |(p, _)| *p > bottom_p)
                .take_while(move |(p, _)| *p >= top_p)
                .map(|(_, w)| w);

                once(motion)
                    .chain(pnts)
                    .map(|spd_dir| SDCoords { spd_dir })
                    .map(|coord| ac.hodo.convert_sd_to_screen(coord))
            } else {
                return;
            }
        };

        let rgba = config.helicity_rgba;
        draw_filled_polygon(&cr, rgba, pnts);
    }
}

fn draw_storm_motion_and_mean_wind(args: DrawingArgs<'_, '_>) {
    let (ac, cr) = (args.ac, args.cr);
    let config = ac.config.borrow();

    if let Some(anal) = ac.get_sounding_for_display() {
        let anal = anal.borrow();
        if let (Some(rm), Some(lm), Some(mw)) = (
            anal.right_mover().into_option(),
            anal.left_mover().into_option(),
            anal.mean_wind().into_option(),
        ) {
            let rm = WindSpdDir::<Knots>::from(rm);
            let lm = WindSpdDir::<Knots>::from(lm);
            let mw = WindSpdDir::<Knots>::from(mw);

            let pnt_size = cr.device_to_user_distance(6.0, 0.0).0;

            let mut coords_rm = ac.hodo.convert_sd_to_screen(SDCoords { spd_dir: rm });
            let mut coords_lm = ac.hodo.convert_sd_to_screen(SDCoords { spd_dir: lm });
            let mut coords_mw = ac.hodo.convert_sd_to_screen(SDCoords { spd_dir: mw });

            let sm_rgba = config.storm_motion_rgba;
            let mw_rgba = config.storm_motion_rgba;
            cr.set_source_rgba(sm_rgba.0, sm_rgba.1, sm_rgba.2, sm_rgba.3);
            cr.arc(
                coords_rm.x,
                coords_rm.y,
                pnt_size,
                0.0,
                2.0 * ::std::f64::consts::PI,
            );
            cr.fill();

            cr.arc(
                coords_lm.x,
                coords_lm.y,
                pnt_size,
                0.0,
                2.0 * ::std::f64::consts::PI,
            );
            cr.fill();

            coords_rm.x += 0.025;
            coords_lm.x += 0.025;

            ac.hodo.draw_tag("RM", coords_rm, sm_rgba, args);
            ac.hodo.draw_tag("LM", coords_lm, sm_rgba, args);

            cr.set_source_rgba(mw_rgba.0, mw_rgba.1, mw_rgba.2, mw_rgba.3);
            cr.arc(
                coords_mw.x,
                coords_mw.y,
                pnt_size,
                0.0,
                2.0 * ::std::f64::consts::PI,
            );
            cr.fill();

            coords_mw.x += 0.025;
            ac.hodo.draw_tag("MW", coords_mw, mw_rgba, args);
        }
    }
}
