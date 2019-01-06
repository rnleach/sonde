//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use sounding_analysis;
use sounding_analysis::Analysis;
use sounding_base::{DataRow, Sounding};

use crate::coords::{SDCoords, TPCoords, WPCoords, XYCoords, XYRect};
use crate::gui::profiles::{CloudContext, RHOmegaContext, WindSpeedContext};
use crate::gui::{self, HodoContext, PlotContext, PlotContextExt, SkewTContext};

use glib;
use gtk::Builder;

// Module for configuring application
pub mod config;
use self::config::Config;

use crate::errors::SondeError;

/// Smart pointer for globally shareable data
pub type AppContextPointer = Rc<AppContext>;

/// Holds the application state. This is a singleton (not enforced) that is shared globally.
pub struct AppContext {
    // Configuration, style and layout settings.
    pub config: RefCell<Config>,

    // Source description is used in the legend if it is present. Not all file formats include a
    // station name or model name or base time. In bufkit files this is usually part of the file
    // name. So whatever function loads a sounding should set this to reflect where it came from.
    source_description: RefCell<Option<String>>,

    // Lists of soundings and currently displayed one
    list: RefCell<Vec<Rc<Analysis>>>,
    extra_profiles: RefCell<Vec<Rc<ExtraProfiles>>>,
    currently_displayed_index: Cell<usize>,
    last_sample: Cell<Option<DataRow>>,

    // Handle to the GUI
    gui: Builder,

    // Handle to skew-t context
    pub skew_t: SkewTContext,

    // Handle to Hodograph context
    pub hodo: HodoContext,

    // Handle to RH Omega Context
    pub rh_omega: RHOmegaContext,

    // Handle to Cloud profile context
    pub cloud: CloudContext,

    // Handle to wind speed profile context
    pub wind_speed: WindSpeedContext,
}

impl AppContext {
    /// Create a new instance of AppContext and return a smart pointer to it.
    ///
    /// Note: It is important at a later time to call set_gui, otherwise nothing will ever be
    /// drawn on the GUI.
    pub fn new() -> AppContextPointer {
        let glade_src = include_str!("../sonde.glade");

        Rc::new(AppContext {
            config: RefCell::new(Config::default()),
            source_description: RefCell::new(None),
            list: RefCell::new(vec![]),
            extra_profiles: RefCell::new(vec![]),
            currently_displayed_index: Cell::new(0),
            last_sample: Cell::new(None),
            gui: Builder::new_from_string(glade_src),
            skew_t: SkewTContext::new(),
            rh_omega: RHOmegaContext::new(),
            cloud: CloudContext::new(),
            hodo: HodoContext::new(),
            wind_speed: WindSpeedContext::new(),
        })
    }

    pub fn fetch_widget<T>(&self, widget_id: &'static str) -> Result<T, SondeError>
    where
        T: glib::IsA<glib::Object>,
    {
        self.gui
            .get_object(widget_id)
            .ok_or_else(|| SondeError::WidgetLoadError(widget_id))
    }

    pub fn load_data(&self, src: &mut Iterator<Item = Analysis>) {
        use crate::app::config;
        use sounding_base::Profile::*;

        *self.list.borrow_mut() = src
            .into_iter()
            .map(|anal| Rc::new(anal.fill_in_missing_analysis()))
            .collect();
        *self.extra_profiles.borrow_mut() = self
            .list
            .borrow()
            .iter()
            .map(|anal| ExtraProfiles::new(anal.sounding()))
            .map(Rc::new)
            .collect();
        self.currently_displayed_index.set(0);
        *self.source_description.borrow_mut() = None;

        let mut skew_t_xy_envelope = XYRect {
            lower_left: XYCoords { x: 0.45, y: 0.45 },
            upper_right: XYCoords { x: 0.55, y: 0.55 },
        };

        let mut rh_omega_xy_envelope = XYRect {
            lower_left: XYCoords { x: 0.45, y: 0.45 },
            upper_right: XYCoords { x: 0.55, y: 0.55 },
        };

        let mut hodo_xy_envelope = XYRect {
            lower_left: XYCoords { x: 0.45, y: 0.45 },
            upper_right: XYCoords { x: 0.55, y: 0.55 },
        };

        let snd_list = self.list.borrow();
        let snd_list = snd_list.iter().map(|anal| anal.sounding());

        for snd in snd_list {
            for pair in snd
                .get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(Temperature))
                .filter_map(|(p, t)| {
                    if let (Some(p), Some(t)) = (p.into(), t.into()) {
                        if p < config::MINP {
                            None
                        } else {
                            Some(TPCoords {
                                temperature: t,
                                pressure: p,
                            })
                        }
                    } else {
                        None
                    }
                }) {
                let XYCoords { x, y } = SkewTContext::convert_tp_to_xy(pair);
                if x < skew_t_xy_envelope.lower_left.x {
                    skew_t_xy_envelope.lower_left.x = x;
                }
                if y < skew_t_xy_envelope.lower_left.y {
                    skew_t_xy_envelope.lower_left.y = y;
                    rh_omega_xy_envelope.lower_left.y = y;
                }
                if x > skew_t_xy_envelope.upper_right.x {
                    skew_t_xy_envelope.upper_right.x = x;
                }
                if y > skew_t_xy_envelope.upper_right.y {
                    skew_t_xy_envelope.upper_right.y = y;
                    rh_omega_xy_envelope.upper_right.y = y;
                }
            }

            for pair in snd
                .get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(DewPoint))
                .filter_map(|(p, t)| {
                    if let (Some(p), Some(t)) = (p.into(), t.into()) {
                        if p < config::MINP {
                            None
                        } else {
                            Some(TPCoords {
                                temperature: t,
                                pressure: p,
                            })
                        }
                    } else {
                        None
                    }
                }) {
                let XYCoords { x, y } = SkewTContext::convert_tp_to_xy(pair);
                if x < skew_t_xy_envelope.lower_left.x {
                    skew_t_xy_envelope.lower_left.x = x;
                }
                if y < skew_t_xy_envelope.lower_left.y {
                    skew_t_xy_envelope.lower_left.y = y;
                    rh_omega_xy_envelope.lower_left.y = y;
                }
                if x > skew_t_xy_envelope.upper_right.x {
                    skew_t_xy_envelope.upper_right.x = x;
                }
                if y > skew_t_xy_envelope.upper_right.y {
                    skew_t_xy_envelope.upper_right.y = y;
                    rh_omega_xy_envelope.upper_right.y = y;
                }
            }

            for pair in snd
                .get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(PressureVerticalVelocity))
                .filter_map(|(p, omega)| {
                    if let (Some(p), Some(o)) = (p.into(), omega.into()) {
                        if p < config::MINP {
                            None
                        } else {
                            Some(WPCoords {
                                w: { f64::max(f64::abs(o), config::MIN_ABS_W) },
                                p,
                            })
                        }
                    } else {
                        None
                    }
                }) {
                let XYCoords { x, y: _y } = RHOmegaContext::convert_wp_to_xy(pair);
                if x > rh_omega_xy_envelope.upper_right.x {
                    rh_omega_xy_envelope.upper_right.x = x;
                }
                let pair = WPCoords {
                    w: -pair.w,
                    p: pair.p,
                };
                let XYCoords { x, y: _y } = RHOmegaContext::convert_wp_to_xy(pair);
                if x < rh_omega_xy_envelope.lower_left.x {
                    rh_omega_xy_envelope.lower_left.x = x;
                }
            }

            for pair in izip!(
                snd.get_profile(Pressure),
                snd.get_profile(WindSpeed),
                snd.get_profile(WindDirection)
            ).filter_map(|(p, ws, wd)| {
                if let (Some(p), Some(s), Some(d)) =
                    (Into::<Option<f64>>::into(p), ws.into(), wd.into())
                {
                    if p < self.config.borrow().min_hodo_pressure {
                        None
                    } else {
                        Some(SDCoords { speed: s, dir: d })
                    }
                } else {
                    None
                }
            }) {
                let XYCoords { x, y } = HodoContext::convert_sd_to_xy(pair);
                if x < hodo_xy_envelope.lower_left.x {
                    hodo_xy_envelope.lower_left.x = x;
                }
                if y < hodo_xy_envelope.lower_left.y {
                    hodo_xy_envelope.lower_left.y = y;
                }
                if x > hodo_xy_envelope.upper_right.x {
                    hodo_xy_envelope.upper_right.x = x;
                }
                if y > hodo_xy_envelope.upper_right.y {
                    hodo_xy_envelope.upper_right.y = y;
                }
            }
        }

        self.skew_t.set_xy_envelope(skew_t_xy_envelope);
        self.hodo.set_xy_envelope(hodo_xy_envelope);

        self.rh_omega.set_xy_envelope(rh_omega_xy_envelope);

        let mut cloud_envelope = rh_omega_xy_envelope;
        cloud_envelope.lower_left.x = 0.0;
        cloud_envelope.upper_right.x = 1.0;
        self.cloud.set_xy_envelope(cloud_envelope);

        self.fit_to_data();
        self.set_currently_displayed(0);
    }

    /// Is there any data to plot?
    pub fn plottable(&self) -> bool {
        !self.list.borrow().is_empty()
    }

    /// Set the next one as the one to display, or wrap to the beginning.
    pub fn display_next(&self) {
        if self.plottable() {
            let mut curr_index = self.currently_displayed_index.get();
            if curr_index < self.list.borrow().len() - 1 {
                curr_index += 1;
            } else {
                curr_index = 0;
            }
            self.set_currently_displayed(curr_index);
        }
    }

    /// Set the previous one as the one to display, or wrap to the end.
    pub fn display_previous(&self) {
        if self.plottable() {
            let mut curr_index = self.currently_displayed_index.get();
            if curr_index > 0 {
                curr_index -= 1;
            } else {
                curr_index = self.list.borrow().len() - 1;
            }
            self.set_currently_displayed(curr_index);
        }
    }

    #[inline]
    fn set_currently_displayed(&self, idx: usize) {
        self.currently_displayed_index.set(idx);

        if log_enabled!(::log::Level::Trace) {
            self.log_anal_summary();
        }

        self.update_sample();
        self.mark_data_dirty();
        self.update_all_gui();
    }

    fn update_sample(&self) {
        if let Some(sample) = self.last_sample.get() {
            if let Some(p) = sample.pressure.into() {
                self.last_sample.set(
                    ::sounding_analysis::linear_interpolate_sounding(
                        self.list.borrow()[self.currently_displayed_index.get()].sounding(),
                        p,
                    ).ok(),
                );
            } else {
                self.last_sample.set(None);
            }
        }
        self.mark_overlay_dirty();
    }

    // Update all the gui elements
    pub fn update_all_gui(&self) {
        gui::draw_all(&self);
        gui::update_text_views(&self);
    }

    /// Get the sounding to draw.
    pub fn get_sounding_for_display(&self) -> Option<Rc<Analysis>> {
        if self.plottable() {
            let shared_ptr = Rc::clone(&self.list.borrow()[self.currently_displayed_index.get()]);
            Some(shared_ptr)
        } else {
            None
        }
    }

    /// Get the extra profiles to draw.
    // pub fn get_extra_profiles_for_display(&self) -> Option<Rc<ExtraProfiles>> {
    //     if self.plottable() {
    //         let shared_ptr =
    //             Rc::clone(&self.extra_profiles.borrow()[self.currently_displayed_index.get()]);
    //         Some(shared_ptr)
    //     } else {
    //         None
    //     }
    // }

    /// Set the source name
    pub fn set_source_description(&self, new_name: Option<String>) {
        *self.source_description.borrow_mut() = new_name;
    }

    /// Get the source name
    pub fn get_source_description(&self) -> Option<String> {
        match *self.source_description.borrow() {
            Some(ref name) => Some(name.clone()),
            None => None,
        }
    }

    /// Fit to the given x-y max coords. SHOULD NOT BE PUBLIC - DO NOT USE IN DRAWING CALLBACKS.
    fn fit_to_data(&self) {
        self.skew_t.zoom_to_envelope();
        self.hodo.zoom_to_envelope();
        self.rh_omega.zoom_to_envelope();
        self.cloud.zoom_to_envelope();
        self.wind_speed.zoom_to_envelope();
        self.mark_background_dirty();
    }

    pub fn get_sample(&self) -> Option<DataRow> {
        self.last_sample.get()
    }

    pub fn set_sample<T>(&self, sample: T)
    where
        Option<DataRow>: From<T>,
    {
        self.last_sample.set(Option::from(sample));
        gui::update_text_highlight(&self);
        self.mark_overlay_dirty();
    }

    pub fn mark_data_dirty(&self) {
        self.hodo.mark_data_dirty();
        self.skew_t.mark_data_dirty();
        self.rh_omega.mark_data_dirty();
        self.cloud.mark_data_dirty();
        self.wind_speed.mark_data_dirty();
    }

    pub fn mark_overlay_dirty(&self) {
        self.hodo.mark_overlay_dirty();
        self.skew_t.mark_overlay_dirty();
        self.rh_omega.mark_overlay_dirty();
        self.cloud.mark_overlay_dirty();
        self.wind_speed.mark_overlay_dirty();
    }

    pub fn mark_background_dirty(&self) {
        self.hodo.mark_background_dirty();
        self.skew_t.mark_background_dirty();
        self.rh_omega.mark_background_dirty();
        self.cloud.mark_background_dirty();
        self.wind_speed.mark_background_dirty();
    }

    fn log_anal_summary(&self) {
        use sounding_analysis::ParcelIndex::*;

        const IDX_ENUM: sounding_analysis::ParcelIndex = CAPE;
        const IDX_KEY: &str = "CAPE";

        let anal = if let Some(anal) = self.get_sounding_for_display() {
            anal
        } else {
            error!("Could not retrieve analysis!");
            return;
        };

        if let Some(val) = anal.provider_analysis().get(IDX_KEY) {
            anal.get_surface_parcel_analysis().and_then(|sfc_pa| {
                anal.get_mixed_layer_parcel_analysis().and_then(|ml_pa| {
                    anal.get_most_unstable_parcel_analysis().and_then(|mu_pa| {
                        trace!(
                            "{} {} SFC: {:?} ML: {:?} MU: {:?}",
                            IDX_KEY,
                            val,
                            sfc_pa.get_index(IDX_ENUM),
                            ml_pa.get_index(IDX_ENUM),
                            mu_pa.get_index(IDX_ENUM),
                        );
                        Some(())
                    })
                })
            });

        // if let Some(my_val) = anal.get_profile_index(IDX_ENUM) {
        //     trace!(
        //         "(Bufkit,Sonde,Diff,%) => ({:5.2}, {:5.2}, {:5.2}, {:6.2}%)",
        //         val,
        //         my_val,
        //         val - my_val,
        //         (val - my_val) / val * 100.0
        //     );
        // } else {
        //  trace!("Analysis value was None for {:?}, but the provider had {}", IDX_ENUM, val);
        // }
        } else {
            trace!("Error loading {} from provider analysis.\n\n", IDX_KEY);
            for (key, val) in anal.provider_analysis().iter() {
                trace!("{} => {}", key, val);
            }
        }

        // trace!(
        //     "Provider analysis for {:?} at index {}.",
        //     self.get_source_description(),
        //     self.currently_displayed_index.get()
        // );

        // for (key, val) in anal.provider_analysis().iter() {
        //     trace!("{} => {}", key, val);
        // }
    }
}

#[derive(Debug, Default)]
pub struct ExtraProfiles {
    // pub lapse_rate: Vec<Optioned<f64>>,
// pub sfc_avg_lapse_rate: Vec<Optioned<f64>>,
}

impl ExtraProfiles {
    pub fn new(_snd: &Sounding) -> Self {
        // let lapse_rate = sounding_analysis::profile::temperature_lapse_rate(snd);
        // let sfc_avg_lapse_rate =
        //     sounding_analysis::profile::sfc_to_level_temperature_lapse_rate(snd);

        ExtraProfiles {
            // lapse_rate,
            // sfc_avg_lapse_rate,
        }
    }
}
