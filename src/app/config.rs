//! Keep configuration data in this module.

use coords::{SDCoords, TPCoords, WPCoords, XYCoords};
use gui::{HodoContext, RHOmegaContext, SkewTContext};

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
    // RH-Omega profile
    //
    /// Show the rh omega frame
    pub show_rh_omega_frame: bool,
    /// Show the omega profile
    pub show_omega_profile: bool,
    /// Line width in pixels for omega
    pub omega_line_width: f64,
    /// Color used for omega line
    pub omega_rgba: (f64, f64, f64, f64),
    /// Show RH
    pub show_rh_profile: bool,
    /// RH Color
    pub rh_rgba: (f64, f64, f64, f64),

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
    /// Show or hide the hail growth zone
    pub show_hail_zone: bool,
    /// Line width for freezing line
    pub freezing_line_width: f64,
    /// Color for freezing line
    pub freezing_line_color: (f64, f64, f64, f64),
    /// Show or hide freezing line
    pub show_freezing_line: bool,
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
    /// Show or hide the moist adiabats
    pub show_iso_theta_e: bool,
    /// Show the omega lines
    pub show_iso_omega_lines: bool,

    //
    // Active readout
    //
    /// Active readout line width
    pub active_readout_line_width: f64,
    /// Active readout line color
    pub active_readout_line_rgba: (f64, f64, f64, f64),
    pub show_active_readout: bool,

    //
    // Hodograph
    //
    /// Background veclocity color
    pub iso_speed_rgba: (f64, f64, f64, f64),
    /// Show or hide iso speed lines
    pub show_iso_speed: bool,
    /// Velocity plot line width
    pub velocity_line_width: f64,
    /// Velociy line color
    pub veclocity_rgba: (f64, f64, f64, f64),
    /// Show or hide the velocity plot.
    pub show_velocity: bool,
    /// Plot hodograph for winds up to a minimum pressure.
    pub min_hodo_pressure: f64,
}

impl Config {}

impl Default for Config {
    fn default() -> Self {
        Config {
            //
            // Window Layout
            //
            window_width: 1100,
            window_height: 550,

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
            // RH-Omega profile
            //
            show_rh_omega_frame: true,
            show_omega_profile: true,
            omega_line_width: 2.0,
            omega_rgba: (0.0, 0.0, 0.0, 1.0),
            show_rh_profile: true,
            rh_rgba: (0.30588, 0.603921, 0.0235294, 1.0),

            //
            // Labeling
            //
            show_labels: true,
            show_legend: true,
            font_name: "Courier New".to_owned(),
            label_font_size: 2.0,
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
            freezing_line_width: 3.0,
            freezing_line_color: (0.0, 0.466666667, 0.780392157, 1.0),
            show_freezing_line: true,
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
            show_iso_omega_lines: true,

            //
            // Active readout
            //
            active_readout_line_width: 3.0,
            active_readout_line_rgba: (1.0, 0.0, 0.0, 1.0),
            show_active_readout: true,

            //
            // Hodograph
            //
            iso_speed_rgba: (0.862745098, 0.388235294, 0.156862745, 1.0),
            show_iso_speed: true,
            velocity_line_width: 2.0,
            veclocity_rgba: (0.0, 0.0, 0.0, 1.0),
            show_velocity: true,
            min_hodo_pressure: 300.0,
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
/// Margin around selected widgets.
pub const WIDGET_MARGIN: i32 = 4;

//
// Constants for defining a standard x-y coordinate system
//

/// Maximum pressure plotted on skew-t (bottom edge)
pub const MAXP: f64 = 1050.0; // mb
/// Minimum pressure plotted on skew-t (top edge)
pub const MINP: f64 = 99.0; // mb
/// Coldest temperature plotted at max pressure, on the bottom edge.
pub const MINT: f64 = -46.5; // C - at MAXP
/// Warmest temperature plotted at max pressure, on the bottom edge.
pub const MAXT: f64 = 50.5; // C - at MAXP
/// Width of the RH-Omega plot area as the decimal fraction of the `DrawingArea` width.
pub const RH_OMEGA_WIDTH: f64 = 0.10;

/// Maximum absolute vertical velocity in Pa/s
pub const MAX_ABS_W: f64 = 10.0;

/// Maximum wind speed on hodograph in Knots
pub const MAX_SPEED: f64 = 250.0;

//
// Limits on the top pressure level for some background lines.
//

/// Highest elevation pressure level to draw isentrops up to
pub const ISENTROPS_TOP_P: f64 = MINP;
/// Number of points to use per isentrop line when drawing.
pub const POINTS_PER_ISENTROP: u32 = 40;
/// Hightest elevation pressure level to draw iso mixing ratio up to
pub const ISO_MIXING_RATIO_TOP_P: f64 = 300.0;

//
// Constant values to plot on background.
//

/// Isotherms to label on the chart.
pub const ISOTHERMS: [f64; 31] = [
    -150.0, -140.0, -130.0, -120.0, -110.0, -100.0, -90.0, -80.0, -70.0, -60.0, -50.0, -40.0,
    -30.0, -25.0, -20.0, -15.0, -10.0, -5.0, 0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0,
    45.0, 50.0, 55.0, 60.0,
];

/// Isobars to plot on the chart background.
pub const ISOBARS: [f64; 9] = [
    1050.0, 1000.0, 925.0, 850.0, 700.0, 500.0, 300.0, 200.0, 100.0
];

/// Isentrops to plot on the chart background.
pub const ISENTROPS: [f64; 17] = [
    230.0, 240.0, 250.0, 260.0, 270.0, 280.0, 290.0, 300.0, 310.0, 320.0, 330.0, 340.0, 350.0,
    360.0, 370.0, 380.0, 390.0,
];

/// Constant theta-e in Celsius.
pub const ISO_THETA_E_C: [f64; 31] = [
    -20.0, -18.0, -16.0, -14.0, -12.0, -10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0,
    10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0,
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

pub const ISO_OMEGA: [f64; 21] = [
    -10.0, -9.0, -8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0,
    7.0, 8.0, 9.0, 10.0,
];

pub const ISO_SPEED: [f64; 25] = [
    10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 130.0, 140.0, 150.0,
    160.0, 170.0, 180.0, 190.0, 200.0, 210.0, 220.0, 230.0, 240.0, 250.0,
];

/* ------------------------------------------------------------------------------------------------
Values below this line are automatically calculated based on the configuration values above and
should not be altered.
------------------------------------------------------------------------------------------------ */

lazy_static! {

    /// Compute points for background isotherms only once
    pub static ref ISOTHERM_PNTS: Vec<[XYCoords; 2]> = {

        ISOTHERMS
        .into_iter()
        .map(|t| {
            [
                TPCoords{temperature:*t, pressure:MAXP},
                TPCoords{temperature:*t, pressure:MINP}
            ]
        })
        .map(|tp| {
            [
                SkewTContext::convert_tp_to_xy(tp[0]),
                SkewTContext::convert_tp_to_xy(tp[1])
            ]
        })
        .collect()
    };

    /// Compute points for background isobars only once
    pub static ref ISOBAR_PNTS: Vec<[XYCoords; 2]> = {
        ISOBARS
        .into_iter()
        .map(|p| {
            [
                TPCoords{temperature:-150.0, pressure:*p},
                TPCoords{temperature:60.0, pressure:*p}
            ]
        })
        .map(|tp| {
            [
                SkewTContext::convert_tp_to_xy(tp[0]),
                SkewTContext::convert_tp_to_xy(tp[1])
            ]
        })
        .collect()
    };

    /// Compute points for background isentrops only once
    pub static ref ISENTROP_PNTS: Vec<Vec<XYCoords>> = {
        ISENTROPS
        .into_iter()
        .map(|theta| generate_isentrop(*theta))
        .collect()
    };

    /// Compute points for background mixing ratio only once
    pub static ref ISO_MIXING_RATIO_PNTS: Vec<[XYCoords; 2]> = {
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
        .map(|tp| {
            [
                SkewTContext::convert_tp_to_xy(tp[0]),
                SkewTContext::convert_tp_to_xy(tp[1])
            ]
        })
        .collect()
    };

    /// Compute points for background theta-e
    pub static ref ISO_THETA_E_PNTS: Vec<Vec<XYCoords>> = {
        use formula::{find_root, theta_e_saturated_kelvin};

        ISO_THETA_E_C
        .iter()
        .map(|theta_c| theta_e_saturated_kelvin(1000.0, *theta_c))
        .map(|theta_e_k| {
            let mut v = vec![];
            let mut p = ISENTROPS_TOP_P;
            let dp = (MAXP - MINP) / f64::from(POINTS_PER_ISENTROP);
            while p < MAXP + 1.0001 * dp {
                let t = find_root(&|t| {theta_e_saturated_kelvin(p,t)- theta_e_k},
                    -150.0, 60.0);
                v.push(SkewTContext::convert_tp_to_xy(TPCoords{temperature:t, pressure: p}));
                p += dp;
            }
            v
        })
        .collect()
    };

    /// Compute points for background omega
    pub static ref ISO_OMEGA_PNTS: Vec<[XYCoords; 2]> = {
        ISO_OMEGA
            .into_iter()
            .map(|w| {
                [
                WPCoords {
                    w: *w,
                    p: MINP,
                },
                WPCoords {
                    w: *w,
                    p: MAXP,
                },
            ]
            })
        .map(|tp| {
            [
                RHOmegaContext::convert_wp_to_xy(tp[0]),
                RHOmegaContext::convert_wp_to_xy(tp[1])
            ]
        })
            .collect()
    };

    /// Compute points for background speed
    pub static ref ISO_SPEED_PNTS: Vec<Vec<XYCoords>> = {

        ISO_SPEED
        .iter()
        .map(|&speed| {
            let mut v = vec![];
            let mut dir = 0.0;
            while dir <= 361.0 {
                v.push(HodoContext::convert_sd_to_xy(SDCoords{speed, dir}));
                dir += 1.0;
            }
            v
        })
        .collect()
    };
}

/// Generate a list of Temperature, Pressure points along an isentrope.
fn generate_isentrop(theta: f64) -> Vec<XYCoords> {
    use std::f64;
    use app::config::{ISENTROPS_TOP_P, MAXP, POINTS_PER_ISENTROP};

    let mut result = vec![];

    let mut p = MAXP;
    while p >= ISENTROPS_TOP_P {
        let t = ::formula::temperature_c_from_theta(theta, p);
        result.push(SkewTContext::convert_tp_to_xy(TPCoords {
            temperature: t,
            pressure: p,
        }));
        p += (ISENTROPS_TOP_P - MAXP) / f64::from(POINTS_PER_ISENTROP);
    }
    let t = ::formula::temperature_c_from_theta(theta, ISENTROPS_TOP_P);

    result.push(SkewTContext::convert_tp_to_xy(TPCoords {
        temperature: t,
        pressure: ISENTROPS_TOP_P,
    }));

    result
}
