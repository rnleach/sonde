use crate::app::{AppContext, AppContextPointer};
use bufr_read::{BufrFile, Message};
use chrono::naive::NaiveDate;
use itertools::izip;
use metfor::{Celsius, HectoPascal, Kelvin, Knots, Meters, WindSpdDir};
use optional::Optioned;
use sounding_analysis::Analysis;
use sounding_base::{Sounding, StationInfo};
use sounding_bufkit::BufkitFile;
use std::{error::Error, path::PathBuf, rc::Rc};

pub fn load_file(path: &PathBuf, ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let extension: Option<String> = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_string());
    let extension = extension.as_ref().map(|ext| ext.as_str());

    match extension {
        Some("buf") => load_bufkit(path, ac)?,
        Some("bufr") => load_bufr(path, ac)?,
        Some(_) => unreachable!(),
        None => unreachable!(),
    }

    if let Some(name) = path.file_name() {
        let mut src_name = "File: ".to_owned();
        src_name.push_str(&name.to_string_lossy());
        ac.set_source_description(Some(src_name));
    }

    Ok(())
}

pub fn load_multiple_bufr(paths: &[PathBuf], ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let datas: Result<Vec<_>, _> = paths
        .iter()
        .map(|p| BufrFile::new(&p.to_string_lossy()))
        .collect();
    let datas: Result<Vec<_>, _> = datas?.into_iter().flat_map(|iter| iter).collect();
    let datas: Result<Vec<Analysis>, _> = datas?.into_iter().map(bufr_to_sounding).collect();
    let datas: Vec<Analysis> = datas?;

    AppContext::load_data(Rc::clone(ac), datas.into_iter());
    ac.set_source_description(Some("Multiple BUFR files".to_owned()));

    Ok(())
}

fn load_bufkit(path: &PathBuf, ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let file = BufkitFile::load(path)?;
    let data = file.data()?;

    AppContext::load_data(Rc::clone(ac), &mut data.into_iter());
    Ok(())
}

fn load_bufr(path: &PathBuf, ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let file = BufrFile::new(&path.to_string_lossy())?;

    let data: Vec<Analysis> = file
        .filter_map(|result| result.ok())
        .filter_map(|msg| bufr_to_sounding(msg).ok())
        .collect();
    AppContext::load_data(Rc::clone(ac), data.into_iter());

    Ok(())
}

fn bufr_to_sounding(msg: Message) -> Result<Analysis, Box<dyn Error>> {
    let pressure_vals = msg.double_array("pressure")?;
    let pressure_vals = pressure_vals
        .iter()
        .map(|p| p.map_t(|p| p / 100.0).map_t(HectoPascal));

    let height = msg.double_array("nonCoordinateGeopotentialHeight")?;
    let height = height.iter().map(|v| v.map_t(Meters));

    let temperature = msg.double_array("airTemperature")?;
    let temperature = temperature
        .iter()
        .map(|t| t.map_t(|t| Celsius::from(Kelvin(t))));

    let dpt = msg.double_array("dewpointTemperature")?;
    let dpt = dpt
        .iter()
        .map(|dp| dp.map_t(|dp| Celsius::from(Kelvin(dp))));

    let wspeed = msg.double_array("windSpeed")?;
    let wdir = msg.double_array("windDirection")?;
    let wind = izip!(wdir, wspeed).map(|(d_opt, s_opt)| {
        d_opt.and_then(|d| {
            s_opt.map_t(|s| WindSpdDir {
                speed: Knots(s),
                direction: d,
            })
        })
    });

    let mut p: Vec<Optioned<HectoPascal>> = vec![];
    let mut h: Vec<Optioned<Meters>> = vec![];
    let mut t: Vec<Optioned<Celsius>> = vec![];
    let mut dp: Vec<Optioned<Celsius>> = vec![];
    let mut w: Vec<Optioned<WindSpdDir<Knots>>> = vec![];

    let mut p0 = HectoPascal(std::f64::MAX);
    let mut z0 = Meters(-std::f64::MAX);

    for (p1, h1, t1, dp1, w1) in izip!(pressure_vals, height, temperature, dpt, wind) {
        if let (Some(p_val), Some(z_val)) = (p1.into(), h1.into()) {
            if p_val <= p0 && z_val > z0 {
                p.push(p1);
                h.push(h1);
                t.push(t1);
                dp.push(dp1);
                w.push(w1);

                p0 = p_val;
                z0 = z_val;
            }
        }
    }

    let latitude = msg
        .double("latitude")
        .ok()
        .and_then(|val| val.into_option());
    let longitude = msg
        .double("longitude")
        .ok()
        .and_then(|val| val.into_option());
    let elevation = msg
        .double("heightOfStationGroundAboveMeanSeaLevel")
        .ok()
        .and_then(|val| val.into_option())
        .map(Meters);
    let lat_lon = latitude.and_then(|lat| longitude.map(|lon| (lat, lon)));
    let stn = StationInfo::new_with_values(None, lat_lon, elevation);

    let year = msg.long("year").ok().and_then(|v| v.into_option());
    let month = msg.long("month").ok().and_then(|v| v.into_option());
    let day = msg.long("day").ok().and_then(|v| v.into_option());
    let hour = msg.long("hour").ok().and_then(|v| v.into_option());

    let vt = year
        .and_then(|y| month.map(|m| (y as i32, m as u32)))
        .and_then(|(y, m)| day.map(|d| (y, m, d as u32)))
        .and_then(|(y, m, d)| hour.map(|h| (y, m, d, h as u32)))
        .map(|(y, m, d, h)| NaiveDate::from_ymd(y, m, d).and_hms(h, 0, 0));

    let snd = Sounding::new()
        .with_station_info(stn)
        .with_valid_time(vt)
        .with_lead_time(0) // Lead time in hours for forecast soundings.
        .with_pressure_profile(p)
        .with_height_profile(h)
        .with_temperature_profile(t)
        .with_dew_point_profile(dp)
        .with_wind_profile(w);

    Ok(Analysis::new(snd))
}
