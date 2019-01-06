//! Keep configuration data in this module.

use crate::coords::{PPCoords, SDCoords, SPCoords, TPCoords, WPCoords, XYCoords};
use crate::gui::profiles::{CloudContext, RHOmegaContext, WindSpeedContext};
use crate::gui::{HodoContext, SkewTContext};

/// Types of parcels you can use when doing parcel analysis.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParcelType {
    Surface,
    MixedLayer,
    MostUnstable,
    Convective,
}

/// Type used for colors in Gtk
pub type Rgba = (f64, f64, f64, f64);

/// Data that can be changed at run-time affecting the look and feel of the application.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    //
    // Window Layout
    //
    /// Width of window in pixels.
    pub window_width: i32,
    /// Height of window in pixels.
    pub window_height: i32,
    /// Position of the main pane
    pub pane_position: f32,
    /// Tabs on the left
    pub left_tabs: Vec<String>,
    /// Tabs on the right
    pub right_tabs: Vec<String>,
    /// Selected tab on left notebook
    pub left_page_selected: i32,
    /// Selected tab on right notebook
    pub right_page_selected: i32,

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
    pub wind_rgba: Rgba,
    /// Line width in pixels for wind barbs.
    pub wind_barb_line_width: f64,
    /// Show the wind profile
    pub show_wind_profile: bool,

    //
    // Temperature profile
    //
    /// Color used for temperature plot.
    pub temperature_rgba: Rgba,
    /// Line width in pixels for temperature plot.
    pub temperature_line_width: f64,
    /// Show the temperature profile
    pub show_temperature: bool,

    //
    // Wet bulb temperature profile
    //
    /// Color used for wet bulb temperature plot.
    pub wet_bulb_rgba: Rgba,
    /// Line width in pixels for dew point plot.
    pub wet_bulb_line_width: f64,
    /// Show the wet bulb profile
    pub show_wet_bulb: bool,

    //
    // Dew point temperature profile
    //
    /// Color used for dew point plot.
    pub dew_point_rgba: Rgba,
    /// Line width in pixels for dew point plot
    pub dew_point_line_width: f64,
    /// Show the dew point profile
    pub show_dew_point: bool,

    //
    // Skew-T overlays
    //
    /// Parcel type to use when doing parcel analysis.
    pub parcel_type: ParcelType,
    /// Show parcel trajectory
    pub show_parcel_profile: bool,
    /// Parcel profile color.
    pub parcel_rgba: Rgba,
    /// Fill parcel positive and negative areas
    pub fill_parcel_areas: bool,
    /// Positive parcel area color.
    pub parcel_positive_rgba: Rgba,
    /// Negative parcela rea color.
    pub parcel_negative_rgba: Rgba,
    /// Show the inversion mix downs
    pub show_inversion_mix_down: bool,
    /// Inversion mix downs color
    pub inversion_mix_down_rgba: Rgba,
    /// Show the downburst profile
    pub show_downburst: bool,
    /// Downburst profile color
    pub downburst_rgba: Rgba,
    /// Fill the DCAPE area
    pub fill_dcape_area: bool,
    /// DCAPE area fill color
    pub dcape_area_color: Rgba,
    /// Color used to fill the dendritic snow growth zone
    pub dendritic_zone_rgba: Rgba,
    /// Show or hide the dendritic zone banding.
    pub show_dendritic_zone: bool,
    /// Color used to fill the hail growth zone
    pub hail_zone_rgba: Rgba,
    /// Show or hide the hail growth zone
    pub show_hail_zone: bool,
    /// Color used to fill the warm layer aloft
    pub warm_layer_rgba: Rgba,
    /// Color used to fill the wet bulb warm layer aloft
    pub warm_wet_bulb_aloft_rgba: Rgba,
    /// Show or hide the hail growth zone
    pub show_warm_layer_aloft: bool,
    /// Line width for freezing line
    pub freezing_line_width: f64,
    /// Color for freezing line
    pub freezing_line_color: Rgba,
    /// Show or hide freezing line
    pub show_freezing_line: bool,
    /// Line width for wet bulb zero line
    pub wet_bulb_zero_line_width: f64,
    /// Color for wet bulb zero line
    pub wet_bulb_zero_line_color: Rgba,
    /// Show or hide wet bulb zero line
    pub show_wet_bulb_zero_line: bool,

    //
    // General profile configuration items
    //
    /// Profile plot line widths
    pub profile_line_width: f64,

    //
    // RH-Omega profile
    //
    /// Show the omega profile
    pub show_omega: bool,
    /// Show the rh profile
    pub show_rh: bool,
    /// Color used for omega line
    pub omega_rgba: Rgba,
    /// RH Color
    pub rh_rgba: Rgba,

    //
    // Cloud profile
    //
    /// Show the cloud frame
    pub show_cloud_frame: bool,
    /// Cloud Color
    pub cloud_rgba: Rgba,

    //
    // Wind speed profile
    //
    /// Show the wind speed profile frame
    pub show_wind_speed_profile: bool,
    /// Wind speed profile color.
    pub wind_speed_profile_rgba: Rgba,

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
    pub label_rgba: Rgba,

    //
    // Background
    //
    /// Line width in pixels for skew-t background lines.
    pub background_line_width: f64,
    /// Background color
    pub background_rgba: Rgba,
    /// Background banding color for temperature bands.
    pub background_band_rgba: Rgba,
    /// Show or hide background temperature banding.
    pub show_background_bands: bool,

    /// Color used for isotherms
    pub isotherm_rgba: Rgba,
    pub show_isotherms: bool,
    /// Color used for isobars
    pub isobar_rgba: Rgba,
    pub show_isobars: bool,
    /// Color used for isentrops
    pub isentrop_rgba: Rgba,
    pub show_isentrops: bool,
    /// Color used for isopleths of mixing ration
    pub iso_mixing_ratio_rgba: Rgba,
    pub show_iso_mixing_ratio: bool,
    /// Color used for isopleths of theta-e
    pub iso_theta_e_rgba: Rgba,
    /// Show or hide the moist adiabats
    pub show_iso_theta_e: bool,

    //
    // Active readout
    //
    /// Active readout line width
    pub active_readout_line_width: f64,
    /// Active readout line color
    pub active_readout_line_rgba: Rgba,
    /// Show/hide the active readout
    pub show_active_readout: bool,
    /// Show sample parcel profile
    pub show_sample_parcel_profile: bool,
    /// Color for sample parcel profile
    pub sample_parcel_profile_color: Rgba,
    /// Show mix down profile of sample parcel
    pub show_sample_mix_down: bool,
    /// Sample mix down profile color
    pub sample_mix_down_rgba: Rgba,

    //
    // Hodograph
    //
    /// Background veclocity color
    pub iso_speed_rgba: Rgba,
    /// Show or hide iso speed lines
    pub show_iso_speed: bool,
    /// Velocity plot line width
    pub velocity_line_width: f64,
    /// Velociy line color
    pub veclocity_rgba: Rgba,
    /// Show or hide the velocity plot.
    pub show_velocity: bool,
    /// Plot hodograph for winds up to a minimum pressure.
    pub min_hodo_pressure: f64,

    //
    // Misc configuration.
    //
    pub bar_graph_line_width: f64,
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
            pane_position: 0.5,
            left_tabs: vec![],
            right_tabs: vec![],
            left_page_selected: 0,
            right_page_selected: 0,

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
            // Skew-T overlays
            //
            parcel_type: ParcelType::MostUnstable,
            show_parcel_profile: true,
            parcel_rgba: (0.0, 0.0, 0.0, 0.75),
            fill_parcel_areas: true,
            parcel_positive_rgba: (0.80, 0.0, 0.0, 0.5),
            parcel_negative_rgba: (0.0, 0.0, 0.80, 0.5),
            show_inversion_mix_down: true,
            inversion_mix_down_rgba: (0.560_784_313_725, 0.349_019_607_843, 0.007_843_137_254, 1.0),
            show_downburst: true,
            downburst_rgba: (0.0, 0.6, 0.0, 1.0),
            fill_dcape_area: true,
            dcape_area_color: (0.0, 0.6, 0.0, 0.5),
            dendritic_zone_rgba: (0.0, 0.466_666_667, 0.780_392_157, 0.55),
            show_dendritic_zone: true,
            hail_zone_rgba: (0.0, 0.803_921_569, 0.803_921_569, 0.55),
            show_hail_zone: true,
            warm_layer_rgba: (0.717_647, 0.254_9, 0.054_9, 0.55),
            warm_wet_bulb_aloft_rgba: (0.8, 0.0, 0.0, 1.0),
            show_warm_layer_aloft: true,
            freezing_line_width: 3.0,
            freezing_line_color: (0.0, 0.466_666_667, 0.780_392_157, 1.0),
            show_freezing_line: true,
            wet_bulb_zero_line_width: 3.0,
            wet_bulb_zero_line_color: (0.360_784_313_725_490_2, 0.207_843_137_254_901_98, 0.4, 1.0),
            show_wet_bulb_zero_line: true,

            //
            // General profile configuration items
            //
            profile_line_width: 2.0,

            //
            // RH-Omega profile
            //
            show_omega: true,
            show_rh: true,
            omega_rgba: (0.0, 0.0, 0.0, 1.0),
            rh_rgba: (0.305_880, 0.603_921, 0.023_529_4, 0.75),

            //
            // Cloud profile
            //
            show_cloud_frame: true,
            cloud_rgba: (0.5, 0.5, 0.5, 0.75),

            //
            // Wind speed profile
            //
            show_wind_speed_profile: true,
            wind_speed_profile_rgba: (0.0, 0.0, 0.0, 1.0),

            //
            // Labeling
            //
            show_labels: true,
            show_legend: true,
            font_name: "Courier New".to_owned(),
            label_font_size: 2.0,
            edge_padding: 5.0,
            label_padding: 3.0,
            label_rgba: (0.862_745_098, 0.388_235_294, 0.156_862_745, 1.0),

            //
            // Background
            //
            background_line_width: 1.0,
            background_rgba: (1.0, 1.0, 1.0, 1.0),
            background_band_rgba: (0.933_333_333, 0.964_705_882, 0.917_647_059, 1.0),
            show_background_bands: true,
            isotherm_rgba: (0.862_745_098, 0.388_235_294, 0.156_862_745, 1.0),
            show_isotherms: true,
            isobar_rgba: (0.862_745_098, 0.388_235_294, 0.156_862_745, 1.0),
            show_isobars: true,
            isentrop_rgba: (0.862_745_098, 0.388_235_294, 0.156_862_745, 1.0),
            show_isentrops: true,
            iso_mixing_ratio_rgba: (0.090_196_078, 0.050_980_392, 0.360_784_314, 1.0),
            show_iso_mixing_ratio: true,
            iso_theta_e_rgba: (0.333_333_333, 0.662_745_098, 0.278_431_373, 1.0),
            show_iso_theta_e: true,

            //
            // Active readout
            //
            active_readout_line_width: 3.0,
            active_readout_line_rgba: (1.0, 0.0, 0.0, 1.0),
            show_active_readout: true,
            show_sample_parcel_profile: true,
            sample_parcel_profile_color: (1.0, 0.0, 0.0, 1.0),
            show_sample_mix_down: true,
            sample_mix_down_rgba: (0.560_784_313_725, 0.349_019_607_843, 0.007_843_137_254, 1.0),

            //
            // Hodograph
            //
            iso_speed_rgba: (0.862_745_098, 0.388_235_294, 0.156_862_745, 1.0),
            show_iso_speed: true,
            velocity_line_width: 2.0,
            veclocity_rgba: (0.0, 0.0, 0.0, 1.0),
            show_velocity: true,
            min_hodo_pressure: 300.0,

            //
            // Misc configuration.
            //
            bar_graph_line_width: 2.0,
        }
    }
}

/**************************************************************************************************
*                         Constant, compile time configuration items.
**************************************************************************************************/
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

/// Maximum absolute vertical velocity in Pa/s
pub const MAX_ABS_W: f64 = 15.0;
/// Minimum allowable vertical velocity in Pa/s, used only for setting scale.
pub const MIN_ABS_W: f64 = 3.0;

/// Maximum wind speed on hodograph in Knots
pub const MAX_SPEED: f64 = 250.0;

/// Maximum wind speed on the wind speed profile in Knots
pub const MAX_PROFILE_SPEED: f64 = MAX_SPEED;

//
// Limits on the top pressure level for some background lines.
//

/// Highest elevation pressure level to draw isentrops up to
pub const ISENTROPS_TOP_P: f64 = MINP;
/// Moist adiabat highest elevation pressure to draw up to
pub const THETA_E_TOP_P: f64 = 200.0;
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
    1050.0, 1000.0, 925.0, 850.0, 700.0, 500.0, 300.0, 200.0, 100.0,
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
    0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0, 14.0,
    16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0, 44.0, 48.0, 52.0, 56.0, 60.0,
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

pub const PERCENTS: [f64; 11] = [
    0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0,
];

pub const PROFILE_SPEEDS: [f64; 20] = [
    1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0,
    90.0, 100.0, 200.0,
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
        use metfor::*;

        ISO_MIXING_RATIO
        .into_iter()
        .map(|mw| {
            [
                TPCoords{
                    temperature: dew_point_from_p_and_mw(MAXP, *mw/1000.0)
                        .expect("dp from mw fail"),
                    pressure: MAXP
                },
                TPCoords{
                    temperature: dew_point_from_p_and_mw(ISO_MIXING_RATIO_TOP_P, *mw/1000.0)
                        .expect("dp from mw fail"),
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
        use metfor::theta_e_kelvin;

        ISO_THETA_E_C
        .iter()
        .map(|theta_c| theta_e_kelvin(*theta_c, *theta_c, 1000.0).expect("theta_e isopleth failed"))
        .map(generate_theta_e_isopleth)
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

    /// Compute points for background cloud coverage
    pub static ref CLOUD_PERCENT_PNTS: Vec<[XYCoords; 2]> = {
        PERCENTS
            .into_iter()
            .map(|p| {
                [
                PPCoords {
                    pcnt: *p / 100.0,
                    press: MINP,
                },
                PPCoords {
                    pcnt: *p / 100.0,
                    press: MAXP,
                },
            ]
            })
        .map(|pp| {
            [
                CloudContext::convert_pp_to_xy(pp[0]),
                CloudContext::convert_pp_to_xy(pp[1])
            ]
        })
            .collect()
    };

    /// Compute points for background cloud coverage
    pub static ref PROFILE_SPEED_PNTS: Vec<[XYCoords; 2]> = {
        PROFILE_SPEEDS
            .into_iter()
            .map(|speed| {
                [
                SPCoords {
                    spd: *speed,
                    press: MINP,
                },
                SPCoords {
                    spd: *speed,
                    press: MAXP,
                },
            ]
            })
        .map(|sp| {
            [
                WindSpeedContext::convert_sp_to_xy(sp[0]),
                WindSpeedContext::convert_sp_to_xy(sp[1])
            ]
        })
            .collect()
    };
}

/// Generate a list of Temperature, Pressure points along an isentrope.
fn generate_isentrop(theta: f64) -> Vec<XYCoords> {
    use crate::app::config::{ISENTROPS_TOP_P, MAXP, POINTS_PER_ISENTROP};
    use metfor::temperature_c_from_theta;
    use std::f64;

    let mut result = vec![];

    let mut p = MAXP;
    while p >= ISENTROPS_TOP_P {
        let t = temperature_c_from_theta(theta, p).expect("constants should not fail!");
        result.push(SkewTContext::convert_tp_to_xy(TPCoords {
            temperature: t,
            pressure: p,
        }));
        p += (ISENTROPS_TOP_P - MAXP) / f64::from(POINTS_PER_ISENTROP);
    }
    let t = temperature_c_from_theta(theta, ISENTROPS_TOP_P).expect("constants should not fail!");

    result.push(SkewTContext::convert_tp_to_xy(TPCoords {
        temperature: t,
        pressure: ISENTROPS_TOP_P,
    }));

    result
}

/// Generate an isopleth for equivalent potential temperatures.
fn generate_theta_e_isopleth(theta_e_k: f64) -> Vec<XYCoords> {
    use metfor::theta_e_kelvin;
    let mut v = vec![];
    let mut p = THETA_E_TOP_P;
    let dp = (MAXP - MINP) / f64::from(POINTS_PER_ISENTROP);

    while p < MAXP + 1.0001 * dp {
        match find_root(&|t| Ok(theta_e_kelvin(t, t, p)? - theta_e_k), -80.0, 50.0).and_then(|t| {
            v.push(SkewTContext::convert_tp_to_xy(TPCoords {
                temperature: t,
                pressure: p,
            }));
            Ok(())
        }) {
            Ok(_) => p += dp,
            Err(_) => {
                p = find_root(
                    &|p| Ok(theta_e_kelvin(-79.999, -79.999, p)? - theta_e_k),
                    THETA_E_TOP_P,
                    MAXP,
                ).unwrap_or_else(|_| p + 1.0)
            }
        }
    }
    v
}

/// Bisection algorithm for finding the root of an equation given values bracketing a root. Used
/// when drawing moist adiabats.
use metfor::Result;
fn find_root(f: &Fn(f64) -> Result<f64>, mut low_val: f64, mut high_val: f64) -> Result<f64> {
    use metfor;

    use std::f64;
    const MAX_IT: usize = 50;
    const EPS: f64 = 1.0e-10;

    if low_val > high_val {
        ::std::mem::swap(&mut low_val, &mut high_val);
    }

    let mut f_low = f(low_val)?;
    let f_high = f(high_val)?;
    if f_high * f_low > 0.0 {
        return Err(metfor::MetForErr::InputOutOfRange);
    }

    let mut mid_val = (high_val - low_val) / 2.0 + low_val;
    let mut f_mid = f(mid_val)?;
    for _ in 0..MAX_IT {
        if f_mid * f_low > 0.0 {
            low_val = mid_val;
            f_low = f_mid;
        } else {
            high_val = mid_val;
            // f_high = f_mid;
        }

        if (high_val - low_val).abs() < EPS {
            break;
        }

        mid_val = (high_val - low_val) / 2.0 + low_val;
        f_mid = f(mid_val)?;
    }

    Ok(mid_val)
}
