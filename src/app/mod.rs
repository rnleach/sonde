//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.

use std::rc::Rc;
use std::cell::RefCell;

use sounding_base::Sounding;

use errors::*;
use gui::sonde_widgets::SondeWidgets;
use gui::sounding::{DeviceCoords, ScreenCoords, TPCoords, XYCoords};

/// Smart pointer for globally shareable data
pub type AppContextPointer = Rc<RefCell<AppContext>>;

/// Holds the application state. This is a singleton (not enforced) that is shared globally.
pub struct AppContext {
    list: Vec<Sounding>,
    // Lower left and  upper right corners of the bounding box that bounds all the soundings in
    // the list.
    lower_left: XYCoords,
    upper_right: XYCoords,

    currently_displayed: usize,
    widgets: Option<SondeWidgets>,

    // Standard x-y coords
    pub zoom_factor: f32, // Multiply by this after translating
    pub translate_x: f32, // subtract this from x before converting to screen coords.
    pub translate_y: f32, // subtract this from y before converting to screen coords.

    // device dimensions
    pub device_height: i32,
    pub device_width: i32,

    // state of input for left button press and panning.
    pub left_button_press_start: DeviceCoords,
    pub left_button_pressed: bool,
}

impl AppContext {
    /// Create a new instance of AppContext and return a smart pointer to it.
    ///
    /// Note: It is important at a later time to call set_widgets, otherwise nothing will ever be
    /// drawn on the GUI.
    pub fn new() -> AppContextPointer {
        Rc::new(RefCell::new(AppContext {
            // Data state
            list: vec![],
            lower_left: (0.0, 0.0),
            upper_right: (1.0, 1.0),
            currently_displayed: 0,
            widgets: None,

            // Sounding Area GUI state
            zoom_factor: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            device_height: 100,
            device_width: 100,
            left_button_press_start: (0.0, 0.0),
            left_button_pressed: false,
        }))
    }

    pub fn set_widgets(&mut self, widgets: SondeWidgets) {
        self.widgets = Some(widgets);
    }

    pub fn load_data(&mut self, src: &mut Iterator<Item = Sounding>) -> Result<()> {
        use config;

        self.list = src.into_iter().collect();
        self.currently_displayed = 0;

        self.lower_left = (0.45, 0.45);
        self.upper_right = (0.55, 0.55);

        for snd in &self.list {
            for pair in snd.pressure.iter().zip(&snd.temperature).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
                    if p < config::MINP { None } else { Some((t, p)) }
                } else {
                    None
                }
            })
            {
                let (x, y) = Self::convert_tp_to_xy(pair);
                if x < self.lower_left.0 {
                    self.lower_left.0 = x;
                }
                if y < self.lower_left.1 {
                    self.lower_left.1 = y;
                }
                if x > self.upper_right.0 {
                    self.upper_right.0 = x;
                }
                if y > self.upper_right.1 {
                    self.upper_right.1 = y;
                }
            }

            // TODO: Once you get surface temperature, make a station pressure-surface temp
            // point and throw that in the mix.
            // if let Some(p) = snd.station_pres.as_option() {

            // }

            for pair in snd.pressure.iter().zip(&snd.dew_point).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
                    if p < config::MINP { None } else { Some((t, p)) }
                } else {
                    None
                }
            })
            {
                let (x, y) = Self::convert_tp_to_xy(pair);
                if x < self.lower_left.0 {
                    self.lower_left.0 = x;
                }
                if y < self.lower_left.1 {
                    self.lower_left.1 = y;
                }
                if x > self.upper_right.0 {
                    self.upper_right.0 = x;
                }
                if y > self.upper_right.1 {
                    self.upper_right.1 = y;
                }
            }
        }

        if let Some(ref wdgs) = self.widgets {
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
            if self.currently_displayed < self.list.len() - 1 {
                self.currently_displayed += 1;
            } else {
                self.currently_displayed = 0;
            }
        }

        if let Some(ref wdgs) = self.widgets {
            wdgs.draw_all();
        }
    }

    /// Set the previous one as the one to display, or wrap to the end.
    pub fn display_previous(&mut self) {
        if self.plottable() {
            if self.currently_displayed > 0 {
                self.currently_displayed -= 1;
            } else {
                self.currently_displayed = self.list.len() - 1;
            }
        }

        if let Some(ref wdgs) = self.widgets {
            wdgs.draw_all();
        }
    }

    /// Get the sounding to draw.
    pub fn get_sounding_for_display(&self) -> Option<&Sounding> {
        if self.plottable() {
            Some(&self.list[self.currently_displayed])
        } else {
            None
        }
    }

    /// A scale factor to use when converting from XY to Screen Coordinates.
    /// NOT the same as the zoom factor.
    #[inline]
    pub fn scale_factor(&self) -> f64 {
        ::std::cmp::min(self.device_height, self.device_width) as f64
    }

    /// Conversion from temperature (t) and pressure (p) to (x,y) coords
    #[inline]
    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use config;
        use std::f32;

        let y = (f32::log10(config::MAXP) - f32::log10(coords.1)) /
            (f32::log10(config::MAXP) - f32::log10(config::MINP));

        let x = (coords.0 - config::MINT) / (config::MAXT - config::MINT);

        // do the skew
        let x = x + y;
        (x, y)
    }

    /// Convert device to screen coords
    #[inline]
    pub fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords {
        let scale_factor = self.scale_factor();
        (
            coords.0 / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            -(coords.1 / scale_factor) + self.device_height as f64 / scale_factor,
        )
    }

    /// Convert device coords to (x,y) coords
    #[inline]
    pub fn convert_device_to_xy(&self, coords: DeviceCoords) -> XYCoords {
        let screen_coords = self.convert_device_to_screen(coords);
        self.convert_screen_to_xy(screen_coords)
    }

    /// Conversion from  (x,y) coords to temperature and pressure.
    #[inline]
    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        use config;
        use std::f32;

        // undo the skew
        let x = coords.0 - coords.1;
        let y = coords.1;

        let t = x * (config::MAXT - config::MINT) + config::MINT;
        let p = 10.0f32.powf(
            f32::log10(config::MAXP) -
                y * (f32::log10(config::MAXP) - f32::log10(config::MINP)),
        );

        (t, p)
    }

    /// Conversion from (x,y) coords to screen coords
    #[inline]
    pub fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {

        // Apply translation first
        let x = coords.0 - self.translate_x;
        let y = coords.1 - self.translate_y;

        // Apply scaling
        let x = (self.zoom_factor * x) as f64;
        let y = (self.zoom_factor * y) as f64;
        (x, y)
    }

    /// Conversion from (x,y) coords to screen coords
    #[inline]
    pub fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Screen coords go 0 -> 1 down the y axis and 0 -> aspect_ratio right along the x axis.

        let x = coords.0 as f32 / self.zoom_factor + self.translate_x;
        let y = coords.1 as f32 / self.zoom_factor + self.translate_y;
        (x, y)
    }

    /// Conversion from temperature/pressure to screen coordinates.
    #[inline]
    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = Self::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    /// Conversion from screen coordinates to temperature, pressure.
    #[inline]
    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        let xy = self.convert_screen_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }

    /// Fit to the given x-y max coords.
    #[inline]
    pub fn fit_to_data(&mut self) {
        use std::f32;

        self.translate_x = self.lower_left.0;
        self.translate_y = self.lower_left.1;

        let width = self.upper_right.0 - self.lower_left.0;
        let height = self.upper_right.1 - self.lower_left.1;

        let width_scale = 1.0 / width;
        let height_scale = 1.0 / height;

        // println!("lower_left: {:?}, upper_right: {:?}, width_scale: {}, height_scale: {}",
        // lower_left, upper_right, width_scale, height_scale);

        self.zoom_factor = f32::min(width_scale, height_scale);

        self.bound_view();
    }

    /// Center the skew-t in the view if zoomed out, and if zoomed in don't let it view beyond the
    /// edges of the skew-t.
    pub fn bound_view(&mut self) {

        let bounds = (self.device_width as f64, self.device_height as f64);
        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy((0.0, 0.0));
        let width = lower_right.0 - upper_left.0;
        let height = upper_left.1 - lower_right.1;

        if width <= 1.0 {
            if self.translate_x < 0.0 {
                self.translate_x = 0.0;
            }
            let max_x = 1.0 - width;
            if self.translate_x > max_x {
                self.translate_x = max_x;
            }
        } else {
            self.translate_x = -(width - 1.0) / 2.0;
        }
        if height < 1.0 {
            if self.translate_y < 0.0 {
                self.translate_y = 0.0;
            }
            let max_y = 1.0 - height;
            if self.translate_y > max_y {
                self.translate_y = max_y;
            }
        } else {
            self.translate_y = -(height - 1.0) / 2.0;
        }
    }
}
