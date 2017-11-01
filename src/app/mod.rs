//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.

use std::rc::Rc;
use std::cell::RefCell;

use sounding_base::Sounding;

use errors::*;
use gui::Gui;
use coords::{DeviceCoords, ScreenCoords, TPCoords, XYCoords, XYRect, ScreenRect};
use config::Config;

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

    // Rectangle that bounds all the soundings in the list.
    xy_envelope: XYRect,

    // Handle to the GUI
    gui: Option<Gui>,

    // Standard x-y coords, used for zooming and panning.
    pub zoom_factor: f64, // Multiply by this after translating
    pub translate: XYCoords,

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
            xy_envelope: XYRect {
                lower_left: XYCoords { x: 0.0, y: 0.0 },
                upper_right: XYCoords { x: 1.0, y: 1.0 },
            },
            currently_displayed_index: 0,
            gui: None,

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
        }))
    }

    pub fn set_gui(&mut self, gui: Gui) {
        self.gui = Some(gui);
    }

    pub fn load_data(&mut self, src: &mut Iterator<Item = Sounding>) -> Result<()> {
        use config;

        self.list = src.into_iter().collect();
        self.currently_displayed_index = 0;
        self.source_description = None;

        self.xy_envelope = XYRect {
            lower_left: XYCoords { x: 0.45, y: 0.45 },
            upper_right: XYCoords { x: 0.55, y: 0.55 },
        };

        for snd in &self.list {
            for pair in snd.pressure.iter().zip(&snd.temperature).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
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
                }
            })
            {
                let XYCoords { x, y } = Self::convert_tp_to_xy(pair);
                if x < self.xy_envelope.lower_left.x {
                    self.xy_envelope.lower_left.x = x;
                }
                if y < self.xy_envelope.lower_left.y {
                    self.xy_envelope.lower_left.y = y;
                }
                if x > self.xy_envelope.upper_right.x {
                    self.xy_envelope.upper_right.x = x;
                }
                if y > self.xy_envelope.upper_right.y {
                    self.xy_envelope.upper_right.y = y;
                }
            }

            for pair in snd.pressure.iter().zip(&snd.dew_point).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
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
                }
            })
            {
                let XYCoords { x, y } = Self::convert_tp_to_xy(pair);
                if x < self.xy_envelope.lower_left.x {
                    self.xy_envelope.lower_left.x = x;
                }
                if y < self.xy_envelope.lower_left.y {
                    self.xy_envelope.lower_left.y = y;
                }
                if x > self.xy_envelope.upper_right.x {
                    self.xy_envelope.upper_right.x = x;
                }
                if y > self.xy_envelope.upper_right.y {
                    self.xy_envelope.upper_right.y = y;
                }
            }
        }

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

    /// This is the scale factor that will be set for the cairo transform matrix.
    ///
    /// By using this scale factor, it makes a distance of 1 in `XYCoords` equal to a distance of
    /// 1 in `ScreenCoords` when the zoom factor is 1.
    pub fn scale_factor(&self) -> f64 {
        ::std::cmp::min(self.device_height, self.device_width) as f64
    }

    /// Conversion from temperature (t) and pressure (p) to (x,y) coords
    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use config;
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.pressure)) /
            (f64::log10(config::MAXP) - f64::log10(config::MINP));

        let x = (coords.temperature - config::MINT) / (config::MAXT - config::MINT);

        // do the skew
        let x = x + y;
        XYCoords { x, y }
    }

    /// Convert device to screen coords
    pub fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords {
        let scale_factor = self.scale_factor();
        ScreenCoords {
            x: coords.col / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            y: -(coords.row / scale_factor) + self.device_height as f64 / scale_factor,
        }
    }

    /// Convert device coords to (x,y) coords
    pub fn convert_device_to_xy(&self, coords: DeviceCoords) -> XYCoords {
        let screen_coords = self.convert_device_to_screen(coords);
        self.convert_screen_to_xy(screen_coords)
    }

    /// Conversion from  (x,y) coords to temperature and pressure.
    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        use config;
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
    pub fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {

        // Apply translation first
        let x = coords.x - self.translate.x;
        let y = coords.y - self.translate.y;

        // Apply scaling
        let x = (self.zoom_factor * x) as f64;
        let y = (self.zoom_factor * y) as f64;
        ScreenCoords { x, y }
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

    /// Get a bounding box in screen coords
    pub fn bounding_box_in_screen_coords(&self) -> ScreenRect {
        let lower_left = self.convert_device_to_screen(DeviceCoords {
            col: 0.0,
            row: self.device_height as f64,
        });
        let upper_right = self.convert_device_to_screen(DeviceCoords {
            col: self.device_width as f64,
            row: 0.0,
        });

        ScreenRect {
            lower_left,
            upper_right,
        }
    }

    /// Fit to the given x-y max coords.
    pub fn fit_to_data(&mut self) {

        use std::f64;

        self.translate.x = self.xy_envelope.lower_left.x;
        self.translate.y = self.xy_envelope.lower_left.y;

        let width = self.xy_envelope.upper_right.x - self.xy_envelope.lower_left.x;
        let height = self.xy_envelope.upper_right.y - self.xy_envelope.lower_left.y;

        let width_scale = 1.0 / width;
        let height_scale = 1.0 / height;

        self.zoom_factor = f64::min(width_scale, height_scale);

        self.bound_view();
    }

    /// Center the skew-t in the view if zoomed out, and if zoomed in don't let it view beyond the
    /// edges of the skew-t.
    pub fn bound_view(&mut self) {

        let bounds = DeviceCoords {
            col: self.device_width as f64,
            row: self.device_height as f64,
        };
        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy(DeviceCoords { col: 0.0, row: 0.0 });
        let width = lower_right.x - upper_left.x;
        let height = upper_left.y - lower_right.y;

        if width <= 1.0 {
            if self.translate.x < 0.0 {
                self.translate.x = 0.0;
            }
            let max_x = 1.0 - width;
            if self.translate.x > max_x {
                self.translate.x = max_x;
            }
        } else {
            self.translate.x = -(width - 1.0) / 2.0;
        }
        if height < 1.0 {
            if self.translate.y < 0.0 {
                self.translate.y = 0.0;
            }
            let max_y = 1.0 - height;
            if self.translate.y > max_y {
                self.translate.y = max_y;
            }
        } else {
            self.translate.y = -(height - 1.0) / 2.0;
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
}
