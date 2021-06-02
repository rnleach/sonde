//! Coordinate systems and geometry definitions. Some conversions are dependent on the application
//! state, and so those functions are a part of the `AppContext`.

use crate::app::config;
use metfor::{Celsius, HectoPascal, Knots, Meters, PaPS, WindSpdDir};

/// Common operations on rectangles
pub trait Rect {
    /// Get the minimum x coordinate
    fn min_x(&self) -> f64;
    /// Get the maximum x coordinate
    fn max_x(&self) -> f64;
    /// Get the minimum y coordinate
    fn min_y(&self) -> f64;
    /// Get the maximum y coordinate
    fn max_y(&self) -> f64;

    /// Check if two rectangles overlap
    fn overlaps(&self, other: &Self) -> bool {
        if self.min_x() > other.max_x() {
            return false;
        }
        if self.max_x() < other.min_x() {
            return false;
        }
        if self.min_y() > other.max_y() {
            return false;
        }
        if self.max_y() < other.min_y() {
            return false;
        }

        true
    }

    // Check if this rectangle is inside another.
    fn inside(&self, big_rect: &Self) -> bool {
        if self.min_x() < big_rect.min_x() {
            return false;
        }
        if self.max_x() > big_rect.max_x() {
            return false;
        }
        if self.min_y() < big_rect.min_y() {
            return false;
        }
        if self.max_y() > big_rect.max_y() {
            return false;
        }

        true
    }

    /// Get the width of this rectangle
    fn width(&self) -> f64 {
        self.max_x() - self.min_x()
    }

    /// Get the height of this rectangle
    fn height(&self) -> f64 {
        self.max_y() - self.min_y()
    }
}

/***************************************************************************************************
 *                   Temperature - Pressure Coordinates for Skew-T Log-P plot.
 * ************************************************************************************************/
/// Temperature-Pressure coordinates.
/// Origin lower left. (Temperature, Pressure)
#[derive(Clone, Copy, Debug)]
pub struct TPCoords {
    /// Temperature in Celsius
    pub temperature: Celsius,
    /// Pressure in hPa
    pub pressure: HectoPascal,
}

/***************************************************************************************************
 *                      Speed - Direction Coordinates for the Hodograph
 * ************************************************************************************************/
/// Speed-Direction coordinates for the hodograph.
/// Origin center. (Speed, Direction wind is from)
#[derive(Clone, Copy, Debug)]
pub struct SDCoords {
    /// Wind speed and direction.
    pub spd_dir: WindSpdDir<Knots>,
}

/***************************************************************************************************
 *                   Omega(W) - Pressure coords for the vertical velocity and RH plot
 * ************************************************************************************************/
/// Omega-Pressure coordinates.
/// Origin lower left. (Omega, Pressure)
#[derive(Clone, Copy, Debug)]
pub struct WPCoords {
    /// Omega in Pa/s
    pub w: PaPS,
    /// Pressure in hPa
    pub p: HectoPascal,
}

/***************************************************************************************************
 *                   Percent - Pressure coords for the Cloud Cover
 * ************************************************************************************************/
/// Percent-Pressure coordinates.
#[derive(Clone, Copy, Debug)]
pub struct PPCoords {
    /// Percent 0.0 - 1.0
    pub pcnt: f64,
    /// Pressure in hPa
    pub press: HectoPascal,
}

/***************************************************************************************************
 *                   Speed - Pressure coords for the wind speed profile
 * ************************************************************************************************/
/// Speed-Pressure coordinates.
#[derive(Clone, Copy, Debug)]
pub struct SPCoords {
    /// Speed in knots
    pub spd: Knots,
    /// Pressure in hPa
    pub press: HectoPascal,
}

/***************************************************************************************************
 *                   △T - Height coords for the fire plume chart
 * ************************************************************************************************/
/// FirePower-Height coordinates.
#[derive(Clone, Copy, Debug)]
pub struct FirePowerHCoords {
    /// Fire Power in Giga Watts
    pub fp: f64,
    /// Height in meters
    pub height: Meters,
}

/***************************************************************************************************
 *                   △T - Percent coords for the percent wet cape chart.
 * ************************************************************************************************/
/// FirePower-Percent coordinates.
#[derive(Clone, Copy, Debug)]
pub struct FirePowerPCoords {
    /// Fire Power in Giga Watts
    pub fp: f64,
    /// Height in meters
    pub percent: f64,
}

/***************************************************************************************************
 *                 X - Y Coords for a default plot area that can be zoomed and panned
 * ************************************************************************************************/

/// XY coordinates of the skew-t graph, range 0.0 to 1.0. This coordinate system is dependent on
/// settings for the maximum/minimum plottable pressure and temperatures in the config module.
/// Origin lower left, (x,y)
#[derive(Clone, Copy, Debug)]
pub struct XYCoords {
    pub x: f64,
    pub y: f64,
}

impl XYCoords {
    pub fn origin() -> Self {
        XYCoords { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct XYRect {
    pub lower_left: XYCoords,
    pub upper_right: XYCoords,
}

impl Rect for XYRect {
    fn min_x(&self) -> f64 {
        self.lower_left.x
    }

    fn max_x(&self) -> f64 {
        self.upper_right.x
    }

    fn min_y(&self) -> f64 {
        self.lower_left.y
    }

    fn max_y(&self) -> f64 {
        self.upper_right.y
    }
}

/***************************************************************************************************
 *                   Screen Coords - the coordinate system to actually draw in.
 * ************************************************************************************************/
/// On screen coordinates. Meant to scale and translate `XYCoords` to fit on the screen.
/// Origin lower left, (x,y).
/// When drawing using cairo functions, use these coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ScreenCoords {
    pub x: f64,
    pub y: f64,
}

impl ScreenCoords {
    pub fn origin() -> Self {
        ScreenCoords { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScreenRect {
    pub lower_left: ScreenCoords,
    pub upper_right: ScreenCoords,
}

impl ScreenRect {
    pub fn add_padding(&self, padding: f64) -> ScreenRect {
        ScreenRect {
            lower_left: ScreenCoords {
                x: self.lower_left.x - padding,
                y: self.lower_left.y - padding,
            },
            upper_right: ScreenCoords {
                x: self.upper_right.x + padding,
                y: self.upper_right.y + padding,
            },
        }
    }

    pub fn expand_to_fit(&mut self, point: ScreenCoords) {
        let ScreenCoords { x, y } = point;

        if x < self.lower_left.x {
            self.lower_left.x = x;
        }

        if x > self.upper_right.x {
            self.upper_right.x = x;
        }

        if y < self.lower_left.y {
            self.lower_left.y = y;
        }

        if y > self.upper_right.y {
            self.upper_right.y = y;
        }
    }
}

impl Rect for ScreenRect {
    fn min_x(&self) -> f64 {
        self.lower_left.x
    }

    fn max_x(&self) -> f64 {
        self.upper_right.x
    }

    fn min_y(&self) -> f64 {
        self.lower_left.y
    }

    fn max_y(&self) -> f64 {
        self.upper_right.y
    }
}

/***************************************************************************************************
 *                   Device Coords - the coordinate system of the device
 * ************************************************************************************************/
/// Device coordinates (pixels positions).
///  Origin upper left, (Column, Row)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviceCoords {
    pub col: f64,
    pub row: f64,
}

impl From<(f64, f64)> for DeviceCoords {
    fn from(src: (f64, f64)) -> Self {
        DeviceCoords {
            col: src.0,
            row: src.1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceRect {
    pub upper_left: DeviceCoords,
    pub width: f64,
    pub height: f64,
}

impl Rect for DeviceRect {
    fn min_x(&self) -> f64 {
        self.upper_left.col
    }

    fn max_x(&self) -> f64 {
        self.upper_left.col + self.width
    }

    fn min_y(&self) -> f64 {
        self.upper_left.row
    }

    fn max_y(&self) -> f64 {
        self.upper_left.row + self.height
    }
}

/***************************************************************************************************
 *                   Converting Pressure to the y coordinate
 * ************************************************************************************************/
/// Given a pressure value, convert it to a y-value from X-Y coordinates.
///
/// Overwhelmingly the veritical coordinate system is based on pressure, so this is a very common
/// operation to do, and you want it to always be done them same.
pub fn convert_pressure_to_y(pressure: HectoPascal) -> f64 {
    (config::MAXP / pressure).log10() / (config::MAXP / config::MINP).log10()
}

/// Provide an inverse function as well.
pub fn convert_y_to_pressure(y: f64) -> HectoPascal {
    config::MAXP * (config::MINP / config::MAXP).powf(y)
}

#[test]
fn test_pressure_to_y_and_back() {
    use metfor::Quantity;

    for i in 100..=1000 {
        let p = HectoPascal(i as f64);
        let y = convert_pressure_to_y(p);
        let pback = convert_y_to_pressure(y);
        println!("p = {:?} y = {:?}  pback = {:?}", p, y, pback);
        assert!((p - pback).abs() < HectoPascal(1.0e-6));
    }
}
