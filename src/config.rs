//! Keep configuration data in this module.

// TODO: Organize this better

// Layout and borders
/// Main window border width in pixels.
pub const BORDER_WIDTH: u32 = 3;
/// Main window initial width
pub const WINDOW_WIDTH: i32 = 850;
/// Main window initial height
pub const WINDOW_HEIGHT: i32 = 650;

/// Active readout line width
pub const ACTIVE_READOUT_LINE_WIDTH: f64 = 3.0;
/// Active readout line color
pub const ACTIVE_READOUT_LINE_RGB: (f64, f64, f64) = (1.0, 0.0, 0.0);

/// Wind barb shaft length in pixels
pub const WIND_BARB_SHAFT_LENGTH_IN_PIXELS: f64 = 35.0;
/// Lenght of wind barbs and pennants
pub const WIND_BARB_BARB_LENGTH_IN_PIXELS: f64 = 15.0;
/// Width of wind barbs and pennants
pub const WIND_BARB_PENNANT_WIDTH_IN_PIXELS: f64 = 6.0;
/// Size of the dot on a wind barb
pub const WIND_BARB_DOT_RADIUS_IN_PIXELS: f64 = 3.5;
/// Color used for winds plot
pub const WIND_RGBA: (f64, f64, f64, f64) = (0.0, 0.0, 0.0, 1.0);
/// Line width for Dew point Plot
pub const WIND_LINE_WIDTH: f64 = 1.0;

/// Font face
pub static FONT_NAME: &'static str = "Courier New";
/// Font size, legend, pressure, temperature lines
pub const LARGE_FONT_SIZE: f64 = 12.0;
/// Default padding in text boxes and the plot edge for text. In pixels.
pub const EDGE_PADDING: f64 = 5.0;
///  Default padding for labels and their background in pixels
pub const LABEL_PADDING: f64 = 3.0;
/// Label coloring
pub const LABEL_RGB: (f64, f64, f64) = (0.862745098, 0.388235294, 0.156862745);

// Constants for defining a standard x-y coordinate system
/// Maximum pressure plotted on skew-t (bottom edge)
pub const MAXP: f64 = 1050.0; // mb
/// Minimum pressure plotted on skew-t (top edge)
pub const MINP: f64 = 99.0; // mb
/// Coldest temperature plotted at max pressure, on the bottom edge.
pub const MINT: f64 = -46.5; // C - at MAXP
/// Warmest temperature plotted at max pressure, on the bottom edge.
pub const MAXT: f64 = 50.5; // C - at MAXP

/// Line width in pixels for skew-t background lines.
pub const BACKGROUND_LINE_WIDTH: f64 = 1.0;

/// Background color
pub const BACKGROUND_RGB: (f64, f64, f64) = (1.0, 1.0, 1.0);
//// Background banding color
pub const BACKGROUND_BAND_RGB: (f64, f64, f64) = (0.933333333, 0.964705882, 0.917647059);
/// Color used to fill the dendritic snow growth zone
pub const DENDRTITIC_ZONE_RGB: (f64, f64, f64) = (0.0, 0.466666667, 0.780392157);
/// Color used to fill the hail growth zone
pub const HAIL_ZONE_RGB: (f64, f64, f64) = (0.0, 0.803921569, 0.803921569);
/// Color used for isotherms
pub const ISOTHERM_RGBA: (f64, f64, f64, f64) = (0.862745098, 0.388235294, 0.156862745, 1.0);
/// Color used for isobars
pub const ISOBAR_RGBA: (f64, f64, f64, f64) = (0.862745098, 0.388235294, 0.156862745, 1.0);
/// Color used for isentrops
pub const ISENTROP_RGBA: (f64, f64, f64, f64) = (0.862745098, 0.388235294, 0.156862745, 1.0);
/// Color used for isopleths of mixing ration
pub const ISO_MIXING_RATIO_RGBA: (f64, f64, f64, f64) =
    (0.090196078, 0.050980392, 0.360784314, 1.0);
/// Color used for isopleths of theta-e
pub const ISO_THETA_E_RGBA: (f64, f64, f64, f64) = (0.333333333, 0.662745098, 0.278431373, 1.0);
/// Color used for temperature plot
pub const TEMPERATURE_RGBA: (f64, f64, f64, f64) = (0.0, 0.0, 0.0, 1.0);
/// Line width for Temperature Plot
pub const TEMPERATURE_LINE_WIDTH: f64 = 2.0;
/// Color used for dew point plot
pub const DEW_POINT_RGBA: (f64, f64, f64, f64) = (0.0, 0.0, 0.0, 1.0);
/// Line width for Dew point Plot
pub const DEW_POINT_LINE_WIDTH: f64 = 2.0;
/// Color used for wet bulb plot
pub const WET_BULB_RGBA: (f64, f64, f64, f64) = (0.0, 0.0, 0.0, 1.0);
/// Line width for Dew point Plot
pub const WET_BULB_LINE_WIDTH: f64 = 1.0;

/// Isotherms to label on the chart.
pub const ISOTHERMS: [f64; 31] = [
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
pub const ISOBARS: [f64; 9] = [
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

/* ------------------------------------------------------------------------------------------------
Values below this line are automatically calculated based on the configuration values above and
should not be altered.
------------------------------------------------------------------------------------------------ */
include!("background_data.rs");
