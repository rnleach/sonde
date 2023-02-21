//! Keep configuration data in this module.

use crate::coords::{
    DtHCoords,
    DtPCoords,
    PPCoords,
    //SDCoords,
    SPCoords,
    TPCoords,
    WPCoords,
    XYCoords,
};
//use crate::gui::profiles::{CloudContext, RHOmegaContext, WindSpeedContext};
//use crate::gui::{FirePlumeContext, FirePlumeEnergyContext, HodoContext};
use crate::gui::SkewTContext;

use lazy_static::lazy_static;
use metfor::{
    Celsius,
    CelsiusDiff,
    HectoPascal,
    Kelvin,
    Knots,
    Meters,
    PaPS,
    Quantity,
    // WindSpdDir,
};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

/// Types of parcels you can use when drawing parcel analysis overlays.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParcelType {
    Surface,
    MixedLayer,
    MostUnstable,
    Effective,
    Convective,
}

/// Types of helicity to use when drawing hodograph overlays.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum HelicityType {
    SurfaceTo3km,
    Effective,
}

/// Which storm motion to plot the Helicity overlay for.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum StormMotionType {
    RightMover,
    LeftMover,
}

/// Type used for colors in Gtk
pub type Rgba = (f64, f64, f64, f64);
pub const GREEN: Rgba = (0.0, 0.8, 0.0, 1.0);
pub const BLUE: Rgba = (0.0, 0.0, 1.0, 1.0);
pub const RED: Rgba = (1.0, 0.0, 0.0, 1.0);

/// Data that can be changed at run-time affecting the look and feel of the application.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    //
    // Session information and window Layout
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
    /// The last file opened.
    pub last_open_file: Option<PathBuf>,

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
    /// Background higlighting in the index view for the parcel indexes.
    pub parcel_indexes_highlight: Rgba,
    /// Fill parcel positive and negative areas
    pub fill_parcel_areas: bool,
    /// Positive parcel area color.
    pub parcel_positive_rgba: Rgba,
    /// Negative parcel area color.
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
    /// Show or hide the effective inflow layer.
    pub show_inflow_layer: bool,
    /// Color for the effective inflow layer overlay
    pub inflow_layer_rgba: Rgba,

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
    /// Show the rh_ice profile
    pub show_rh_ice: bool,
    /// Color used for omega line
    pub omega_rgba: Rgba,
    /// RH color
    pub rh_rgba: Rgba,
    /// RH ice color
    pub rh_ice_rgba: Rgba,

    //
    // Cloud profile
    //
    /// Show the cloud frame
    pub show_cloud_frame: bool,
    /// Cloud Color
    pub cloud_rgba: Rgba,

    //
    // Labeling and text
    //
    /// Whether to show labels
    pub show_labels: bool,
    /// Whether to show the legend
    pub show_legend: bool,
    /// Font face
    pub font_name: String,
    /// Font size for labels in points
    pub label_font_size: f64,
    /// Font size for text windows.
    pub text_area_font_size_points: f64,
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
    /// Show/hide the active readout
    pub show_active_readout: bool,

    /// Show the active readout box
    pub show_active_readout_text: bool,
    /// Show the active readout horizontal line
    pub show_active_readout_line: bool,
    /// Active readout line width
    pub active_readout_line_width: f64,
    /// Active readout line color
    pub active_readout_line_rgba: Rgba,

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
    /// Plot hodograph for winds up to a minimum pressure.
    pub min_hodo_pressure: HectoPascal,
    /// Plot the helicity overlays.
    pub show_helicity_overlay: bool,
    /// Helicity overlay color
    pub helicity_rgba: Rgba,
    /// Which layer to plot the helicity for
    pub helicity_layer: HelicityType,
    /// Which storm motion to plot the helicity for.
    pub helicity_storm_motion: StormMotionType,
    /// Storm motion points color for the hodograph
    pub storm_motion_rgba: Rgba,

    //
    // Fire plume related settings.
    //
    pub fire_plume_line_color: Rgba,
    /// Line color of level of max integrated buoyancy on fire plume chart.
    pub fire_plume_lmib_color: Rgba,
    /// Line color of max height on fire plume chart.
    pub fire_plume_maxh_color: Rgba,
    /// Line color of the LCL on the fire plume chart.
    pub fire_plume_lcl_color: Rgba,
    /// Line color of percent wet cape on fire plume chart.
    pub fire_plume_pct_wet_cape_color: Rgba,
    /// Show the PFT overlay
    pub show_pft: bool,
    /// The width of the lines in the PFT overlay
    pub pft_line_width: f64,
    /// PFT SP-Curve color
    pub pft_sp_curve_color: Rgba,
    /// PFT mean specific humidity line color
    pub pft_mean_q_color: Rgba,
    /// PFT mean potential temperature line color
    pub pft_mean_theta_color: Rgba,
    /// PFT cloud parcel line color
    pub pft_cloud_parcel_color: Rgba,

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
            last_open_file: None,

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
            parcel_indexes_highlight: (0.0, 0.75, 0.75, 1.0),
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
            wet_bulb_zero_line_color: (0.360_784_313_725_490_2, 0.207_843_137_254_901_97, 0.4, 1.0),
            show_wet_bulb_zero_line: true,
            show_inflow_layer: true,
            inflow_layer_rgba: (1.0, 0.4, 0.1, 1.0),

            //
            // General profile configuration items
            //
            profile_line_width: 2.0,

            //
            // RH-Omega profile
            //
            show_omega: true,
            show_rh: true,
            show_rh_ice: false,
            omega_rgba: (0.0, 0.0, 0.0, 1.0),
            rh_rgba: (0.305_880, 0.603_921, 0.023_529_4, 0.75),
            rh_ice_rgba: (0.0, 0.603_921, 0.603_921, 0.50),

            //
            // Cloud profile
            //
            show_cloud_frame: true,
            cloud_rgba: (0.5, 0.5, 0.5, 0.75),

            //
            // Labeling and text
            //
            show_labels: true,
            show_legend: true,
            font_name: "Courier New".to_owned(),
            label_font_size: 2.0,
            text_area_font_size_points: 11.0,
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
            show_active_readout: true,
            show_active_readout_text: true,
            show_active_readout_line: true,
            active_readout_line_width: 3.0,
            active_readout_line_rgba: (1.0, 0.0, 0.0, 1.0),
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
            min_hodo_pressure: HectoPascal(300.0),
            show_helicity_overlay: true,
            helicity_rgba: (1.0, 0.4, 0.1, 0.6),
            helicity_layer: HelicityType::Effective,
            helicity_storm_motion: StormMotionType::RightMover,
            storm_motion_rgba: (0.0, 0.0, 0.0, 1.0),

            //
            // Fire plume related settings.
            //
            fire_plume_line_color: (1.0, 0.6, 0.0, 1.0),
            fire_plume_lmib_color: (1.0, 0.5, 0.0, 1.0),
            fire_plume_maxh_color: (0.0, 0.0, 0.8, 1.0),
            fire_plume_pct_wet_cape_color: (0.0, 0.0, 0.0, 1.0),
            fire_plume_lcl_color: (0.0, 0.7, 0.8, 1.0),
            show_pft: false,
            pft_line_width: 3.0,
            pft_sp_curve_color: (0.0, 0.2, 1.0, 1.0),
            pft_mean_q_color: (0.305_882_352, 0.603_921_568, 0.023_529_411, 1.0),
            pft_mean_theta_color: (0.807_843_137, 0.360_784_313, 0.0, 1.0),
            pft_cloud_parcel_color: (0.203_921_568, 0.396_078, 0.643_137_254, 1.0),

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
pub const MAXP: HectoPascal = HectoPascal(1050.0); // hPa
/// Minimum pressure plotted on skew-t (top edge)
pub const MINP: HectoPascal = HectoPascal(99.0); // hPa
/// Coldest temperature plotted at max pressure, on the bottom edge.
pub const MINT: Celsius = Celsius(-40.5); // C - at MAXP
/// Warmest temperature plotted at max pressure, on the bottom edge.
pub const MAXT: Celsius = Celsius(55.5); // C - at MAXP

/// Maximum absolute vertical velocity in Pa/s
pub const MAX_ABS_W: PaPS = PaPS(15.0);

/// Maximum wind speed on hodograph in Knots
pub const MAX_SPEED: Knots = Knots(200.0);

/// Maximum wind speed on the wind speed profile in Knots
pub const MAX_PROFILE_SPEED: Knots = MAX_SPEED;

/// Maximum DeltaT in fire plume plot
pub const MAX_DELTA_T: CelsiusDiff = CelsiusDiff(22.0);
/// Minimum DeltaT in fire plume plot
pub const MIN_DELTA_T: CelsiusDiff = CelsiusDiff(-2.0);
/// Maximum height for fire plume plot
pub const MAX_FIRE_PLUME_HEIGHT: Meters = Meters(15_000.0);
/// Minimum height for fire plume plot
pub const MIN_FIRE_PLUME_HEIGHT: Meters = Meters(-1_000.0);
/// Maximum cape for fire plume plot
pub const MAX_FIRE_PLUME_PCT: f64 = 110.0;
/// Minimum cape for fire plume plot
pub const MIN_FIRE_PLUME_PCT: f64 = -10.0;

//
// Limits on the top pressure level for some background lines.
//

/// Highest elevation pressure level to draw isentrops up to
pub const ISENTROPS_TOP_P: HectoPascal = MINP;
/// Moist adiabat highest elevation pressure to draw up to
pub const THETA_E_TOP_P: HectoPascal = HectoPascal(200.0);
/// Number of points to use per isentrop line when drawing.
pub const POINTS_PER_ISENTROP: u32 = 40;
/// Hightest elevation pressure level to draw iso mixing ratio up to
pub const ISO_MIXING_RATIO_TOP_P: HectoPascal = HectoPascal(400.0);

//
// Constant values to plot on background.
//

/// Isotherms to label on the chart.
pub const ISOTHERMS: [Celsius; 31] = [
    Celsius(-150.0),
    Celsius(-140.0),
    Celsius(-130.0),
    Celsius(-120.0),
    Celsius(-110.0),
    Celsius(-100.0),
    Celsius(-90.0),
    Celsius(-80.0),
    Celsius(-70.0),
    Celsius(-60.0),
    Celsius(-50.0),
    Celsius(-40.0),
    Celsius(-30.0),
    Celsius(-25.0),
    Celsius(-20.0),
    Celsius(-15.0),
    Celsius(-10.0),
    Celsius(-5.0),
    Celsius(0.0),
    Celsius(5.0),
    Celsius(10.0),
    Celsius(15.0),
    Celsius(20.0),
    Celsius(25.0),
    Celsius(30.0),
    Celsius(35.0),
    Celsius(40.0),
    Celsius(45.0),
    Celsius(50.0),
    Celsius(55.0),
    Celsius(60.0),
];

/// Isobars to plot on the chart background.
pub const ISOBARS: [HectoPascal; 9] = [
    HectoPascal(1050.0),
    HectoPascal(1000.0),
    HectoPascal(925.0),
    HectoPascal(850.0),
    HectoPascal(700.0),
    HectoPascal(500.0),
    HectoPascal(300.0),
    HectoPascal(200.0),
    HectoPascal(100.0),
];

/// Isentrops to plot on the chart background.
pub const ISENTROPS: [Kelvin; 17] = [
    Kelvin(230.0),
    Kelvin(240.0),
    Kelvin(250.0),
    Kelvin(260.0),
    Kelvin(270.0),
    Kelvin(280.0),
    Kelvin(290.0),
    Kelvin(300.0),
    Kelvin(310.0),
    Kelvin(320.0),
    Kelvin(330.0),
    Kelvin(340.0),
    Kelvin(350.0),
    Kelvin(360.0),
    Kelvin(370.0),
    Kelvin(380.0),
    Kelvin(390.0),
];

/// Constant theta-e in Celsius.
pub const ISO_THETA_E_C: [Celsius; 31] = [
    Celsius(-20.0),
    Celsius(-18.0),
    Celsius(-16.0),
    Celsius(-14.0),
    Celsius(-12.0),
    Celsius(-10.0),
    Celsius(-8.0),
    Celsius(-6.0),
    Celsius(-4.0),
    Celsius(-2.0),
    Celsius(0.0),
    Celsius(2.0),
    Celsius(4.0),
    Celsius(6.0),
    Celsius(8.0),
    Celsius(10.0),
    Celsius(12.0),
    Celsius(14.0),
    Celsius(16.0),
    Celsius(18.0),
    Celsius(20.0),
    Celsius(22.0),
    Celsius(24.0),
    Celsius(26.0),
    Celsius(28.0),
    Celsius(30.0),
    Celsius(32.0),
    Celsius(34.0),
    Celsius(36.0),
    Celsius(38.0),
    Celsius(40.0),
];

/// Isopleths of mixing ratio
pub const ISO_MIXING_RATIO: [f64; 34] = [
    0.01, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0,
    14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0, 44.0, 48.0, 52.0, 56.0, 60.0, 68.0, 76.0,
];

pub const ISO_OMEGA: [PaPS; 21] = [
    PaPS(-10.0),
    PaPS(-9.0),
    PaPS(-8.0),
    PaPS(-7.0),
    PaPS(-6.0),
    PaPS(-5.0),
    PaPS(-4.0),
    PaPS(-3.0),
    PaPS(-2.0),
    PaPS(-1.0),
    PaPS(0.0),
    PaPS(1.0),
    PaPS(2.0),
    PaPS(3.0),
    PaPS(4.0),
    PaPS(5.0),
    PaPS(6.0),
    PaPS(7.0),
    PaPS(8.0),
    PaPS(9.0),
    PaPS(10.0),
];

pub const ISO_SPEED: [Knots; 20] = [
    Knots(10.0),
    Knots(20.0),
    Knots(30.0),
    Knots(40.0),
    Knots(50.0),
    Knots(60.0),
    Knots(70.0),
    Knots(80.0),
    Knots(90.0),
    Knots(100.0),
    Knots(110.0),
    Knots(120.0),
    Knots(130.0),
    Knots(140.0),
    Knots(150.0),
    Knots(160.0),
    Knots(170.0),
    Knots(180.0),
    Knots(190.0),
    Knots(200.0),
];

pub const PERCENTS: [f64; 11] = [
    0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0,
];

pub const PROFILE_SPEEDS: [Knots; 20] = [
    Knots(1.0),
    Knots(2.0),
    Knots(3.0),
    Knots(4.0),
    Knots(5.0),
    Knots(6.0),
    Knots(7.0),
    Knots(8.0),
    Knots(9.0),
    Knots(10.0),
    Knots(20.0),
    Knots(30.0),
    Knots(40.0),
    Knots(50.0),
    Knots(60.0),
    Knots(70.0),
    Knots(80.0),
    Knots(90.0),
    Knots(100.0),
    Knots(200.0),
];

pub const FIRE_PLUME_DTS: [CelsiusDiff; 11] = [
    CelsiusDiff(0.0),
    CelsiusDiff(2.0),
    CelsiusDiff(4.0),
    CelsiusDiff(6.0),
    CelsiusDiff(8.0),
    CelsiusDiff(10.0),
    CelsiusDiff(12.0),
    CelsiusDiff(14.0),
    CelsiusDiff(16.0),
    CelsiusDiff(18.0),
    CelsiusDiff(20.0),
];

pub const FIRE_PLUME_HEIGHTS: [Meters; 8] = [
    Meters(0.0),
    Meters(2_000.0),
    Meters(4_000.0),
    Meters(6_000.0),
    Meters(8_000.0),
    Meters(10_000.0),
    Meters(12_000.0),
    Meters(14_000.0),
];

pub const FIRE_PLUME_PCTS: [f64; 11] = [
    0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0,
];

/* ------------------------------------------------------------------------------------------------
Values below this line are automatically calculated based on the configuration values above and
should not be altered.
------------------------------------------------------------------------------------------------ */

lazy_static! {

    /// Compute points for background isotherms only once
    pub static ref ISOTHERM_PNTS: Vec<[XYCoords; 2]> = {
        ISOTHERMS
        .iter()
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
        .iter()
        .map(|p| {
            [
                TPCoords{temperature:Celsius(-150.0), pressure:*p},
                TPCoords{temperature:Celsius(60.0), pressure:*p}
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
        .iter()
        .map(|theta| generate_isentrop(*theta))
        .collect()
    };

    /// Compute points for background mixing ratio only once
    pub static ref ISO_MIXING_RATIO_PNTS: Vec<[XYCoords; 2]> = {
        use metfor::*;

        ISO_MIXING_RATIO
        .iter()
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
        use metfor::equiv_pot_temperature;

        ISO_THETA_E_C
        .iter()
        .map(|theta_c| equiv_pot_temperature(*theta_c, *theta_c, HectoPascal(1000.0)).expect("theta_e isopleth failed"))
        .map(generate_theta_e_isopleth)
        .collect()
    };

    /// Compute points for background omega
    pub static ref ISO_OMEGA_PNTS: Vec<[XYCoords; 2]> = {
        ISO_OMEGA
            .iter()
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
//                RHOmegaContext::convert_wp_to_xy(tp[0]),
//                RHOmegaContext::convert_wp_to_xy(tp[1])
                XYCoords{x:0.0, y:0.0},
                XYCoords{x:1.0, y:1.0}
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
//                v.push(HodoContext::convert_sd_to_xy(SDCoords{spd_dir:WindSpdDir{speed, direction: dir}}));
//                dir += 1.0;
                v.push(XYCoords{x:0.0, y:0.0});
            }
            v
        })
        .collect()
    };

    /// Compute points for background cloud coverage
    pub static ref CLOUD_PERCENT_PNTS: Vec<[XYCoords; 2]> = {
        PERCENTS
            .iter()
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
//                CloudContext::convert_pp_to_xy(pp[0]),
//                CloudContext::convert_pp_to_xy(pp[1])
                XYCoords{x:0.0, y:0.0},
                XYCoords{x:1.0, y:1.0}
            ]
        })
            .collect()
    };

    /// Compute points for background cloud coverage
    pub static ref PROFILE_SPEED_PNTS: Vec<[XYCoords; 2]> = {
        PROFILE_SPEEDS
            .iter()
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
//                WindSpeedContext::convert_sp_to_xy(sp[0]),
//                WindSpeedContext::convert_sp_to_xy(sp[1])
                XYCoords{x:0.0, y:0.0},
                XYCoords{x:1.0, y:1.0}
            ]
        })
            .collect()
    };

    /// Compute points for background △T in fire plume charts
    pub static ref FIRE_PLUME_DT_PNTS: Vec<[XYCoords; 2]> = {
       FIRE_PLUME_DTS
           .iter()
           .map(|dt| {
               [
                   DtHCoords {
                   dt: *dt,
                   height: MAX_FIRE_PLUME_HEIGHT,
               },
                   DtHCoords {
                       dt: *dt,
                       height: MIN_FIRE_PLUME_HEIGHT,
                   },
               ]
               })
               .map(|dt| {
                   [
//                       FirePlumeContext::convert_dth_to_xy(dt[0]),
//                       FirePlumeContext::convert_dth_to_xy(dt[1]),
                XYCoords{x:0.0, y:0.0},
                XYCoords{x:1.0, y:1.0}
                   ]
               })
           .collect()
    };

    /// Compute points for background height in fire plume charts
    pub static ref FIRE_PLUME_HEIGHT_PNTS: Vec<[XYCoords; 2]> = {
       FIRE_PLUME_HEIGHTS
           .iter()
           .map(|height| {
               [
                   DtHCoords {
                   dt: MIN_DELTA_T,
                   height: *height,
               },
                   DtHCoords {
                       dt: MAX_DELTA_T,
                       height: *height,
                   },
               ]
               })
           .map(|dt| {
               [
 //                  FirePlumeContext::convert_dth_to_xy(dt[0]),
 //                  FirePlumeContext::convert_dth_to_xy(dt[1]),
                XYCoords{x:0.0, y:0.0},
                XYCoords{x:1.0, y:1.0}
               ]
           })
           .collect()
    };

    /// Compute points for background height in fire plume charts
    pub static ref FIRE_PLUME_PCTS_PNTS: Vec<[XYCoords; 2]> = {
       FIRE_PLUME_PCTS
           .iter()
           .map(|percent| {
               [
                   DtPCoords {
                       dt: MIN_DELTA_T,
                       percent: *percent,
                   },
                   DtPCoords {
                       dt: MAX_DELTA_T,
                       percent: *percent,
                   },
               ]
               })
           .map(|dt| {
               [
//                   FirePlumeEnergyContext::convert_dtp_to_xy(dt[0]),
//                   FirePlumeEnergyContext::convert_dtp_to_xy(dt[1]),
                XYCoords{x:0.0, y:0.0},
                XYCoords{x:1.0, y:1.0}
               ]
           })
           .collect()
    };

}

/// Generate a list of Temperature, Pressure points along an isentrope.
fn generate_isentrop(theta: Kelvin) -> Vec<XYCoords> {
    use metfor::temperature_from_pot_temp;
    use std::f64;

    let mut result = vec![];

    let mut p = MAXP;
    while p >= ISENTROPS_TOP_P {
        let t: Celsius = temperature_from_pot_temp(theta, p).into();
                result.push(SkewTContext::convert_tp_to_xy(TPCoords {
                    temperature: t,
                    pressure: p,
                }));
        p += HectoPascal((ISENTROPS_TOP_P - MAXP).unpack() / f64::from(POINTS_PER_ISENTROP));
    }
    let t: Celsius = temperature_from_pot_temp(theta, ISENTROPS_TOP_P).into();

        result.push(SkewTContext::convert_tp_to_xy(TPCoords {
            temperature: t,
            pressure: ISENTROPS_TOP_P,
        }));

    result
}

/// Generate an isopleth for equivalent potential temperatures.
fn generate_theta_e_isopleth(theta_e_k: Kelvin) -> Vec<XYCoords> {
    let mut v = vec![];
    let mut p = THETA_E_TOP_P;
    let dp = HectoPascal((MAXP - MINP).unpack() / f64::from(POINTS_PER_ISENTROP));

    while p < MAXP + dp * 1.0001 {
        match metfor::find_root(
            &|t| {
                Some(
                    (metfor::equiv_pot_temperature(Celsius(t), Celsius(t), p)? - theta_e_k)
                        .unpack(),
                )
            },
            Celsius(-80.0).unpack(),
            Celsius(50.0).unpack(),
        )
        .map(|t| {
            v.push(SkewTContext::convert_tp_to_xy(TPCoords {
                temperature: Celsius(t),
                pressure: p,
            }));
        }) {
            Some(_) => p += dp,
            None => {
                p = metfor::find_root(
                    &|p| {
                        Some(
                            (metfor::equiv_pot_temperature(
                                Celsius(-79.999),
                                Celsius(-79.999),
                                HectoPascal(p),
                            )? - theta_e_k)
                                .unpack(),
                        )
                    },
                    THETA_E_TOP_P.unpack(),
                    MAXP.unpack(),
                )
                .map(HectoPascal)
                .unwrap_or_else(|| p + HectoPascal(1.0))
            }
        }
    }
    v
}
