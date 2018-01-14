//! Module for the GUI components of the application.

use std::rc::Rc;

use cairo::{Context, Matrix, Operator};
use gtk::prelude::*;
use gtk::{DrawingArea, Notebook, TextView, Window, WindowType};
use gdk::{keyval_from_name, EventButton, EventConfigure, EventKey, EventMotion, EventScroll,
          ScrollDirection};

use app::{AppContext, AppContextPointer};
use coords::{convert_pressure_to_y, DeviceCoords, DeviceRect, XYCoords};

mod cloud;
mod control_area;
mod hodograph;
mod index_area;
mod main_window;
mod plot_context;
mod rh_omega;
mod sounding;
mod text_area;
mod utility;

pub use self::cloud::CloudContext;
pub use self::hodograph::HodoContext;
pub use self::plot_context::{PlotContext, PlotContextExt};
pub use self::rh_omega::RHOmegaContext;
pub use self::sounding::SkewTContext;
pub use self::text_area::update_text_highlight;

use self::utility::DrawingArgs;

/// Aggregation of the GUI components need for later reference.
///
/// Note: This is cloneable because Gtk+ Gui objects are cheap to clone, and just increment a
/// reference count in the gtk-rs library. So cloning this after it is initialized does not copy
/// the GUI, but instead gives a duplicate of the references to the objects.
#[derive(Clone)]
pub struct Gui {
    // Left pane
    sounding_area: DrawingArea,

    // Right pane
    hodograph_area: DrawingArea,
    index_area: DrawingArea,
    control_area: Notebook,
    text_area: TextView,

    // Profiles
    rh_omega_area: DrawingArea,
    cloud: DrawingArea,

    // Main window
    window: Window,

    // Smart pointer.
    app_context: AppContextPointer,
}

impl Gui {
    pub fn new(acp: &AppContextPointer) -> Gui {
        let gui = Gui {
            sounding_area: DrawingArea::new(),

            hodograph_area: DrawingArea::new(),
            index_area: DrawingArea::new(),
            control_area: Notebook::new(),
            text_area: TextView::new(),

            rh_omega_area: DrawingArea::new(),
            cloud: DrawingArea::new(),

            window: Window::new(WindowType::Toplevel),
            app_context: Rc::clone(acp),
        };

        sounding::SkewTContext::set_up_drawing_area(&gui.get_sounding_area(), acp);
        hodograph::HodoContext::set_up_drawing_area(&gui.get_hodograph_area(), acp);
        control_area::set_up_control_area(&gui.get_control_area(), acp);
        index_area::set_up_index_area(&gui.get_index_area());
        text_area::set_up_text_area(&gui.get_text_area(), acp);
        rh_omega::RHOmegaContext::set_up_drawing_area(&gui.get_rh_omega_area(), acp);
        cloud::CloudContext::set_up_drawing_area(&gui.get_cloud_area(), acp);

        main_window::layout(&gui, acp);

        gui
    }

    pub fn get_sounding_area(&self) -> DrawingArea {
        self.sounding_area.clone()
    }

    pub fn get_hodograph_area(&self) -> DrawingArea {
        self.hodograph_area.clone()
    }

    pub fn get_index_area(&self) -> DrawingArea {
        self.index_area.clone()
    }

    pub fn get_control_area(&self) -> Notebook {
        self.control_area.clone()
    }

    pub fn get_text_area(&self) -> TextView {
        self.text_area.clone()
    }

    pub fn get_rh_omega_area(&self) -> DrawingArea {
        self.rh_omega_area.clone()
    }

    pub fn get_cloud_area(&self) -> DrawingArea {
        self.cloud.clone()
    }

    pub fn get_window(&self) -> Window {
        self.window.clone()
    }

    pub fn draw_all(&self) {
        self.sounding_area.queue_draw();
        self.hodograph_area.queue_draw();
        self.rh_omega_area.queue_draw();
        self.cloud.queue_draw();
    }

    pub fn update_text_view(&self, ac: &AppContext) {
        if self.text_area.is_visible() {
            self::text_area::update_text_area(&self.text_area, ac);
            self::text_area::update_text_highlight(&self.text_area, ac);
        }
    }
}

trait Drawable: PlotContext + PlotContextExt {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer);
    fn draw_background(&self, args: DrawingArgs);
    fn draw_data(&self, args: DrawingArgs);
    fn draw_overlays(&self, args: DrawingArgs);

    fn init_matrix(&self, args: DrawingArgs) {
        let cr = args.cr;

        cr.save();

        let (x1, y1, x2, y2) = cr.clip_extents();
        let width = f64::abs(x2 - x1);
        let height = f64::abs(y2 - y1);

        let device_rect = DeviceRect {
            upper_left: DeviceCoords { row: 0.0, col: 0.0 },
            width,
            height,
        };
        self.set_device_rect(device_rect);
        let scale_factor = self.scale_factor();

        // Start fresh
        cr.identity_matrix();
        // Set the scale factor
        cr.scale(scale_factor, scale_factor);
        // Set origin at lower left.
        cr.transform(Matrix {
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: -1.0,
            x0: 0.0,
            y0: device_rect.height / scale_factor,
        });

        self.set_matrix(cr.get_matrix());
        cr.restore();
    }

    /// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
    fn scroll_event(&self, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
        const DELTA_SCALE: f64 = 1.05;
        const MIN_ZOOM: f64 = 1.0;
        const MAX_ZOOM: f64 = 10.0;

        let pos = self.convert_device_to_xy(DeviceCoords::from(event.get_position()));
        let dir = event.get_direction();

        let old_zoom = self.get_zoom_factor();
        let mut new_zoom = old_zoom;

        match dir {
            ScrollDirection::Up => {
                new_zoom *= DELTA_SCALE;
            }
            ScrollDirection::Down => {
                new_zoom /= DELTA_SCALE;
            }
            _ => {}
        }

        if new_zoom < MIN_ZOOM {
            new_zoom = MIN_ZOOM;
        } else if new_zoom > MAX_ZOOM {
            new_zoom = MAX_ZOOM;
        }
        self.set_zoom_factor(new_zoom);

        let mut translate = self.get_translate();
        translate = XYCoords {
            x: pos.x - old_zoom / new_zoom * (pos.x - translate.x),
            y: pos.y - old_zoom / new_zoom * (pos.y - translate.y),
        };
        self.set_translate(translate);
        self.bound_view();
        self.mark_background_dirty();

        ac.update_all_gui();

        Inhibit(true)
    }

    fn button_press_event(&self, event: &EventButton) -> Inhibit {
        // Left mouse button
        if event.get_button() == 1 {
            self.set_last_cursor_position(Some(event.get_position().into()));
            self.set_left_button_pressed(true);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn button_release_event(&self, event: &EventButton) -> Inhibit {
        if event.get_button() == 1 {
            self.set_last_cursor_position(None);
            self.set_left_button_pressed(false);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn leave_event(&self, ac: &AppContextPointer) -> Inhibit {
        self.set_last_cursor_position(None);
        ac.set_sample(None);
        ac.update_all_gui();

        Inhibit(false)
    }

    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        ev: &EventMotion,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);
                let new_position = DeviceCoords::from(ev.get_position());
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
                ac.update_all_gui();
            }
        }
        Inhibit(false)
    }

    fn key_press_event(event: &EventKey, ac: &AppContextPointer) -> Inhibit {
        let keyval = event.get_keyval();
        if keyval == keyval_from_name("Right") || keyval == keyval_from_name("KP_Right") {
            ac.display_next();
            Inhibit(true)
        } else if keyval == keyval_from_name("Left") || keyval == keyval_from_name("KP_Left") {
            ac.display_previous();
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn size_allocate_event(&self, da: &DrawingArea) {
        self.update_cache_allocations(da);
    }

    fn configure_event(&self, event: &EventConfigure) -> bool {
        let rect = self.get_device_rect();
        let (width, height) = event.get_size();
        if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON
            || (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
        {
            self.mark_background_dirty();
        }
        false
    }

    fn draw_background_cached(&self, args: DrawingArgs) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if self.is_background_dirty() {
            let tmp_cr = Context::new(&self.get_background_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            let rgba = config.background_rgba;
            tmp_cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.bound_view();

            self.draw_background(tmp_args);

            self.clear_background_dirty();
        }

        cr.set_source_surface(&self.get_background_layer(), 0.0, 0.0);
        cr.paint();
    }

    fn draw_data_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_data_dirty() {
            let tmp_cr = Context::new(&self.get_data_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.draw_data(tmp_args);

            self.clear_data_dirty();
        }

        cr.set_source_surface(&self.get_data_layer(), 0.0, 0.0);
        cr.paint();
    }

    fn draw_overlay_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_overlay_dirty() {
            let tmp_cr = Context::new(&self.get_overlay_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.draw_overlays(tmp_args);

            self.clear_overlay_dirty();
        }

        cr.set_source_surface(&self.get_overlay_layer(), 0.0, 0.0);
        cr.paint();
    }
}

trait MasterDrawable: Drawable {
    fn draw_callback(&self, cr: &Context, acp: &AppContextPointer) -> Inhibit {
        let args = DrawingArgs::new(acp, cr);

        self.init_matrix(args);
        self.draw_background_cached(args);
        self.draw_data_cached(args);
        self.draw_overlay_cached(args);

        Inhibit(false)
    }
}

trait SlaveProfileDrawable: Drawable {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64;
    fn set_translate_y(&self, new_translate: XYCoords);

    fn draw_callback(&self, cr: &Context, acp: &AppContextPointer) -> Inhibit {
        let args = DrawingArgs::new(acp, cr);

        let device_height = self.get_device_rect().height;
        let device_width = self.get_device_rect().width;
        let aspect_ratio = device_height / device_width;

        self.set_zoom_factor(aspect_ratio * self.get_master_zoom(acp));
        self.set_translate_y(acp.skew_t.get_translate());
        self.bound_view();

        self.init_matrix(args);
        self.draw_background_cached(args);
        self.draw_data_cached(args);
        self.draw_overlay_cached(args);

        Inhibit(false)
    }

    fn draw_dendtritic_snow_growth_zone(&self, args: DrawingArgs) {
        use sounding_base::Profile::Pressure;

        let (ac, cr) = (args.ac, args.cr);

        if !ac.config.borrow().show_dendritic_zone {
            return;
        }

        // If is plottable, draw snow growth zones
        if let Some(ref snd) = ac.get_sounding_for_display() {
            let bb = self.bounding_box_in_screen_coords();
            let (left, right) = (bb.lower_left.x, bb.upper_right.x);

            let rgba = ac.config.borrow().dendritic_zone_rgba;
            cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

            for (bottom_p, top_p) in ::sounding_analysis::dendritic_growth_zone(snd, Pressure) {
                let mut coords = [
                    (left, bottom_p),
                    (left, top_p),
                    (right, top_p),
                    (right, bottom_p),
                ];

                // Convert points to screen coords
                for coord in &mut coords {
                    coord.1 = convert_pressure_to_y(coord.1);
                }

                let mut coord_iter = coords.iter();
                for coord in coord_iter.by_ref().take(1) {
                    cr.move_to(coord.0, coord.1);
                }
                for coord in coord_iter {
                    cr.line_to(coord.0, coord.1);
                }

                cr.close_path();
                cr.fill();
            }
        }
    }
}
