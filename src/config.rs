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
/// Hightest elevation pressure level to draw iso mixing ratio up to
pub const ISO_MIXING_RATIO_TOP_P: f32 = 300.0;

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
/// Color used for isopleths of mixing ration
pub const ISO_MIXING_RATIO_RGBA: (f64, f64, f64, f64) = (0.0, 0.6, 0.0, 0.5);

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

/// Isopleths of mixing ratio
pub const ISO_MIXING_RATIO: [f32; 32] = [
    0.1,
    0.2,
    0.4,
    0.6,
    0.8,
    1.0,
    1.5,
    2.0,
    2.5,
    3.0,
    4.0,
    5.0,
    6.0,
    7.0,
    8.0,
    10.0,
    12.0,
    14.0,
    16.0,
    18.0,
    20.0,
    24.0,
    28.0,
    32.0,
    36.0,
    40.0,
    44.0,
    48.0,
    52.0,
    56.0,
    60.0,
    68.0,
//    76.0, // Uncomment this when we can have arrays larger than 32.
];

/* ------------------------------------------------------------------------------------------------
Values below this line are automatically calculated based on the configuration values above and
should not be altered.
------------------------------------------------------------------------------------------------ */
use gui::sounding::TPCoords;

lazy_static! {

    /// Compute points for background isotherms only once
    pub static ref COLD_ISOTHERM_PNTS: Vec<(TPCoords, TPCoords)> = {
        COLD_ISOTHERMS
        .into_iter()
        .map(|t| ((*t, MAXP), (*t, MINP)))
        .collect()
    };

    /// Compute points for background isotherms only once
    pub static ref WARM_ISOTHERM_PNTS: Vec<(TPCoords, TPCoords)> = {
        WARM_ISOTHERMS
        .into_iter()
        .map(|t| ((*t, MAXP), (*t, MINP)))
        .collect()
    };

    /// Compute points for background isobars only once
    pub static ref ISOBAR_PNTS: Vec<(TPCoords, TPCoords)> = {
        ISOBARS
            .into_iter()
            .map(|p| ((-150.0, *p), (60.0, *p)))
            .collect()
    };

    /// Compute points for background isentrops only once
    pub static ref ISENTROP_PNTS: Vec<Vec<TPCoords>> = {
        ISENTROPS
        .into_iter()
        .map(|theta| generate_isentrop(*theta))
        .collect()
    };

    /// Compute points for background mixing ratio only once
    pub static ref ISO_MIXING_RATIO_PNTS: Vec<(TPCoords, TPCoords)> = {
        ISO_MIXING_RATIO
        .into_iter()
        .map(|mw| {
            (
                (temperatures_from_p_and_mw(MAXP, *mw), MAXP),
                (
                    temperatures_from_p_and_mw(ISO_MIXING_RATIO_TOP_P, *mw),
                    ISO_MIXING_RATIO_TOP_P,
                ),
            )
        })
        .collect()
    };
}

/* ------------------------------------------------------------------------------------------------
Utility functions for calculating points.
------------------------------------------------------------------------------------------------ */
// FIXME: Move these to their own module, maybe even met calculations crate?
/// Generate a list of Temperature, Pressure points along an isentrope.
pub fn generate_isentrop(theta: f32) -> Vec<TPCoords> {
    use std::f32;
    use config::{MAXP, ISENTROPS_TOP_P, POINTS_PER_ISENTROP};
    const P0: f32 = 1000.0; // For calcuating theta

    let mut result = vec![];

    let mut p = MAXP;
    while p >= ISENTROPS_TOP_P {
        let t = theta * f32::powf(P0 / p, -0.286) - 273.15;
        result.push((t, p));
        p += (ISENTROPS_TOP_P - MAXP) / (POINTS_PER_ISENTROP as f32);
    }

    result
}

/// Given a mixing ratio and pressure, calculate the temperature. The p is in hPa and the mw is in
/// g/kg.
pub fn temperatures_from_p_and_mw(p: f32, mw: f32) -> f32 {
    use std::f32;

    let z = mw * p / 6.11 / 621.97 / (1.0 + mw / 621.97);
    237.5 * f32::log10(z) / (7.5 - f32::log10(z))
}
