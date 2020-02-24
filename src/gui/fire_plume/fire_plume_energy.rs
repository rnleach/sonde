use crate::{
    app::{
        config::{self, Rgba},
        sample::Sample,
        AppContext, AppContextPointer, ZoomableDrawingAreas,
    },
    coords::{DeviceCoords, DtECoords, ScreenCoords, ScreenRect, XYCoords},
    errors::SondeError,
    gui::{
        plot_context::{GenericContext, HasGenericContext, PlotContext, PlotContextExt},
        utility::{check_overlap_then_add, plot_curve_from_points},
        Drawable, DrawingArgs, MasterDrawable,
    },
};
use gdk::EventMotion;
use gtk::{prelude::*, DrawingArea};
use metfor::{CelsiusDiff, JpKg, Quantity};
use sounding_analysis::{experimental::fire::lift_plume_parcel, Parcel};
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

    pub fn convert_dte_to_xy(coords: DtECoords) -> XYCoords {
        let min_e = config::MIN_FIRE_PLUME_CAPE.unpack();
        let max_e = config::MAX_FIRE_PLUME_CAPE.unpack();

        let DtECoords {
            dt,
            energy: JpKg(energy),
        } = coords;

        let x = super::convert_dt_to_x(dt);
        let y = (energy - min_e) / (max_e - min_e);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_dte(coords: XYCoords) -> DtECoords {
        let min_e = config::MIN_FIRE_PLUME_CAPE.unpack();
        let max_e = config::MAX_FIRE_PLUME_CAPE.unpack();

        let XYCoords { x, y } = coords;

        let dt = super::convert_x_to_dt(x);
        let energy = JpKg(y * (max_e - min_e) + min_e);

        DtECoords { dt, energy }
    }

    pub fn convert_dte_to_screen(&self, coords: DtECoords) -> ScreenCoords {
        let xy = FirePlumeEnergyContext::convert_dte_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_dte(&self, coords: DeviceCoords) -> DtECoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_dte(xy)
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

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.fire_plume_energy.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.fire_plume_energy.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| {
            ac.fire_plume_energy.button_press_event(ev, &ac)
        });

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| {
            ac.fire_plume_energy.button_release_event(ev)
        });

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| {
            ac.fire_plume_energy.mouse_motion_event(da, ev, &ac)
        });

        let ac = Rc::clone(acp);
        da.connect_enter_notify_event(move |_da, _ev| ac.fire_plume_energy.enter_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.fire_plume_energy.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| FirePlumeEnergyContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.fire_plume_energy.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.fire_plume_energy.size_allocate_event(da));

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
        for pnts in config::FIRE_PLUME_CAPES_PNTS.iter() {
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

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                x: mut screen_x, ..
            } = self.convert_dte_to_screen(DtECoords {
                dt,
                energy: JpKg(0.0),
            });
            screen_x -= extents.width / 2.0;

            let label_lower_left = ScreenCoords {
                x: screen_x,
                y: lower_left.y,
            };
            let label_upper_right = ScreenCoords {
                x: screen_x + extents.width,
                y: lower_left.y + extents.height,
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

        for &e in config::FIRE_PLUME_CAPES.iter().skip(1) {
            let label = format!("{:.1}", e.unpack() / 1_000.0);

            let extents = cr.text_extents(&label);

            let ScreenCoords {
                y: mut screen_y, ..
            } = self.convert_dte_to_screen(DtECoords {
                dt: CelsiusDiff(0.0),
                energy: e,
            });
            screen_y -= extents.height / 2.0;

            let label_lower_left = ScreenCoords {
                x: lower_left.x,
                y: screen_y,
            };
            let label_upper_right = ScreenCoords {
                x: lower_left.x + extents.width,
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

        labels
    }

    fn build_legend_strings(ac: &AppContext) -> Vec<(String, Rgba)> {
        let config = ac.config.borrow();

        let mut lines = Vec::with_capacity(2);

        lines.push((
            "Net CAPE (kJ/kg)".to_owned(),
            config.fire_plume_net_cape_color,
        ));
        lines.push(("NCAPE * 10".to_owned(), config.fire_plume_ncape_color));

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

        let t0 = match anal.starting_parcel_for_blow_up_anal() {
            Some(pcl) => pcl.temperature,
            None => return,
        };

        if let Some(vals) = anal.plumes() {
            let line_width = config.profile_line_width;
            let net_cape_rgba = config.fire_plume_net_cape_color;

            let capes = vals
                .iter()
                .filter_map(|plume_anal| {
                    plume_anal
                        .net_cape
                        .map(|cape| (plume_anal.parcel.temperature - t0, cape))
                })
                .map(|(dt, energy)| DtECoords { dt, energy })
                .map(|dt_coord| ac.fire_plume_energy.convert_dte_to_screen(dt_coord));

            plot_curve_from_points(cr, line_width, net_cape_rgba, capes);

            let ncape_rgba = config.fire_plume_ncape_color;
            let ncapes = vals
                .iter()
                .filter_map(|plume_anal| {
                    plume_anal
                        .net_cape
                        .and_then(|net_cape| {
                            plume_anal
                                .el_height
                                .map(|el| 10.0 * net_cape.unpack() / el.unpack())
                        })
                        .map(|ncape| (plume_anal.parcel.temperature - t0, ncape))
                })
                // Bogus the ncape into a JpKg wrapper for now instead of creating a new
                // coordinate.
                .map(|(dt, ncape)| DtECoords {
                    dt,
                    energy: JpKg(1000.0 * ncape),
                })
                .map(|dt_coord| ac.fire_plume_energy.convert_dte_to_screen(dt_coord));

            plot_curve_from_points(cr, line_width, ncape_rgba, ncapes);
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

        if let Sample::FirePlume { plume_anal, .. } = *vals {
            let dt = plume_anal.parcel.temperature - t0;

            if let Some(ncape) = plume_anal.net_cape.and_then(|cape| {
                plume_anal
                    .el_height
                    .map(|el| 10.0 * cape.unpack() / el.unpack())
            }) {
                let ncape_pnt = DtECoords {
                    dt,
                    energy: JpKg(1_000.0 * ncape),
                };
                let screen_coords_cape = ac.fire_plume_energy.convert_dte_to_screen(ncape_pnt);
                Self::draw_point(screen_coords_cape, pnt_color, args);
            }

            if let Some(cape) = plume_anal.net_cape {
                let cape_pnt = DtECoords { dt, energy: cape };
                let screen_coords_cape = ac.fire_plume_energy.convert_dte_to_screen(cape_pnt);
                Self::draw_point(screen_coords_cape, pnt_color, args);
            }
        }
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

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);
                let new_position = DeviceCoords::from(event.get_position());
                self.set_last_cursor_position(Some(new_position));

                let new_position = self.convert_device_to_xy(new_position);
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
            let position: DeviceCoords = event.get_position().into();
            self.set_last_cursor_position(Some(position));

            let sample = ac
                .get_sounding_for_display()
                .and_then(|anal| {
                    let DtECoords { dt, .. } = self.convert_device_to_dte(position);
                    let pcl = anal
                        .borrow()
                        .starting_parcel_for_blow_up_anal()
                        .map(|p0| Parcel {
                            temperature: p0.temperature + dt,
                            ..p0
                        });
                    pcl.map(|pcl| (anal, pcl))
                })
                .and_then(|(anal, pcl)| {
                    lift_plume_parcel(pcl, anal.borrow().sounding())
                        .ok()
                        .map(|(pp, pa)| Sample::FirePlume {
                            parcel: pcl,
                            profile: pp,
                            plume_anal: pa,
                        })
                })
                .unwrap_or(Sample::None);

            ac.set_sample(sample);
            ac.mark_overlay_dirty();
            crate::gui::draw_all(&ac);
        }

        Inhibit(false)
    }

    fn enter_event(&self, ac: &AppContextPointer) -> Inhibit {
        ac.set_last_focus(ZoomableDrawingAreas::FirePlumeEnergy);
        Inhibit(false)
    }
}

impl MasterDrawable for FirePlumeEnergyContext {}
