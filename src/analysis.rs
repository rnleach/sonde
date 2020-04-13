//! Data type and methods for building and describing an analysis.
//!
//! Not every possible analysis is in this data.
use metfor::{
    Celsius, CelsiusDiff, HectoPascal, IntHelicityM2pS2, JpKg, Km, Meters, MetersPSec, Mm, WindUV,
};
use optional::{none, some, Optioned};
use sounding_analysis::{
    average_parcel, bunkers_storm_motion, dcape, effective_inflow_layer,
    experimental::fire::{blow_up, calc_plumes, PlumeAscentAnalysis},
    haines, haines_high, haines_low, haines_mid, hot_dry_windy, lift_parcel, mean_wind,
    mixed_layer_parcel, most_unstable_parcel, precipitable_water, robust_convective_parcel_ascent,
    sr_helicity, surface_parcel, Layer, Parcel, ParcelAscentAnalysis, ParcelProfile, PrecipType,
    Sounding,
};
use std::collections::HashMap;

/// Convenient package for commonly requested analysis values.
///
/// All parcel related values are assumed to be for the 100hPa mixed layer at the surface.
#[derive(Debug, Clone)]
pub struct Analysis {
    // Sounding used to make the analysis
    sounding: Sounding,

    // Profile specific indicies
    precipitable_water: Optioned<Mm>,
    convective_t: Optioned<Celsius>,
    right_mover: Optioned<WindUV<MetersPSec>>,
    left_mover: Optioned<WindUV<MetersPSec>>,
    mean_wind: Optioned<WindUV<MetersPSec>>,
    sr_helicity_3k_rm: Optioned<IntHelicityM2pS2>,
    sr_helicity_3k_lm: Optioned<IntHelicityM2pS2>,
    effective_inflow_layer: Option<Layer>,
    sr_helicity_eff_rm: Optioned<IntHelicityM2pS2>,
    sr_helicity_eff_lm: Optioned<IntHelicityM2pS2>,

    // Fire weather indicies
    haines: Optioned<u8>,
    haines_low: Optioned<u8>,
    haines_mid: Optioned<u8>,
    haines_high: Optioned<u8>,
    hdw: Optioned<f64>,
    blow_up_anal_start_parcel: Option<Parcel>,
    el_blow_up_dt_low: Optioned<CelsiusDiff>,
    el_blow_up_height_low: Optioned<Meters>,
    el_blow_up_dt_high: Optioned<CelsiusDiff>,
    el_blow_up_height_high: Optioned<Meters>,
    top_blow_up_dt_low: Optioned<CelsiusDiff>,
    top_blow_up_height_low: Optioned<Meters>,
    top_blow_up_dt_high: Optioned<CelsiusDiff>,
    top_blow_up_height_high: Optioned<Meters>,
    lcl_dt_low: Optioned<CelsiusDiff>,
    plumes_low: Option<Vec<PlumeAscentAnalysis>>,
    plumes_high: Option<Vec<PlumeAscentAnalysis>>,
    max_p: HectoPascal, // Keep track of the lowest level in the sounding.

    // Downburst
    dcape: Optioned<JpKg>,
    downrush_t: Optioned<Celsius>,
    downburst_profile: Option<ParcelProfile>,

    // Parcel analysis
    mixed_layer: Option<ParcelAscentAnalysis>,
    surface: Option<ParcelAscentAnalysis>,
    most_unstable: Option<ParcelAscentAnalysis>,
    convective: Option<ParcelAscentAnalysis>,
    effective: Option<ParcelAscentAnalysis>,

    // Precipitation
    provider_wx_code: Option<PrecipType>,
    bourgouin_wx_code: Option<PrecipType>,
    nssl_wx_code: Option<PrecipType>,

    // Provider analysis
    provider_analysis: HashMap<&'static str, f64>,
}

impl Analysis {
    /// Create a new `Analysis`.
    pub fn new(snd: Sounding) -> Self {
        let max_p = snd
            .bottom_up()
            .filter_map(|dr| dr.pressure.into_option())
            .next()
            .unwrap_or(HectoPascal(0.0));

        Analysis {
            sounding: snd,
            precipitable_water: none(),
            convective_t: none(),
            right_mover: none(),
            left_mover: none(),
            mean_wind: none(),
            sr_helicity_3k_rm: none(),
            sr_helicity_3k_lm: none(),
            effective_inflow_layer: None,
            sr_helicity_eff_rm: none(),
            sr_helicity_eff_lm: none(),

            haines: none(),
            haines_low: none(),
            haines_mid: none(),
            haines_high: none(),
            hdw: none(),
            blow_up_anal_start_parcel: None,
            el_blow_up_dt_low: none(),
            el_blow_up_height_low: none(),
            el_blow_up_dt_high: none(),
            el_blow_up_height_high: none(),
            top_blow_up_dt_low: none(),
            top_blow_up_height_low: none(),
            top_blow_up_dt_high: none(),
            top_blow_up_height_high: none(),
            lcl_dt_low: none(),
            plumes_low: None,
            plumes_high: None,
            max_p,

            dcape: none(),
            downrush_t: none(),
            downburst_profile: None,

            mixed_layer: None,
            surface: None,
            most_unstable: None,
            convective: None,
            effective: None,

            provider_wx_code: None,
            bourgouin_wx_code: None,
            nssl_wx_code: None,

            provider_analysis: HashMap::new(),
        }
    }

    /// Get the precipitable water.
    pub fn pwat(&self) -> Optioned<Mm> {
        self.precipitable_water
    }

    /// Get the convective temperature.
    pub fn convective_t(&self) -> Optioned<Celsius> {
        self.convective_t
    }

    /// Get the right mover.
    pub fn right_mover(&self) -> Optioned<WindUV<MetersPSec>> {
        self.right_mover
    }

    /// Get the left mover.
    pub fn left_mover(&self) -> Optioned<WindUV<MetersPSec>> {
        self.left_mover
    }

    /// Get the mean wind.
    pub fn mean_wind(&self) -> Optioned<WindUV<MetersPSec>> {
        self.mean_wind
    }

    /// Get the storm relative helicity for a right mover storm
    pub fn sr_helicity_3k_rm(&self) -> Optioned<IntHelicityM2pS2> {
        self.sr_helicity_3k_rm
    }

    /// Get the storm relative helicity for a left mover storm
    pub fn sr_helicity_3k_lm(&self) -> Optioned<IntHelicityM2pS2> {
        self.sr_helicity_3k_lm
    }

    /// Get the effective inflow layer
    pub fn effective_inflow_layer(&self) -> Option<Layer> {
        self.effective_inflow_layer
    }

    /// Get the effective storm relative helicity for a right mover storm
    pub fn sr_helicity_eff_rm(&self) -> Optioned<IntHelicityM2pS2> {
        self.sr_helicity_eff_rm
    }

    /// Get the effective storm relative helicity for a left mover storm
    pub fn sr_helicity_eff_lm(&self) -> Optioned<IntHelicityM2pS2> {
        self.sr_helicity_eff_lm
    }

    /// Get the downrush temperature from a microburst.
    pub fn downrush_t(&self) -> Optioned<Celsius> {
        self.downrush_t
    }

    /// Get the 1 hour precipitation from the provider analysis, if it exists.
    pub fn provider_1hr_precip(&self) -> Optioned<Mm> {
        Optioned::from(
            self.provider_analysis
                .get("Precipitation1HrMm")
                .map(|val| Mm(*val)),
        )
    }

    /// Get the 1 hour convective precipitation from the provider analysis, if it exists.
    pub fn provider_1hr_convective_precip(&self) -> Optioned<Mm> {
        Optioned::from(
            self.provider_analysis
                .get("ConvectivePrecip1HrMm")
                .map(|val| Mm(*val)),
        )
    }

    /// Get the visibility from the provider analysis, if it exists.
    pub fn provider_vis(&self) -> Optioned<Km> {
        Optioned::from(
            self.provider_analysis
                .get("VisibilityKm")
                .map(|val| Km(*val)),
        )
    }

    /// Get the weather symbol code from the provider, i.e. model physics scheme.
    pub fn provider_precip_type(&self) -> Option<PrecipType> {
        self.provider_wx_code
    }

    /// Get the weather symbol code using the Bourgouin method.
    pub fn bourgouin_precip_type(&self) -> Option<PrecipType> {
        self.bourgouin_wx_code
    }

    /// Get the weather symbol code using the NSSL method.
    pub fn nssl_precip_type(&self) -> Option<PrecipType> {
        self.nssl_wx_code
    }

    /// Get the Haines Index.
    #[allow(dead_code)]
    pub fn haines(&self) -> Optioned<u8> {
        self.haines
    }

    /// Get the low level Haines Index.
    pub fn haines_low(&self) -> Optioned<u8> {
        self.haines_low
    }

    /// Get the mid level Haines Index.
    pub fn haines_mid(&self) -> Optioned<u8> {
        self.haines_mid
    }

    /// Get the high level Haines Index.
    pub fn haines_high(&self) -> Optioned<u8> {
        self.haines_high
    }

    /// Get the hot-dry-windy index.
    pub fn hdw(&self) -> Optioned<f64> {
        self.hdw
    }

    /// Get the change in temperature required for a blow up. EXPERIMENTAL.
    pub fn el_blow_up_dt_low(&self) -> Optioned<CelsiusDiff> {
        self.el_blow_up_dt_low
    }

    /// Get the change in temperature required for a blow up. EXPERIMENTAL.
    pub fn el_blow_up_dt_high(&self) -> Optioned<CelsiusDiff> {
        self.el_blow_up_dt_high
    }

    /// Get the height change of the EL if the blow up dt is met. EXPERIMENTAL.
    pub fn el_blow_up_height_change_low(&self) -> Optioned<Meters> {
        self.el_blow_up_height_low
    }

    /// Get the height change of the EL if the blow up dt is met. EXPERIMENTAL.
    pub fn el_blow_up_height_change_high(&self) -> Optioned<Meters> {
        self.el_blow_up_height_high
    }

    /// Get the change in temperature required for a blow up. EXPERIMENTAL.
    pub fn top_blow_up_dt_low(&self) -> Optioned<CelsiusDiff> {
        self.top_blow_up_dt_low
    }

    /// Get the change in temperature required for a blow up. EXPERIMENTAL.
    pub fn top_blow_up_dt_high(&self) -> Optioned<CelsiusDiff> {
        self.top_blow_up_dt_high
    }

    /// Get the height change of the EL if the blow up dt is met. EXPERIMENTAL.
    pub fn top_blow_up_height_change_low(&self) -> Optioned<Meters> {
        self.top_blow_up_height_low
    }

    /// Get the height change of the EL if the blow up dt is met. EXPERIMENTAL.
    pub fn top_blow_up_height_change_high(&self) -> Optioned<Meters> {
        self.top_blow_up_height_high
    }

    /// Get the amount of heating necessary to create a cloud on the plume top.
    pub fn lcl_dt_low(&self) -> Optioned<CelsiusDiff> {
        self.lcl_dt_low
    }

    /// Get the starting parcel for a blow up analysis.
    pub fn starting_parcel_for_blow_up_anal(&self) -> Option<Parcel> {
        self.blow_up_anal_start_parcel
    }

    /// Get the plumes analysis
    pub fn plumes_low(&self) -> &Option<Vec<PlumeAscentAnalysis>> {
        &self.plumes_low
    }

    /// Get the plumes analysis
    pub fn plumes_high(&self) -> &Option<Vec<PlumeAscentAnalysis>> {
        &self.plumes_high
    }

    /// Get the max pressure (lowest level) in the sounding
    pub fn max_pressure(&self) -> HectoPascal {
        self.max_p
    }

    /// Get the DCAPE.
    pub fn dcape(&self) -> Optioned<JpKg> {
        self.dcape
    }

    /// Get the mixed layer parcel analysis
    pub fn mixed_layer_parcel_analysis(&self) -> Option<&ParcelAscentAnalysis> {
        self.mixed_layer.as_ref()
    }

    /// Get the surface parcel analysis
    pub fn surface_parcel_analysis(&self) -> Option<&ParcelAscentAnalysis> {
        self.surface.as_ref()
    }

    /// Get the most unstable parcel analysis
    pub fn most_unstable_parcel_analysis(&self) -> Option<&ParcelAscentAnalysis> {
        self.most_unstable.as_ref()
    }

    /// Get the convective parcel analysis
    pub fn convective_parcel_analysis(&self) -> Option<&ParcelAscentAnalysis> {
        self.convective.as_ref()
    }

    /// Get the effective parcel analysis
    pub fn effective_parcel_analysis(&self) -> Option<&ParcelAscentAnalysis> {
        self.effective.as_ref()
    }

    /// Get the downburst profile
    pub fn downburst_profile(&self) -> Option<&ParcelProfile> {
        self.downburst_profile.as_ref()
    }

    /// Set the provider analysis.
    ///
    /// This is just a table of what ever values you want to store, it may be empty.
    pub fn with_provider_analysis(self, provider_analysis: HashMap<&'static str, f64>) -> Self {
        Analysis {
            provider_analysis,
            ..self
        }
    }

    /// Get a reference to the provider analysis so you can query it.
    #[allow(dead_code)]
    pub fn provider_analysis(&self) -> &HashMap<&'static str, f64> {
        &self.provider_analysis
    }

    /// Get a reference to the sounding.
    pub fn sounding(&self) -> &Sounding {
        &self.sounding
    }

    /// Analyze the sounding to get as much information as you can.
    pub fn fill_in_missing_analysis_mut(&mut self) {
        self.precipitable_water = self
            .precipitable_water
            .or_else(|| Optioned::from(precipitable_water(&self.sounding).ok()));

        self.haines = self
            .haines
            .or_else(|| Optioned::from(haines(&self.sounding).ok()));
        self.haines_low = self
            .haines_low
            .or_else(|| Optioned::from(haines_low(&self.sounding).ok()));
        self.haines_mid = self
            .haines_mid
            .or_else(|| Optioned::from(haines_mid(&self.sounding).ok()));
        self.haines_high = self
            .haines_high
            .or_else(|| Optioned::from(haines_high(&self.sounding).ok()));
        self.hdw = self
            .hdw
            .or_else(|| Optioned::from(hot_dry_windy(&self.sounding).ok()));

        if self.dcape.is_none() || self.downrush_t.is_none() || self.downburst_profile.is_none() {
            let result = dcape(&self.sounding);

            if let Ok((pp, dcape, down_t)) = result {
                self.dcape = some(dcape);
                self.downrush_t = some(down_t);
                self.downburst_profile = Some(pp);
            }
        }

        if self.mixed_layer.is_none() {
            self.mixed_layer = match mixed_layer_parcel(&self.sounding) {
                Ok(parcel) => lift_parcel(parcel, &self.sounding).ok(),
                Err(_) => None,
            };
        }
        if self.most_unstable.is_none() {
            self.most_unstable = match most_unstable_parcel(&self.sounding) {
                Ok(parcel) => lift_parcel(parcel, &self.sounding).ok(),
                Err(_) => None,
            };
        }
        if self.surface.is_none() {
            self.surface = match surface_parcel(&self.sounding) {
                Ok(parcel) => lift_parcel(parcel, &self.sounding).ok(),
                Err(_) => None,
            };
        }
        if self.convective.is_none() {
            self.convective = robust_convective_parcel_ascent(&self.sounding).ok();
        }

        // Convective T
        if self.convective_t.is_none() {
            self.convective_t = self
                .convective
                .as_ref()
                .map(|parcel_anal| parcel_anal.parcel().temperature)
                .into();
        }

        // Fill in the model derived precip type.
        if self.provider_wx_code.is_none() {
            if let Some(code) = self
                .provider_analysis
                .get("WxSymbolCode")
                .map(|val| PrecipType::from(*val as u8))
            {
                let conv_precip = self.provider_1hr_convective_precip().into_option();
                let total_precip = self.provider_1hr_precip().into_option();
                let visby = self.provider_vis().into_option();
                self.provider_wx_code = Some(sounding_analysis::check_precip_type_intensity(
                    code,
                    total_precip,
                    conv_precip,
                    visby,
                ));
            }
        }

        // Fill in the bourgouin derived precip type.
        if self.bourgouin_wx_code.is_none() {
            let code = sounding_analysis::bourgouin_precip_type(self.sounding())
                .unwrap_or(PrecipType::Unknown);
            let total_precip = self.provider_1hr_precip().into_option();
            let conv_precip = self.provider_1hr_convective_precip().into_option();
            let visby = self.provider_vis().into_option();
            self.bourgouin_wx_code = Some(sounding_analysis::check_precip_type_intensity(
                code,
                total_precip,
                conv_precip,
                visby,
            ));
        }

        // Fill in the NSSL derived precip type.
        if self.nssl_wx_code.is_none() {
            let code =
                sounding_analysis::nssl_precip_type(self.sounding()).unwrap_or(PrecipType::Unknown);
            let total_precip = self.provider_1hr_precip().into_option();
            let conv_precip = self.provider_1hr_convective_precip().into_option();
            let visby = self.provider_vis().into_option();
            self.nssl_wx_code = Some(sounding_analysis::check_precip_type_intensity(
                code,
                total_precip,
                conv_precip,
                visby,
            ));
        }

        // Left and right mover storm motion
        if self.right_mover.is_none() || self.left_mover.is_none() {
            let (rm, lm) = match bunkers_storm_motion(&self.sounding) {
                Ok((rm, lm)) => (some(rm), some(lm)),
                Err(_) => (none(), none()),
            };

            self.right_mover = rm;
            self.left_mover = lm;
        }

        // Fill in the mean wind
        if self.mean_wind.is_none() {
            if let Some(layer) = &sounding_analysis::layer_agl(&self.sounding, Meters(6000.0)).ok()
            {
                self.mean_wind = Optioned::from(mean_wind(layer, &self.sounding).ok());
            }
        }

        // Fill in the storm relative helicity
        if self.sr_helicity_3k_rm.is_none() || self.sr_helicity_3k_lm.is_none() {
            if let (Some(layer), Some(sm), Some(lm)) = (
                &sounding_analysis::layer_agl(&self.sounding, Meters(3000.0)).ok(),
                self.right_mover.into_option(),
                self.left_mover.into_option(),
            ) {
                self.sr_helicity_3k_rm =
                    Optioned::from(sr_helicity(layer, sm, &self.sounding()).ok());

                self.sr_helicity_3k_lm =
                    Optioned::from(sr_helicity(layer, lm, &self.sounding()).ok());
            }
        }

        // Fill in the effective inflow layer
        if self.effective_inflow_layer.is_none() {
            self.effective_inflow_layer = effective_inflow_layer(&self.sounding());
        }

        // Fill in the effective storm relative helicity
        if self.sr_helicity_eff_rm.is_none() || self.sr_helicity_eff_lm.is_none() {
            if let (Some(layer), Some(sm), Some(lm)) = (
                &self.effective_inflow_layer,
                self.right_mover.into_option(),
                self.left_mover.into_option(),
            ) {
                self.sr_helicity_eff_rm =
                    Optioned::from(sr_helicity(layer, sm, &self.sounding()).ok());

                self.sr_helicity_eff_lm =
                    Optioned::from(sr_helicity(layer, lm, &self.sounding()).ok());
            }
        }

        // Fill in the effective layer parcel analysis
        if self.effective.is_none() && self.effective_inflow_layer.is_some() {
            self.effective =
                match average_parcel(&self.sounding, &self.effective_inflow_layer.unwrap()) {
                    Ok(parcel) => lift_parcel(parcel, &self.sounding).ok(),
                    Err(_) => None,
                };
        }

        // Fill in the experimental fire weather parameters.
        if self.el_blow_up_dt_low.is_none()
            || self.top_blow_up_dt_low.is_none()
            || self.el_blow_up_height_low.is_none()
            || self.top_blow_up_height_low.is_none()
            || self.blow_up_anal_start_parcel.is_none()
            || self.lcl_dt_low.is_none()
        {
            let blow_up_anal = blow_up(self.sounding(), Some(8.0)).ok();
            let (starting_pcl, dt_el, height_el, dt_top, height_top, lcl_dt_low) = blow_up_anal
                .map(|bu_anal| {
                    (
                        Some(bu_anal.starting_parcel),
                        some(bu_anal.delta_t_el),
                        some(bu_anal.delta_z_el),
                        some(bu_anal.delta_t_top),
                        some(bu_anal.delta_z_top),
                        some(bu_anal.delta_t_cloud),
                    )
                })
                .unwrap_or((None, none(), none(), none(), none(), none()));

            self.el_blow_up_dt_low = dt_el;
            self.el_blow_up_height_low = height_el;
            self.top_blow_up_dt_low = dt_top;
            self.top_blow_up_height_low = height_top;
            self.lcl_dt_low = lcl_dt_low;
            self.blow_up_anal_start_parcel = starting_pcl;
        }

        if self.el_blow_up_dt_high.is_none()
            || self.top_blow_up_dt_high.is_none()
            || self.el_blow_up_height_high.is_none()
            || self.top_blow_up_height_high.is_none()
            || self.blow_up_anal_start_parcel.is_none()
        {
            let blow_up_anal = blow_up(self.sounding(), Some(12.0)).ok();
            let (starting_pcl, dt_el, height_el, dt_top, height_top) = blow_up_anal
                .map(|bu_anal| {
                    (
                        Some(bu_anal.starting_parcel),
                        some(bu_anal.delta_t_el),
                        some(bu_anal.delta_z_el),
                        some(bu_anal.delta_t_top),
                        some(bu_anal.delta_z_top),
                    )
                })
                .unwrap_or((None, none(), none(), none(), none()));

            self.el_blow_up_dt_high = dt_el;
            self.el_blow_up_height_high = height_el;
            self.top_blow_up_dt_high = dt_top;
            self.top_blow_up_height_high = height_top;
            self.blow_up_anal_start_parcel = starting_pcl;
        }

        // blow_up_anal_start_parcel is needed for taking the plume ascent parcels' temperature
        // values into a delta T.
        if self.plumes_low.is_none() && self.blow_up_anal_start_parcel.is_some() {
            self.plumes_low = calc_plumes(
                self.sounding(),
                CelsiusDiff(0.1),
                CelsiusDiff(20.0),
                Some(8.0),
            )
            .ok();
        }

        // FIXME: 8.0 for low and 12.0 for high should be in constants.
        if self.plumes_high.is_none() && self.blow_up_anal_start_parcel.is_some() {
            self.plumes_high = calc_plumes(
                self.sounding(),
                CelsiusDiff(0.1),
                CelsiusDiff(20.0),
                Some(12.0),
            )
            .ok();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Intensity {
    Light,
    Moderate,
    Heavy,
}

#[derive(Clone, Copy, Debug)]
pub enum PrecipTypeAlgorithm {
    Model,
    Bourgouin,
    NSSL,
}
