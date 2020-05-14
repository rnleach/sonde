use crate::analysis::Analysis;
use metfor::{Celsius, CelsiusDiff};
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
        profile_dry1: ParcelProfile,
        profile_dry2: ParcelProfile,
        profile_dry3: ParcelProfile,
        profile_dry4: ParcelProfile,
        plume_anal_dry1: PlumeAscentAnalysis,
        plume_anal_dry2: PlumeAscentAnalysis,
        plume_anal_dry3: PlumeAscentAnalysis,
        plume_anal_dry4: PlumeAscentAnalysis,
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
        create_plume_parcel_from(parcel_env, target_t - parcel_env.temperature, Some(15.0));
    let (profile_high, plume_anal_high) = match lift_plume_parcel(parcel_high, anal.sounding()) {
        Ok(high_vals) => high_vals,
        Err(_) => return Sample::None,
    };

    let parcel_dry1 = create_plume_parcel_from(parcel_env, target_t - parcel_env.temperature, None);
    let (profile_dry1, plume_anal_dry1) = match lift_plume_parcel(parcel_dry1, anal.sounding()) {
        Ok(dry_vals) => dry_vals,
        Err(_) => return Sample::None,
    };

    let parcel_dry2 = create_plume_parcel_from(
        parcel_env,
        target_t + CelsiusDiff(2.0) - parcel_env.temperature,
        None,
    );
    let (profile_dry2, plume_anal_dry2) = match lift_plume_parcel(parcel_dry2, anal.sounding()) {
        Ok(dry_vals) => dry_vals,
        Err(_) => return Sample::None,
    };

    let parcel_dry3 = create_plume_parcel_from(
        parcel_env,
        target_t + CelsiusDiff(4.0) - parcel_env.temperature,
        None,
    );
    let (profile_dry3, plume_anal_dry3) = match lift_plume_parcel(parcel_dry3, anal.sounding()) {
        Ok(dry_vals) => dry_vals,
        Err(_) => return Sample::None,
    };

    let parcel_dry4 = create_plume_parcel_from(
        parcel_env,
        target_t + CelsiusDiff(6.0) - parcel_env.temperature,
        None,
    );
    let (profile_dry4, plume_anal_dry4) = match lift_plume_parcel(parcel_dry4, anal.sounding()) {
        Ok(dry_vals) => dry_vals,
        Err(_) => return Sample::None,
    };

    Sample::FirePlume {
        parcel_low,
        profile_low,
        plume_anal_low,
        parcel_high,
        profile_high,
        plume_anal_high,
        profile_dry1,
        profile_dry2,
        profile_dry3,
        profile_dry4,
        plume_anal_dry1,
        plume_anal_dry2,
        plume_anal_dry3,
        plume_anal_dry4,
    }
}
