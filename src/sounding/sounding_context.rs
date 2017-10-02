//! Sounding context to store `sounding_area` state between calls.

use std::cell::RefCell;
use std::rc::Rc;
use super::{DeviceCoords, ScreenCoords, TPCoords, XYCoords};

/// Smart pointer so that this type can be easily shared as global state.
pub type SoundingContextPointer = Rc<RefCell<SoundingContext>>;

/// Stores state of the sounding view between function, method, and callback calls.
pub struct SoundingContext {
    // Standard x-y coords
    pub zoom_factor: f32, // Multiply by this after translating
    pub translate_x: f32, // subtract this from x before converting to screen coords.
    pub translate_y: f32, // subtract this from y before converting to screen coords.
    pub device_height: i32,
    pub device_width: i32,
}

impl SoundingContext {
    // Constants for defining a standard x-y coordinate system
    /// Maximum pressure plotted on skew-t (bottom edge)
    pub const MAXP: f32 = 1050.0; // mb
    /// Minimum pressure plotted on skew-t (top edge)
    pub const MINP: f32 = 90.0; // mb
    /// Coldest temperature plotted at max pressure, on the bottom edge.
    pub const MINT: f32 = -46.5; // C - at MAXP
    /// Warmest temperature plotted at max pressure, on the bottom edge.
    pub const MAXT: f32 = 50.5; // C - at MAXP

    /// Used during program initialization to create the SoundingContext and smart pointer.
    pub fn create_sounding_context() -> SoundingContextPointer {
        Rc::new(RefCell::new(SoundingContext {
            zoom_factor: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            device_height: 100,
            device_width: 100,
        }))
    }

    /// Conversion from temperature (t) and pressure (p) to (x,y) coords
    #[inline]
    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use std::f32;

        let y = (f32::log10(SoundingContext::MAXP) - f32::log10(coords.1)) /
            (f32::log10(SoundingContext::MAXP) - f32::log10(SoundingContext::MINP));
        // let y = f32::log10(-(coords.1 - SoundingContext::MAXP) + 0.00001) /
        //     f32::log10(-(SoundingContext::MINP - SoundingContext::MAXP) + 0.00001);
        let x = (coords.0 - SoundingContext::MINT) /
            (SoundingContext::MAXT - SoundingContext::MINT);
        // do the skew
        let x = x + y;
        (x, y)
    }

    /// Convert device coords to (x,y) coords
    #[inline]
    pub fn convert_device_to_xy(&self, coords: DeviceCoords) -> XYCoords {
        let screen_coords = (
            coords.0 / self.device_height as f64,
            -(coords.1 / self.device_height as f64) + 1.0,
        );

        self.convert_screen_to_xy(screen_coords)
    }

    /// TODO: implement Conversion from  (x,y) coords to temperature and pressure.
    #[inline]
    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        unimplemented!();
    }

    /// Conversion from (x,y) coords to screen coords
    #[inline]
    pub fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        // Screen coords go 0 -> 1 up the y axis and 0 -> aspect_ratio right along the x axis.

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
        let xy = SoundingContext::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    /// TODO: implement Conversion from screen coordinates to temperature, pressure.
    #[inline]
    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        unimplemented!();
    }

    /// TODO: implement Fit to the given x-y max coords.
    #[inline]
    pub fn fit_to(&mut self, lower_left: XYCoords, upper_right: XYCoords) {
        unimplemented!();
    }
}
