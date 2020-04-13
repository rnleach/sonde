use crate::analysis::Analysis;
use metfor::Celsius;
use sounding_analysis::{
    experimental::fire::{create_plume_parcel_from, lift_plume_parcel, PlumeAscentAnalysis},
    lift_parcel, DataRow, Parcel, ParcelAscentAnalysis, ParcelProfile,
};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Sample {
    Sounding {
        data: DataRow,
        pcl_anal: Option<ParcelAscentAnalysis>,
    },
    FirePlume {
        parcel_low: Parcel,
        profile_low: ParcelProfile,
        plume_anal_low: PlumeAscentAnalysis,
        parcel_high: Parcel,
        profile_high: ParcelProfile,
        plume_anal_high: PlumeAscentAnalysis,
    },
    None,
}

pub fn create_sample_sounding(data: DataRow, anal: &Analysis) -> Sample {
    let pcl_anal =
        Parcel::from_datarow(data).and_then(|pcl| lift_parcel(pcl, anal.sounding()).ok());

    Sample::Sounding { data, pcl_anal }
}

pub fn create_sample_plume(parcel_env: Parcel, target_t: Celsius, anal: &Analysis) -> Sample {
    let parcel_low =
        create_plume_parcel_from(parcel_env, target_t - parcel_env.temperature, Some(8.0));
    let (profile_low, plume_anal_low) = match lift_plume_parcel(parcel_low, anal.sounding()) {
        Ok(low_vals) => low_vals,
        Err(_) => return Sample::None,
    };

    let parcel_high =
        create_plume_parcel_from(parcel_env, target_t - parcel_env.temperature, Some(12.0));
    let (profile_high, plume_anal_high) = match lift_plume_parcel(parcel_high, anal.sounding()) {
        Ok(high_vals) => high_vals,
        Err(_) => return Sample::None,
    };

    Sample::FirePlume {
        parcel_low,
        profile_low,
        plume_anal_low,
        parcel_high,
        profile_high,
        plume_anal_high,
    }
}
