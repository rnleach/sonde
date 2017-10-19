/// Temperature-Pressure coordinates.
/// Origin lower left. (Temperature, Pressure)
pub type TPCoords = (f32, f32);
/// XY coordinates of the skew-t graph, range 0.0 to 1.0.
/// Origin lower left, (x,y)
pub type XYCoords = (f32, f32);
/// On screen coordinates. Meant to scale and translate XYCoords to fit on the screen.
/// Origin lower left, (x,y)
pub type ScreenCoords = (f64, f64);
/// Device coordinates (pixels positions).
///  Origin upper left, (Column, Row)
pub type DeviceCoords = (f64, f64);
