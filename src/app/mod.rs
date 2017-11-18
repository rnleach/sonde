//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.

use std::rc::Rc;
use std::cell::RefCell;

use cairo::Context;
use gtk::prelude::*;

use sounding_base::Sounding;

use errors::*;
use gui::Gui;
use coords::{DeviceCoords, ScreenCoords, TPCoords, WPCoords, XYCoords, XYRect, ScreenRect};

// Module for configuring application
pub mod config;
use app::config::Config;

/// Smart pointer for globally shareable data
pub type AppContextPointer = Rc<RefCell<AppContext>>;

/// Holds the application state. This is a singleton (not enforced) that is shared globally.
pub struct AppContext {
    // Configuration, style and layout settings.
    pub config: Config,

    // Source description is used in the legend if it is present. Not all file formats include a
    // station name or model name or base time. In bufkit files this is usually part of the file
    // name. So whatever function loads a sounding should set this to reflect where it came from.
    source_description: Option<String>,

    list: Vec<Sounding>,
    currently_displayed_index: usize,
    pub last_sample_pressure: Option<f64>,

    // Handle to the GUI
    gui: Option<Gui>,

    // Handle to skew-t context
    pub skew_t: SkewTContext,

    // Handle to RH Omega Context
    pub rh_omega: RHOmegaContext,
}

impl AppContext {
    /// Create a new instance of AppContext and return a smart pointer to it.
    ///
    /// Note: It is important at a later time to call set_gui, otherwise nothing will ever be
    /// drawn on the GUI.
    pub fn new() -> AppContextPointer {
        Rc::new(RefCell::new(AppContext {
            config: Config::default(),
            source_description: None,
            list: vec![],
            currently_displayed_index: 0,
            last_sample_pressure: None,
            gui: None,
            skew_t: SkewTContext::new(),
            rh_omega: RHOmegaContext::new(),
        }))
    }

    pub fn set_gui(&mut self, gui: Gui) {
        self.gui = Some(gui);
    }

    pub fn load_data(&mut self, src: &mut Iterator<Item = Sounding>) -> Result<()> {
        use app::config;
        use sounding_base::Profile::*;

        self.list = src.into_iter().collect();
        self.currently_displayed_index = 0;
        self.source_description = None;

        self.skew_t.xy_envelope = XYRect {
            lower_left: XYCoords { x: 0.45, y: 0.45 },
            upper_right: XYCoords { x: 0.55, y: 0.55 },
        };

        for snd in &self.list {
            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(Temperature))
                .filter_map(|p| if let (Some(p), Some(t)) =
                    (p.0.as_option(), p.1.as_option())
                {
                    if p < config::MINP {
                        None
                    } else {
                        Some(TPCoords {
                            temperature: t,
                            pressure: p,
                        })
                    }
                } else {
                    None
                })
            {
                let XYCoords { x, y } = SkewTContext::convert_tp_to_xy(pair);
                if x < self.skew_t.xy_envelope.lower_left.x {
                    self.skew_t.xy_envelope.lower_left.x = x;
                }
                if y < self.skew_t.xy_envelope.lower_left.y {
                    self.skew_t.xy_envelope.lower_left.y = y;
                }
                if x > self.skew_t.xy_envelope.upper_right.x {
                    self.skew_t.xy_envelope.upper_right.x = x;
                }
                if y > self.skew_t.xy_envelope.upper_right.y {
                    self.skew_t.xy_envelope.upper_right.y = y;
                }
            }

            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(DewPoint))
                .filter_map(|p| if let (Some(p), Some(t)) =
                    (p.0.as_option(), p.1.as_option())
                {
                    if p < config::MINP {
                        None
                    } else {
                        Some(TPCoords {
                            temperature: t,
                            pressure: p,
                        })
                    }
                } else {
                    None
                })
            {
                let XYCoords { x, y } = SkewTContext::convert_tp_to_xy(pair);
                if x < self.skew_t.xy_envelope.lower_left.x {
                    self.skew_t.xy_envelope.lower_left.x = x;
                }
                if y < self.skew_t.xy_envelope.lower_left.y {
                    self.skew_t.xy_envelope.lower_left.y = y;
                }
                if x > self.skew_t.xy_envelope.upper_right.x {
                    self.skew_t.xy_envelope.upper_right.x = x;
                }
                if y > self.skew_t.xy_envelope.upper_right.y {
                    self.skew_t.xy_envelope.upper_right.y = y;
                }
            }
        }

        self.rh_omega.max_abs_omega = config::MAX_ABS_W;
        for snd in &self.list {
            for abs_omega in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(PressureVerticalVelocity))
                .filter_map(|p| if let (Some(p), Some(o)) =
                    (p.0.as_option(), p.1.as_option())
                {
                    if p < config::MINP {
                        None
                    } else {
                        Some(o.abs())
                    }
                } else {
                    None
                })
            {
                if abs_omega > self.rh_omega.max_abs_omega {
                    self.rh_omega.max_abs_omega = abs_omega;
                }
            }
        }
        self.rh_omega.max_abs_omega = self.rh_omega.max_abs_omega.ceil();

        self.fit_to_data();

        if let Some(ref wdgs) = self.gui {
            wdgs.draw_all();
        }

        Ok(())
    }

    /// Is there any data to plot?
    pub fn plottable(&self) -> bool {
        !self.list.is_empty()
    }

    /// Set the next one as the one to display, or wrap to the beginning.
    pub fn display_next(&mut self) {
        if self.plottable() {
            if self.currently_displayed_index < self.list.len() - 1 {
                self.currently_displayed_index += 1;
            } else {
                self.currently_displayed_index = 0;
            }
        }

        if let Some(ref wdgs) = self.gui {
            wdgs.draw_all();
        }
    }

    /// Set the previous one as the one to display, or wrap to the end.
    pub fn display_previous(&mut self) {
        if self.plottable() {
            if self.currently_displayed_index > 0 {
                self.currently_displayed_index -= 1;
            } else {
                self.currently_displayed_index = self.list.len() - 1;
            }
        }

        if let Some(ref wdgs) = self.gui {
            wdgs.draw_all();
        }
    }

    /// Get the sounding to draw.
    pub fn get_sounding_for_display(&self) -> Option<&Sounding> {
        if self.plottable() {
            Some(&self.list[self.currently_displayed_index])
        } else {
            None
        }
    }

    /// Set the source name
    pub fn set_source_description(&mut self, new_name: Option<String>) {
        self.source_description = new_name;
    }

    /// Get the source name
    pub fn get_source_description(&self) -> Option<String> {
        match self.source_description {
            Some(ref name) => Some(name.clone()),
            None => None,
        }
    }

    /// Get the screen resolution in dpi
    pub fn get_dpi(&self) -> Option<f64> {
        use gtk::WidgetExt;
        use gdk::ScreenExt;

        match self.gui {
            None => None,
            Some(ref gui) => {
                match gui.get_sounding_area().get_screen() {
                    None => None,
                    Some(ref screen) => Some(screen.get_resolution()),
                }
            }
        }
    }

    /// Fit to the given x-y max coords. SHOULD NOT BE PUBLIC - DO NOT USE IN DRAWING CALLBACKS.
    fn fit_to_data(&mut self) {

        use std::f64;

        let lower_left = self.skew_t.xy_envelope.lower_left;
        self.set_skew_t_translation(lower_left);

        let width = self.skew_t.xy_envelope.upper_right.x - self.skew_t.xy_envelope.lower_left.x;
        let height = self.skew_t.xy_envelope.upper_right.y - self.skew_t.xy_envelope.lower_left.y;

        let width_scale = 1.0 / width;
        let height_scale = 1.0 / height;

        self.set_zoom_factor(f64::min(width_scale, height_scale));

        self.bound_view();
    }

    /// Update the dimensions of the skew-t drawing area
    pub fn update_plot_context_allocations(&mut self) {
        if let Some(ref gui) = self.gui {

            let alloc = gui.get_sounding_area().get_allocation();
            self.skew_t.device_width = alloc.width;
            self.skew_t.device_height = alloc.height;

            let alloc = gui.get_omega_area().get_allocation();
            self.rh_omega.device_width = alloc.width;
            self.rh_omega.device_height = alloc.height;
        }
    }

    /// Right justify the skew-t in the view if zoomed out, and if zoomed in don't let it view
    /// beyond the edges of the skew-t.
    pub fn bound_view(&mut self) {

        let bounds = DeviceCoords {
            col: f64::from(self.skew_t.device_width),
            row: f64::from(self.skew_t.device_height),
        };
        let lower_right = self.skew_t.convert_device_to_xy(bounds);
        let upper_left = self.skew_t.convert_device_to_xy(
            DeviceCoords { col: 0.0, row: 0.0 },
        );
        let width = lower_right.x - upper_left.x;
        let height = upper_left.y - lower_right.y;

        if width <= 1.0 {
            if self.skew_t.translate.x < 0.0 {
                self.skew_t.translate.x = 0.0;
            }
            let max_x = 1.0 - width;
            if self.skew_t.translate.x > max_x {
                self.skew_t.translate.x = max_x;
            }
        } else {
            self.skew_t.translate.x = 0.0;
        }
        if height < 1.0 {
            if self.skew_t.translate.y < 0.0 {
                self.skew_t.translate.y = 0.0;
            }
            let max_y = 1.0 - height;
            if self.skew_t.translate.y > max_y {
                self.skew_t.translate.y = max_y;
            }
        } else {
            self.skew_t.translate.y = -(height - 1.0) / 2.0;
        }
        self.rh_omega.translate_y = self.skew_t.translate.y;
    }

    /// Get the zoom factor
    pub fn get_zoom_factor(&self) -> f64 {
        self.skew_t.zoom_factor
    }

    /// Set the zoom factor
    pub fn set_zoom_factor(&mut self, new_zoom: f64) {
        self.skew_t.zoom_factor = new_zoom;
        self.rh_omega.zoom_factor = new_zoom;
    }

    /// Get the translation needed to draw correctly for panning and zooming the skew_t
    pub fn get_skew_t_translation(&self) -> XYCoords {
        self.skew_t.translate
    }

    /// Set the translation needed to draw correctly for panning and zooming the skew_t
    pub fn set_skew_t_translation(&mut self, translate: XYCoords) {
        self.skew_t.translate = translate;
        self.rh_omega.translate_y = translate.y;
    }

    pub fn queue_draw_skew_t_rh_omega(&self) {

        if let Some(ref gui) = self.gui {
            gui.get_sounding_area().queue_draw();
            gui.get_omega_area().queue_draw();
        }
    }

    pub fn show_hide_rh_omega(&self) {
        if let Some(ref gui) = self.gui {
            if self.config.show_rh_omega_frame {
                gui.get_omega_area().show();
            } else {
                gui.get_omega_area().hide();
            }
        }
    }
}

pub trait PlotContext {
    /// Conversion from (x,y) coords to screen coords
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords;

    /// Conversion from device coordinates to `ScreenCoords`
    fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords;

    /// Get the device width
    fn device_width(&self) -> i32;

    /// Get device height
    fn device_height(&self) -> i32;

    /// Get the edges of the X-Y plot area in `ScreenCoords`, may or may not be on the screen.
    fn calculate_plot_edges(&self, cr: &Context, ac: &AppContext) -> ScreenRect {

        let ScreenRect {
            lower_left,
            upper_right,
        } = self.bounding_box_in_screen_coords();
        let ScreenCoords {
            x: mut screen_x_min,
            y: mut screen_y_min,
        } = lower_left;
        let ScreenCoords {
            x: mut screen_x_max,
            y: mut screen_y_max,
        } = upper_right;

        // If screen area is bigger than plot area, labels will be clipped, keep them on the plot
        let ScreenCoords { x: xmin, y: ymin } =
            self.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
        let ScreenCoords { x: xmax, y: ymax } =
            self.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });

        if xmin > screen_x_min {
            screen_x_min = xmin;
        }
        if xmax < screen_x_max {
            screen_x_max = xmax;
        }
        if ymax < screen_y_max {
            screen_y_max = ymax;
        }
        if ymin > screen_y_min {
            screen_y_min = ymin;
        }

        // Add some padding to keep away from the window edge
        let padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
        screen_x_max -= padding;
        screen_y_max -= padding;
        screen_x_min += padding;
        screen_y_min += padding;

        ScreenRect {
            lower_left: ScreenCoords {
                x: screen_x_min,
                y: screen_y_min,
            },
            upper_right: ScreenCoords {
                x: screen_x_max,
                y: screen_y_max,
            },
        }
    }

    /// Get a bounding box in screen coords
    fn bounding_box_in_screen_coords(&self) -> ScreenRect {
        let lower_left = self.convert_device_to_screen(DeviceCoords {
            col: 0.0,
            row: f64::from(self.device_height()),
        });
        let upper_right = self.convert_device_to_screen(DeviceCoords {
            col: f64::from(self.device_width()),
            row: 0.0,
        });

        ScreenRect {
            lower_left,
            upper_right,
        }
    }
}

pub struct SkewTContext {
    // Rectangle that bounds all the soundings in the list.
    xy_envelope: XYRect,

    // Standard x-y coords, used for zooming and panning.
    zoom_factor: f64, // Multiply by this after translating
    translate: XYCoords,

    // device dimensions
    pub device_height: i32,
    pub device_width: i32,

    // state of input for left button press and panning.
    pub left_button_pressed: bool,

    // last cursor position in skew_t widget, used for sampling and panning
    pub last_cursor_position_skew_t: Option<DeviceCoords>,

    // Distance used for adding padding around labels in `ScreenCoords`
    pub label_padding: f64,
    // Distance using for keeping things too close to the edge of the window in `ScreenCoords`
    pub edge_padding: f64,
}

impl SkewTContext {
    pub fn new() -> Self {
        SkewTContext {
            xy_envelope: XYRect {
                lower_left: XYCoords { x: 0.0, y: 0.0 },
                upper_right: XYCoords { x: 1.0, y: 1.0 },
            },

            // Sounding Area GUI state
            zoom_factor: 1.0,
            translate: XYCoords::origin(),
            device_height: 100,
            device_width: 100,
            last_cursor_position_skew_t: None,
            left_button_pressed: false,

            // Drawing cache
            edge_padding: 0.0,
            label_padding: 0.0,
        }
    }

    /// This is the scale factor that will be set for the cairo transform matrix.
    ///
    /// By using this scale factor, it makes a distance of 1 in `XYCoords` equal to a distance of
    /// 1 in `ScreenCoords` when the zoom factor is 1.
    pub fn scale_factor(&self) -> f64 {
        f64::from(::std::cmp::min(self.device_height, self.device_width))
    }

    /// Conversion from temperature (t) and pressure (p) to (x,y) coords
    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use app::config;
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.pressure)) /
            (f64::log10(config::MAXP) - f64::log10(config::MINP));

        let x = (coords.temperature - config::MINT) / (config::MAXT - config::MINT);

        // do the skew
        let x = x + y;
        XYCoords { x, y }
    }

    /// Convert device coords to (x,y) coords
    pub fn convert_device_to_xy(&self, coords: DeviceCoords) -> XYCoords {
        let screen_coords = self.convert_device_to_screen(coords);
        self.convert_screen_to_xy(screen_coords)
    }

    /// Conversion from  (x,y) coords to temperature and pressure.
    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        use app::config;
        use std::f64;

        // undo the skew
        let x = coords.x - coords.y;
        let y = coords.y;

        let t = x * (config::MAXT - config::MINT) + config::MINT;
        let p = 10.0f64.powf(
            f64::log10(config::MAXP) -
                y * (f64::log10(config::MAXP) - f64::log10(config::MINP)),
        );

        TPCoords {
            temperature: t,
            pressure: p,
        }
    }

    /// Conversion from (x,y) coords to screen coords
    pub fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Screen coords go 0 -> 1 down the y axis and 0 -> aspect_ratio right along the x axis.

        let x = coords.x / self.zoom_factor + self.translate.x;
        let y = coords.y / self.zoom_factor + self.translate.y;
        XYCoords { x, y }
    }

    /// Conversion from temperature/pressure to screen coordinates.
    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = Self::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    /// Conversion from screen coordinates to temperature, pressure.
    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        let xy = self.convert_screen_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }

    /// Convert from device coords to temperature, pressure.
    pub fn convert_device_to_tp(&self, coords: DeviceCoords) -> TPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }
}

impl PlotContext for SkewTContext {
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {

        // Apply translation first
        let x = coords.x - self.translate.x;
        let y = coords.y - self.translate.y;

        // Apply scaling
        let x = self.zoom_factor * x;
        let y = self.zoom_factor * y;
        ScreenCoords { x, y }
    }

    fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords {
        let scale_factor = self.scale_factor();
        ScreenCoords {
            x: coords.col / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            y: -(coords.row / scale_factor) + f64::from(self.device_height) / scale_factor,
        }
    }

    fn device_height(&self) -> i32 {
        self.device_height
    }

    fn device_width(&self) -> i32 {
        self.device_width
    }
}

pub struct RHOmegaContext {
    // Bound for the omega plot
    max_abs_omega: f64,

    // Translate for zoom and pan in skew-t
    translate_y: f64,
    zoom_factor: f64,
    pub skew_t_scale_factor: f64,

    // device dimensions
    pub device_height: i32,
    pub device_width: i32,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            max_abs_omega: config::MAX_ABS_W,
            translate_y: 0.0,
            zoom_factor: 1.0,
            skew_t_scale_factor: 1.0,

            device_height: 100,
            device_width: 100,
        }
    }

    /// This is the scale factor that will be set for the cairo transform matrix.
    ///
    /// By using this scale factor, it makes a distance of 1 in `XYCoords` equal to a distance of
    /// 1 in `ScreenCoords` when the zoom factor is 1.
    pub fn scale_factor(&self) -> f64 {
        f64::from(::std::cmp::min(self.device_height, self.device_width))
    }

    /// Conversion from omega (w) and pressure (p) to (x,y) coords
    pub fn convert_wp_to_xy(&self, coords: WPCoords) -> XYCoords {
        use app::config;
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.p)) /
            (f64::log10(config::MAXP) - f64::log10(config::MINP));

        // The + sign below looks weird, but is correct.
        let x = (coords.w + self.max_abs_omega) / (2.0 * self.max_abs_omega);

        XYCoords { x, y }
    }

    /// Conversion from `XYCoords` to `WPCoords`
    pub fn convert_xy_to_wp(&self, coords: XYCoords) -> WPCoords {
        use app::config;
        use std::f64;

        let p = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP)) +
                f64::log10(config::MAXP),
        );
        let w = coords.x * (2.0 * self.max_abs_omega) - self.max_abs_omega;

        WPCoords { w, p }
    }

    /// Conversion from screen coords to xy
    pub fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x * self.skew_t_scale_factor / self.scale_factor();
        let y = coords.y / self.zoom_factor;

        // Unapply translation
        let x = x;
        let y = y + self.translate_y;

        XYCoords { x, y }
    }

    /// Converstion from screen to `WPCoords`
    pub fn convert_screen_to_wp(&self, coords: ScreenCoords) -> WPCoords {
        let xy = self.convert_screen_to_xy(coords);
        self.convert_xy_to_wp(xy)
    }

    /// Conversion from `DeviceCoords` to `WPCoords`
    pub fn convert_device_to_wp(&self, coords: DeviceCoords) -> WPCoords {
        let screen = self.convert_device_to_screen(coords);
        self.convert_screen_to_wp(screen)
    }

    /// Conversion from omega/pressure to screen coordinates.
    pub fn convert_wp_to_screen(&self, coords: WPCoords) -> ScreenCoords {
        let xy = self.convert_wp_to_xy(coords);

        self.convert_xy_to_screen(xy)
    }

    /// Get maximum absolute omega
    pub fn get_max_abs_omega(&self) -> f64 {
        self.max_abs_omega
    }
}

impl PlotContext for RHOmegaContext {
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {

        // Apply translation first
        let x = coords.x;
        let y = coords.y - self.translate_y;

        // Apply scaling
        let x = x / self.skew_t_scale_factor * self.scale_factor();
        let y = self.zoom_factor * y;
        ScreenCoords { x, y }
    }

    /// Conversion from device to screen coordinates.
    fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords {
        let scale_factor = self.skew_t_scale_factor;
        ScreenCoords {
            x: coords.col / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            y: -(coords.row / scale_factor) + f64::from(self.device_height) / scale_factor,
        }
    }

    fn device_height(&self) -> i32 {
        self.device_height
    }

    fn device_width(&self) -> i32 {
        self.device_width
    }
}

#[test]
fn test_coord_conversion_rh_omega() {
    let context = RHOmegaContext::new();

    let screen1 = ScreenCoords { x: 0.5, y: 0.5 };
    let screen2 = context.convert_xy_to_screen(context.convert_screen_to_xy(screen1));

    assert!(::approx_equal(screen1.x, screen2.x, 1.0e-9));
    assert!(::approx_equal(screen1.y, screen2.y, 1.0e-9));

    let wp1 = WPCoords { w: 0.0, p: 505.0 };
    let wp2 = context.convert_screen_to_wp(context.convert_wp_to_screen(wp1));

    assert!(::approx_equal(wp1.p, wp2.p, 1.0e-9));
    assert!(::approx_equal(wp1.p, wp2.p, 1.0e-9));
}
