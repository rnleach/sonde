use crate::{
    app::{
        config::{self, HelicityType, Rgba, StormMotionType},
        sample::Sample,
        AppContext, AppContextPointer, ZoomableDrawingAreas,
    },
    coords::{SDCoords, ScreenCoords, ScreenRect, XYCoords},
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, draw_filled_polygon, plot_curve_from_points},
        Drawable, DrawingArgs, MasterDrawable,
    },
};
use gtk::{
    gio::{SimpleAction, SimpleActionGroup},
    prelude::*,
    DrawingArea, EventControllerKey, EventControllerMotion, EventControllerScroll,
    EventControllerScrollFlags, GestureClick, Inhibit, Window,
};
use itertools::izip;
use metfor::{Knots, Meters, Quantity, WindSpdDir, WindUV};
use sounding_analysis::DataRow;
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

        // Set up the drawing function.
        let ac = Rc::clone(acp);
        da.set_draw_func(move |_da, cr, _width, _height| {
            ac.hodo.draw_callback(cr, &ac);
        });

        // Set up the scroll (or zoom in/out) callbacks.
        let ac = Rc::clone(acp);
        let scroll_control = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        scroll_control.connect_scroll(move |_scroll_control, _dx, dy| {
            ac.mark_background_dirty();
            ac.hodo.scroll_event(dy, &ac);

            Inhibit(true)
        });
        da.add_controller(scroll_control);

        // Set up the button clicks.
        let left_mouse_button = GestureClick::builder().build();

        let ac = Rc::clone(acp);
        left_mouse_button.connect_pressed(move |_mouse_button, _n_pressed, x, y| {
            ac.hodo.left_button_press_event((x, y), &ac);
        });

        let ac = Rc::clone(acp);
        left_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.hodo.left_button_release_event((x, y), &ac);
        });

        da.add_controller(left_mouse_button);

        let right_mouse_button = GestureClick::builder().button(3).build();
        let ac = Rc::clone(acp);
        right_mouse_button.connect_released(move |_mouse_button, _n_press, x, y| {
            ac.hodo.right_button_release_event((x, y), &ac);
        });
        da.add_controller(right_mouse_button);

        // Set up the mouse motion events
        let mouse_motion = EventControllerMotion::new();

        let ac = Rc::clone(acp);
        mouse_motion.connect_motion(move |mouse_motion, x, y| {
            ac.hodo.mouse_motion_event(mouse_motion, (x, y), &ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_enter(move |_mouse_motion, _x, _y| {
            ac.hodo.enter_event(&ac);
        });

        let ac = Rc::clone(acp);
        mouse_motion.connect_leave(move |_mouse_motion| {
            ac.hodo.leave_event(&ac);
        });

        da.add_controller(mouse_motion);

        // Set up the key presses.
        let key_press = EventControllerKey::new();
        let ac = Rc::clone(acp);
        key_press.connect_key_pressed(move |_key_press, key, _code, _key_modifier| {
            HodoContext::key_press_event(key, &ac)
        });
        da.add_controller(key_press);

        let ac = Rc::clone(acp);
        da.connect_resize(move |da, width, height| {
            // TODO merge below methods into one.
            ac.hodo.size_allocate_event(da);
            ac.hodo.resize_event(width, height, &ac);
        });

        build_hodograph_area_context_menu(acp)?;

        Ok(())
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    fn draw_background_fill(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        if config.show_background_bands {
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
                    cr.fill().unwrap();
                }
                do_draw = !do_draw;
            }
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

                    let extents = cr.text_extents(&label).unwrap();

                    let ScreenCoords {
                        x: mut screen_x,
                        y: mut screen_y,
                    } = self.convert_sd_to_screen(SDCoords {
                        spd_dir: WindSpdDir {
                            speed: s,
                            direction: *direction,
                        },
                    });
                    screen_y -= extents.height() / 2.0;
                    screen_x -= extents.width() / 2.0;

                    let label_lower_left = ScreenCoords {
                        x: screen_x,
                        y: screen_y,
                    };
                    let label_upper_right = ScreenCoords {
                        x: screen_x + extents.width(),
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

        let (ac, config) = (args.ac, args.ac.config.borrow());

        let spd_dir = match *ac.get_sample() {
            Sample::Sounding {
                data: DataRow { pressure, wind, .. },
                ..
            } => {
                if let Some(wind) = pressure
                    .into_option()
                    .filter(|pr| *pr > config.min_hodo_pressure)
                    .and(wind.into_option())
                {
                    wind
                } else {
                    return;
                }
            }
            Sample::FirePlume { .. } | Sample::None => return,
        };

        let coords = ac.hodo.convert_sd_to_screen(SDCoords { spd_dir });
        let rgba = config.active_readout_line_rgba;
        Self::draw_point(coords, rgba, args);
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/

    fn right_button_release_event(&self, _position: (f64, f64), ac: &AppContextPointer) {
        if let Ok(popover) = ac.fetch_widget::<gtk::PopoverMenu>("hodo_popover") {
            if let Some(pos) = self.get_last_cursor_position() {
                let llx: i32 = pos.col as i32;
                let lly: i32 = pos.row as i32;
                let rect = gtk::gdk::Rectangle::new(llx, lly, 1, 1);
                popover.set_pointing_to(Some(&rect));
                popover.popup();
            }
        }
    }

    fn enter_event(&self, ac: &AppContextPointer) {
        ac.set_last_focus(ZoomableDrawingAreas::Hodo);
    }
}

impl MasterDrawable for HodoContext {}

/**************************************************************************************************
 *                                   DrawingArea set up
 **************************************************************************************************/
fn build_hodograph_area_context_menu(acp: &AppContextPointer) -> Result<(), SondeError> {
    let window: Window = acp.fetch_widget("main_window")?;
    let config = acp.config.borrow();

    let hodo_group = SimpleActionGroup::new();
    window.insert_action_group("hodo", Some(&hodo_group));

    // Configure the layer to use for helicity calculations
    let current_helicity_layer = match config.helicity_layer {
        HelicityType::SurfaceTo3km => "sfc_to_3km",
        HelicityType::Effective => "effective",
    };

    let helicity_layer_action = SimpleAction::new_stateful(
        "helicity_layer_action",
        Some(gtk::glib::VariantTy::STRING),
        current_helicity_layer.into(),
    );

    let ac = Rc::clone(acp);
    helicity_layer_action.connect_activate(move |action, variant| {
        let val: &str = variant.unwrap().str().unwrap();
        action.set_state(val.into());

        let layer = match val {
            "sfc_to_3km" => HelicityType::SurfaceTo3km,
            "effective" => HelicityType::Effective,
            _ => unreachable!(),
        };

        ac.config.borrow_mut().helicity_layer = layer;
        ac.mark_data_dirty();
        crate::gui::draw_all(&ac);
        crate::gui::update_text_views(&ac);
    });
    hodo_group.add_action(&helicity_layer_action);

    // Show/hide the helicity overlay (fill the helicity area).
    let ac = acp.clone();
    let show_action = SimpleAction::new("show_helicity_overlay", None);
    show_action.connect_activate(move |_action, _variant| {
        let mut config = ac.config.borrow_mut();
        config.show_helicity_overlay = !config.show_helicity_overlay;
        ac.mark_data_dirty();
        crate::gui::draw_all(&ac);
        crate::gui::update_text_views(&ac);
    });
    hodo_group.add_action(&show_action);

    // Configure the helicity storm type (left move or right mover)
    let current_helicity_type = match config.helicity_storm_motion {
        StormMotionType::RightMover => "right",
        StormMotionType::LeftMover => "left",
    };

    let helicity_type_action = SimpleAction::new_stateful(
        "helicity_type",
        Some(gtk::glib::VariantTy::STRING),
        current_helicity_type.into(),
    );

    let ac = Rc::clone(acp);
    helicity_type_action.connect_activate(move |action, variant| {
        let val: &str = variant.unwrap().str().unwrap();
        action.set_state(val.into());

        let direction = match val {
            "right" => StormMotionType::RightMover,
            "left" => StormMotionType::LeftMover,
            _ => unreachable!(),
        };

        ac.config.borrow_mut().helicity_storm_motion = direction;
        ac.mark_data_dirty();
        crate::gui::draw_all(&ac);
        crate::gui::update_text_views(&ac);
    });
    hodo_group.add_action(&helicity_type_action);

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
            config.wind_rgba,
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

    if !config.show_helicity_overlay {
        return;
    }

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

            let pnt_size = cr.device_to_user_distance(6.0, 0.0).unwrap().0;

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
            cr.fill().unwrap();

            cr.arc(
                coords_lm.x,
                coords_lm.y,
                pnt_size,
                0.0,
                2.0 * ::std::f64::consts::PI,
            );
            cr.fill().unwrap();

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
            cr.fill().unwrap();

            coords_mw.x += 0.025;
            ac.hodo.draw_tag("MW", coords_mw, mw_rgba, args);
        }
    }
}
