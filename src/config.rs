//! Keep configuration data in this module.

// Constants for defining a standard x-y coordinate system
/// Maximum pressure plotted on skew-t (bottom edge)
pub const MAXP: f32 = 1050.0; // mb
/// Minimum pressure plotted on skew-t (top edge)
pub const MINP: f32 = 99.0; // mb
/// Coldest temperature plotted at max pressure, on the bottom edge.
pub const MINT: f32 = -46.5; // C - at MAXP
/// Warmest temperature plotted at max pressure, on the bottom edge.
pub const MAXT: f32 = 50.5; // C - at MAXP

/// Hightest elevation pressure level to draw isentrops up to
pub const ISENTROPS_TOP_P: f32 = 300.0;
/// Number of points to use per isentrop line when drawing.
pub const POINTS_PER_ISENTROP: u32 = 30;

/// Line width in pixels for skew-t background lines.
pub const BACKGROUND_LINE_WIDTH: f64 = 1.0;

/// Background color
pub const BACKGROUND_RGB: (f64, f64, f64) = (0.0, 0.0, 0.0);
/// Color used for cold isotherms
pub const COLD_ISOTHERM_RGBA: (f64, f64, f64, f64) = (0.0, 0.0, 1.0, 0.5);
/// Color used for warm isotherms
pub const WARM_ISOTHERM_RGBA: (f64, f64, f64, f64) = (1.0, 0.0, 0.0, 0.5);
/// Color used for isobars
pub const ISOBAR_RGBA: (f64, f64, f64, f64) = (1.0, 1.0, 1.0, 0.5);
/// Color used for isentrops
pub const ISENTROP_RGBA: (f64, f64, f64, f64) = (0.6, 0.6, 0.0, 0.5);

/// Color used for temperature plot
pub const TEMPERATURE_RGBA: (f64, f64, f64, f64) = (1.0, 0.0, 0.0, 1.0);
/// Line width for Temperature Plot
pub const TEMPERATURE_LINE_WIDTH: f64 = 2.0;
/// Color used for dew point plot
pub const DEW_POINT_RGBA: (f64, f64, f64, f64) = (0.0, 1.0, 0.0, 1.0);
/// Line width for Dew point Plot
pub const DEW_POINT_LINE_WIDTH: f64 = 2.0;
/// Color used for wet bulb plot
pub const WET_BULB_RGBA: (f64, f64, f64, f64) = (0.0, 1.0, 1.0, 1.0);
/// Line width for Dew point Plot
pub const WET_BULB_LINE_WIDTH: f64 = 1.0;


/// Isotherms to plot on the chart, freezing and below.
pub const COLD_ISOTHERMS: [f32; 19] = [
    -150.0,
    -140.0,
    -130.0,
    -120.0,
    -110.0,
    -100.0,
    -90.0,
    -80.0,
    -70.0,
    -60.0,
    -50.0,
    -40.0,
    -30.0,
    -25.0,
    -20.0,
    -15.0,
    -10.0,
    -5.0,
    0.0,
];

/// Isotherms to plot on the chart, above freezing.
pub const WARM_ISOTHERMS: [f32; 12] = [
    5.0,
    10.0,
    15.0,
    20.0,
    25.0,
    30.0,
    35.0,
    40.0,
    45.0,
    50.0,
    55.0,
    60.0,
];

/// Isobars to plot on the chart background.
pub const ISOBARS: [f32; 9] = [
    1050.0,
    1000.0,
    925.0,
    850.0,
    700.0,
    500.0,
    300.0,
    200.0,
    100.0,
];

/// Isentrops to plot on the chart background.
pub const ISENTROPS: [f32; 17] = [
    230.0,
    240.0,
    250.0,
    260.0,
    270.0,
    280.0,
    290.0,
    300.0,
    310.0,
    320.0,
    330.0,
    340.0,
    350.0,
    360.0,
    370.0,
    380.0,
    390.0,
];
