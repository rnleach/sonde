use crate::analysis::Analysis;
use sounding_analysis::{
    experimental::fire::{lift_plume_parcel, PlumeAscentAnalysis},
    lift_parcel, DataRow, Parcel, ParcelAscentAnalysis, ParcelProfile,
};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Sample {
    Sounding {
        data: DataRow,
        pcl_anal: ParcelAscentAnalysis,
    },
    FirePlume {
        parcel: Parcel,
        profile: ParcelProfile,
        plume_anal: PlumeAscentAnalysis,
    },
    None,
}

pub fn create_sample_sounding(data: DataRow, anal: &Analysis) -> Sample {
    Parcel::from_datarow(data)
        .and_then(|pcl| lift_parcel(pcl, anal.sounding()).ok())
        .map(|pcl_anal| Sample::Sounding { data, pcl_anal })
        .unwrap_or(Sample::None)
}

pub fn create_sample_plume(parcel: Parcel, anal: &Analysis) -> Sample {
    lift_plume_parcel(parcel, anal.sounding())
        .ok()
        .map(|(profile, plume_anal)| Sample::FirePlume {
            parcel,
            profile,
            plume_anal,
        })
        .unwrap_or(Sample::None)
}
