use crate::{
    analysis::Analysis,
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use sonde_bufr::load_309052_sounding;
use sounding_bufkit::BufkitFile;
use std::{
    error::Error,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
};

pub fn load_multiple(paths: &[PathBuf], ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let datas: Result<Vec<_>, _> = paths.iter().map(|pb| load_file(pb)).collect();
    let mut datas: Vec<_> = datas?.into_iter().flatten().collect();

    // Sort by valid time ascending, then by lead time ascending
    datas.sort_by(|left, right| {
        if let (Some(left_vt), Some(right_vt)) =
            (left.sounding().valid_time(), right.sounding().valid_time())
        {
            if left_vt == right_vt {
                if let (Some(left_lt), Some(right_lt)) = (
                    left.sounding().lead_time().into_option(),
                    right.sounding().lead_time().into_option(),
                ) {
                    left_lt.cmp(&right_lt)
                } else {
                    std::cmp::Ordering::Equal
                }
            } else {
                left_vt.cmp(&right_vt)
            }
        } else {
            std::cmp::Ordering::Equal
        }
    });

    // Scan through the list and only pass the first value with a given valid time.
    let datas = datas
        .into_iter()
        .scan(None, |prev, anal| {
            let valid_time = anal.sounding().valid_time();
            if *prev == valid_time {
                Some(None)
            } else {
                *prev = valid_time;
                Some(Some(anal))
            }
        })
        // Get rid of the None and only let the
        .flatten();

    AppContext::load_data(Rc::clone(ac), datas);

    Ok(())
}

// Make `pub` so I can use it in benches too.
pub fn load_file(path: &Path) -> Result<Vec<Analysis>, Box<dyn Error>> {
    let extension: Option<String> = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_string());
    let extension = extension.as_deref();

    let mut load_fns = [load_bufkit, load_bufr, load_wyoming_html];

    if Some("bufr") == extension {
        // Try the bufr loader first.
        load_fns.swap(0, 1);
    }

    if Some("html") == extension {
        // Try the wyoming text list loader first.
        load_fns.swap(0, 2);
    }

    for load_fn in load_fns.iter() {
        match load_fn(path) {
            Ok(data_vec) => return Ok(data_vec),
            Err(_) => continue,
        }
    }

    Err(Box::new(SondeError::NoMatchingFileType))
}

fn load_wyoming_html(path: &Path) -> Result<Vec<Analysis>, Box<dyn Error>> {
    let mut text = String::new();

    let mut f = std::fs::File::open(path)?;
    f.read_to_string(&mut text)?;

    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown file.".to_owned());

    let data = sounding_wyoming_text_list::parse_text(&file_name, &text)
        .map(|(snd, provider_anal)| Analysis::new(snd).with_provider_analysis(provider_anal))
        .collect();

    Ok(data)
}

fn load_bufkit(path: &Path) -> Result<Vec<Analysis>, Box<dyn Error>> {
    let file = BufkitFile::load(path)?;
    let data = file
        .data()?
        .into_iter()
        .map(|(snd, provider_anal)| Analysis::new(snd).with_provider_analysis(provider_anal))
        .collect();
    Ok(data)
}

fn load_bufr(path: &Path) -> Result<Vec<Analysis>, Box<dyn Error>> {
    let mut data: Vec<Analysis> = Vec::with_capacity(1);

    let snd = load_309052_sounding(path)?;
    data.push(Analysis::new(snd));

    Ok(data)
}

