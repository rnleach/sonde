use crate::analysis::Analysis;
use metfor::{GigaWatts, Kelvin, Quantity};
use sounding_analysis::{
    lift_parcel, DataRow, Parcel, ParcelAscentAnalysis,
    experimental::fire_briggs::{plume_ascent_analysis, PlumeAscentAnalysis},
};

use itertools::{izip, Itertools};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Sample {
    Sounding {
        data: DataRow,
        pcl_anal: Option<ParcelAscentAnalysis>,
    },
    FirePlume {
        plume_anal_low: PlumeAscentAnalysis,
        plume_anal_high: PlumeAscentAnalysis,
        fire_power: GigaWatts,
    },
    None,
}

pub fn create_sample_sounding(data: DataRow, anal: &Analysis) -> Sample {
    let pcl_anal =
        Parcel::from_datarow(data).and_then(|pcl| lift_parcel(pcl, anal.sounding()).ok());

    Sample::Sounding { data, pcl_anal }
}

pub fn create_sample_plume(fire_power: GigaWatts, anal: &Analysis) -> Sample {

    if let (Some(bfpa_low), Some(bfpa_high)) = (anal.briggs_plume_heating_low(), anal.briggs_plume_heating_high()) {

        let plume_anal_low = izip!(bfpa_low.fire_power.iter(), bfpa_low.betas.iter())
            .tuple_windows::<(_,_)>()
            .filter(|((&fp0, beta0),(&fp1, beta1))| fp0 <= fire_power && fp1 >= fire_power)
            .fold((None), |acc, ((fp0, beta0),(fp1, beta1))| {
                let rise = beta1 - beta0;
                let run = fp1.unpack() - fp0.unpack();
                let dx = fire_power.unpack() - fp0.unpack();
                let beta_x = beta0 + dx * rise / run;

                plume_ascent_analysis(
                    bfpa_low.starting_theta,
                    bfpa_low.starting_sh,
                    beta_x,
                    bfpa_low.moisture_ratio,
                    bfpa_low.sfc_height,
                    bfpa_low.p_sfc,
                    anal.sounding())
            });

        let plume_anal_high = izip!(bfpa_high.fire_power.iter(), bfpa_high.betas.iter())
            .tuple_windows::<(_,_)>()
            .filter(|((&fp0, beta0),(&fp1, beta1))| fp0 <= fire_power && fp1 >= fire_power)
            .fold(None, |acc, ((fp0, beta0),(fp1, beta1))| {
                let rise = beta1 - beta0;
                let run = fp1.unpack() - fp0.unpack();
                let dx = fire_power.unpack() - fp0.unpack();
                let beta_x = beta0 + dx * rise / run;

                plume_ascent_analysis(
                    bfpa_high.starting_theta,
                    bfpa_high.starting_sh,
                    beta_x,
                    bfpa_high.moisture_ratio,
                    bfpa_high.sfc_height,
                    bfpa_high.p_sfc,
                    anal.sounding())
            });

        if let (Some(plume_anal_low), Some(plume_anal_high)) = (plume_anal_low, plume_anal_high) {
            Sample::FirePlume {
                plume_anal_low,
                plume_anal_high,
                fire_power,
            }
        } else {
            Sample::None
        }
    } else {
        Sample::None
    }
}
