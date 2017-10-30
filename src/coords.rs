// FIXME: Document the different coordinate systems.

/// Temperature-Pressure coordinates.
/// Origin lower left. (Temperature, Pressure)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TPCoords {
    /// Temperature in Celsius
    pub temperature: f64,
    /// Pressure in hPa
    pub pressure: f64,
}

/// XY coordinates of the skew-t graph, range 0.0 to 1.0.
/// Origin lower left, (x,y)
pub type XYCoords = (f64, f64);
/// On screen coordinates. Meant to scale and translate XYCoords to fit on the screen.
/// Origin lower left, (x,y)
pub type ScreenCoords = (f64, f64);
/// Device coordinates (pixels positions).
///  Origin upper left, (Column, Row)
pub type DeviceCoords = (f64, f64);
