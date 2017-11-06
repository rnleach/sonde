//! Keep configuration data in this module.

use coords::TPCoords;

/// Data that can be changed at run-time affecting the look and feel of the application.
pub struct Config {
    //
    // Window Layout
    //
    /// Width of window in pixels.
    pub window_width: i32,
    /// Height of window in pixels.
    pub window_height: i32,

    //
    // Wind profile
    //
    /// Wind barb shaft length in pixels.
    pub wind_barb_shaft_length: f64,
    /// Lenght of wind barbs and pennants in pixels.
    pub wind_barb_barb_length: f64,
    /// Width of wind barbs and pennants in pixels.
    pub wind_barb_pennant_width: f64,
    /// Radius of the dot on a wind barb in pixels.
    pub wind_barb_dot_radius: f64,
    /// Color used for winds plot.
    pub wind_rgba: (f64, f64, f64, f64),
    /// Line width in pixels for wind barbs.
    pub wind_barb_line_width: f64,
    /// Show the wind profile
    pub show_wind_profile: bool,

    //
    // Temperature profile
    //
    /// Color used for temperature plot.
    pub temperature_rgba: (f64, f64, f64, f64),
    /// Line width in pixels for temperature plot.
    pub temperature_line_width: f64,
    /// Show the temperature profile
    pub show_temperature: bool,

    //
    // Wet bulb temperature profile
    //
    /// Color used for wet bulb temperature plot.
    pub wet_bulb_rgba: (f64, f64, f64, f64),
    /// Line width in pixels for dew point plot.
    pub wet_bulb_line_width: f64,
    /// Show the wet bulb profile
    pub show_wet_bulb: bool,

    //
    // Dew point temperature profile
    //
    /// Color used for dew point plot.
    pub dew_point_rgba: (f64, f64, f64, f64),
    /// Line width in pixels for dew point plot
    pub dew_point_line_width: f64,
    /// Show the dew point profile
    pub show_dew_point: bool,

    //
    // Omega profile
    //
    /// Show the omega profile
    pub show_omega: bool,
    /// Line width in pixels for omega
    pub omega_line_width: f64,
    /// Color used for omega line
    pub omega_rgba: (f64, f64, f64, f64),

    //
    // Labeling
    //
    /// Whether to show labels
    pub show_labels: bool,
    /// Whether to show the legend
    pub show_legend: bool,
    /// Font face
    pub font_name: String,
    /// Font size for labels in points
    pub label_font_size: f64,
    /// Default padding in text boxes and the plot edge for text. In pixels.
    pub edge_padding: f64,
    ///  Default padding for labels and their background in pixels
    pub label_padding: f64,
    /// Label color
    pub label_rgba: (f64, f64, f64, f64),

    //
    // Background
    //
    /// Line width in pixels for skew-t background lines.
    pub background_line_width: f64,
    /// Background color
    pub background_rgba: (f64, f64, f64, f64),
    /// Background banding color for temperature bands.
    pub background_band_rgba: (f64, f64, f64, f64),
    /// Show or hide background temperature banding.
    pub show_background_bands: bool,
    /// Color used to fill the dendritic snow growth zone
    pub dendritic_zone_rgba: (f64, f64, f64, f64),
    /// Show or hide the dendritic zone banding.
    pub show_dendritic_zone: bool,
    /// Color used to fill the hail growth zone
    pub hail_zone_rgba: (f64, f64, f64, f64),
    // Show or hide the hail growth zone
    pub show_hail_zone: bool,
    /// Color used for isotherms
    pub isotherm_rgba: (f64, f64, f64, f64),
    pub show_isotherms: bool,
    /// Color used for isobars
    pub isobar_rgba: (f64, f64, f64, f64),
    pub show_isobars: bool,
    /// Color used for isentrops
    pub isentrop_rgba: (f64, f64, f64, f64),
    pub show_isentrops: bool,
    /// Color used for isopleths of mixing ration
    pub iso_mixing_ratio_rgba: (f64, f64, f64, f64),
    pub show_iso_mixing_ratio: bool,
    /// Color used for isopleths of theta-e
    pub iso_theta_e_rgba: (f64, f64, f64, f64),
    pub show_iso_theta_e: bool,

    //
    // Active readout
    //
    /// Active readout line width
    pub active_readout_line_width: f64,
    /// Active readout line color
    pub active_readout_line_rgba: (f64, f64, f64, f64),
    pub show_active_readout: bool,
}

impl Config {}

impl Default for Config {
    fn default() -> Self {
        Config {
            //
            // Window Layout
            //
            window_width: 850,
            window_height: 650,

            //
            // Wind profile
            //
            wind_barb_shaft_length: 35.0,
            wind_barb_barb_length: 15.0,
            wind_barb_pennant_width: 6.0,
            wind_barb_dot_radius: 3.5,
            wind_rgba: (0.0, 0.0, 0.0, 1.0),
            wind_barb_line_width: 1.0,
            show_wind_profile: true,

            //
            // Temperature profile
            //
            temperature_rgba: (0.0, 0.0, 0.0, 1.0),
            temperature_line_width: 2.0,
            show_temperature: true,

            //
            // Wet bulb temperature profile
            //
            wet_bulb_rgba: (0.0, 0.0, 0.0, 1.0),
            wet_bulb_line_width: 1.0,
            show_wet_bulb: true,

            //
            // Dew point temperature profile
            //
            dew_point_rgba: (0.0, 0.0, 0.0, 1.0),
            dew_point_line_width: 2.0,
            show_dew_point: true,

            //
            // Omega profile
            //
            show_omega: true,
            omega_line_width: 1.0,
            omega_rgba: (0.0, 0.0, 0.0, 1.0),

            //
            // Labeling
            //
            show_labels: true,
            show_legend: true,
            font_name: "Courier New".to_owned(),
            label_font_size: 12.0,
            edge_padding: 5.0,
            label_padding: 3.0,
            label_rgba: (0.862745098, 0.388235294, 0.156862745, 1.0),

            //
            // Background
            //
            background_line_width: 1.0,
            background_rgba: (1.0, 1.0, 1.0, 1.0),
            background_band_rgba: (0.933333333, 0.964705882, 0.917647059, 1.0),
            show_background_bands: true,
            dendritic_zone_rgba: (0.0, 0.466666667, 0.780392157, 1.0),
            show_dendritic_zone: true,
            hail_zone_rgba: (0.0, 0.803921569, 0.803921569, 1.0),
            show_hail_zone: true,
            isotherm_rgba: (0.862745098, 0.388235294, 0.156862745, 1.0),
            show_isotherms: true,
            isobar_rgba: (0.862745098, 0.388235294, 0.156862745, 1.0),
            show_isobars: true,
            isentrop_rgba: (0.862745098, 0.388235294, 0.156862745, 1.0),
            show_isentrops: true,
            iso_mixing_ratio_rgba: (0.090196078, 0.050980392, 0.360784314, 1.0),
            show_iso_mixing_ratio: true,
            iso_theta_e_rgba: (0.333333333, 0.662745098, 0.278431373, 1.0),
            show_iso_theta_e: true,

            //
            // Active readout
            //
            active_readout_line_width: 3.0,
            active_readout_line_rgba: (1.0, 0.0, 0.0, 1.0),
            show_active_readout: true,
        }
    }
}

/**************************************************************************************************
*                         Constant, compile time configuration items.
**************************************************************************************************/
//
// Window Layout
//
/// Window border width in pixels
pub const BORDER_WIDTH: u32 = 3;

//
// Constants for defining a standard x-y coordinate system
//
// NOTE: Leave these as compile time constants unless background isopleths are dynamically
//       calculated also.
/// Maximum pressure plotted on skew-t (bottom edge)
pub const MAXP: f64 = 1050.0; // mb
/// Minimum pressure plotted on skew-t (top edge)
pub const MINP: f64 = 99.0; // mb
/// Coldest temperature plotted at max pressure, on the bottom edge.
pub const MINT: f64 = -46.5; // C - at MAXP
/// Warmest temperature plotted at max pressure, on the bottom edge.
pub const MAXT: f64 = 50.5; // C - at MAXP

//
// Limits on the top pressure level for some background lines.
//
// NOTE: Leave these as compile time constants unless background isopleths are dynamically
//       calculated also.
/// Highest elevation pressure level to draw isentrops up to
pub const ISENTROPS_TOP_P: f64 = 200.0;
/// Number of points to use per isentrop line when drawing.
pub const POINTS_PER_ISENTROP: u32 = 30;
/// Hightest elevation pressure level to draw iso mixing ratio up to
pub const ISO_MIXING_RATIO_TOP_P: f64 = 300.0;

//
// Constant values to plot on background.
//
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

/// Isentrops to plot on the chart background.
pub const ISENTROPS: [f64; 17] = [
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

/// Constant theta-e in Celsius.
pub const ISO_THETA_E_C: [f64; 31] = [
    -20.0,
    -18.0,
    -16.0,
    -14.0,
    -12.0,
    -10.0,
    -8.0,
    -6.0,
    -4.0,
    -2.0,
    0.0,
    2.0,
    4.0,
    6.0,
    8.0,
    10.0,
    12.0,
    14.0,
    16.0,
    18.0,
    20.0,
    22.0,
    24.0,
    26.0,
    28.0,
    30.0,
    32.0,
    34.0,
    36.0,
    38.0,
    40.0,
];

/// Isopleths of mixing ratio
pub const ISO_MIXING_RATIO: [f64; 32] = [
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

lazy_static! {

    /// Compute points for background isotherms only once
    pub static ref ISOTHERM_PNTS: Vec<[TPCoords; 2]> = {
        ISOTHERMS
        .into_iter()
        .map(|t| {
            [
                TPCoords{temperature:*t, pressure:MAXP},
                TPCoords{temperature:*t, pressure:MINP}
            ]
        })
        .collect()
    };

    /// Compute points for background isobars only once
    pub static ref ISOBAR_PNTS: Vec<[TPCoords; 2]> = {
        ISOBARS
            .into_iter()
            .map(|p| {
                [
                    TPCoords{temperature:-150.0, pressure:*p},
                    TPCoords{temperature:60.0, pressure:*p}
                ]
            })
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
    pub static ref ISO_MIXING_RATIO_PNTS: Vec<[TPCoords; 2]> = {
        use formula::*;

        ISO_MIXING_RATIO
        .into_iter()
        .map(|mw| {
            [
                TPCoords{
                    temperature: temperature_from_p_and_saturated_mw(MAXP, *mw),
                    pressure: MAXP
                },
                TPCoords{
                    temperature: temperature_from_p_and_saturated_mw(ISO_MIXING_RATIO_TOP_P, *mw),
                    pressure: ISO_MIXING_RATIO_TOP_P,
                },
            ]
        })
        .collect()
    };

    /// Compute points for background theta-e
    pub static ref ISO_THETA_E_PNTS: Vec<Vec<TPCoords>> = {
        use formula::{find_root, theta_e_saturated_kelvin};

        ISO_THETA_E_C
        .iter()
        .map(|theta_c| theta_e_saturated_kelvin(1000.0, *theta_c))
        .map(|theta_e_k| {
            let mut v = vec![];
            let mut p = ISENTROPS_TOP_P;
            let dp = (MAXP - MINP) / POINTS_PER_ISENTROP as f64;
            while p < MAXP + 1.0001 * dp {
                let t = find_root(&|t| {theta_e_saturated_kelvin(p,t)- theta_e_k},
                    -150.0, 60.0);
                v.push(TPCoords{temperature:t, pressure: p});
                p += dp;
            }
            v
        })
        .collect()
    };
}

/// Generate a list of Temperature, Pressure points along an isentrope.
fn generate_isentrop(theta: f64) -> Vec<TPCoords> {
    use std::f64;
    use app::config::{MAXP, ISENTROPS_TOP_P, POINTS_PER_ISENTROP};

    let mut result = vec![];

    let mut p = MAXP;
    while p >= ISENTROPS_TOP_P {
        let t = ::formula::temperature_c_from_theta(theta, p);
        result.push(TPCoords {
            temperature: t,
            pressure: p,
        });
        p += (ISENTROPS_TOP_P - MAXP) / (POINTS_PER_ISENTROP as f64);
    }
    let t = ::formula::temperature_c_from_theta(theta, ISENTROPS_TOP_P);
    result.push(TPCoords {
        temperature: t,
        pressure: ISENTROPS_TOP_P,
    });

    result
}