//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.
use crate::{
    coords::{SDCoords, TPCoords, WPCoords, XYCoords, XYRect},
    errors::SondeError,
    gui::{
        self,
        profiles::{CloudContext, RHOmegaContext, WindSpeedContext},
        HodoContext, PlotContext, PlotContextExt, SkewTContext,
    },
};
use itertools::izip;
use metfor::Quantity;
use sounding_analysis::{self, Analysis};
use sounding_base::DataRow;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

// Module for configuring application
pub mod config;
use self::config::Config;

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
    list: RefCell<Vec<Rc<RefCell<Analysis>>>>,
    analyzed_count: Cell<usize>,
    currently_displayed_index: Cell<usize>,
    last_sample: Cell<Option<DataRow>>,

    // Handle to the GUI
    gui: gtk::Builder,

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
    pub fn initialize() -> AppContextPointer {
        let glade_src = include_str!("./sonde.glade");

        Rc::new(AppContext {
            config: RefCell::new(Config::default()),
            source_description: RefCell::new(None),
            list: RefCell::new(vec![]),
            analyzed_count: Cell::new(0),
            currently_displayed_index: Cell::new(0),
            last_sample: Cell::new(None),
            gui: gtk::Builder::new_from_string(glade_src),
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

    pub fn load_data<I>(acp: AppContextPointer, src: I)
    where
        I: Iterator<Item = Analysis>,
    {
        // Copy in the list and make sure it is sorted.
        {
            let list: &mut Vec<_> = &mut acp.list.borrow_mut();
            *list = src.map(RefCell::new).map(Rc::new).collect();
            list.sort_by_key(|anal| anal.borrow().sounding().valid_time());
        }

        acp.currently_displayed_index.set(0);
        *acp.source_description.borrow_mut() = None;

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

        for anal in acp.list.borrow().iter() {
            let anal = anal.borrow();
            let snd = anal.sounding();
            for pair in snd
                .pressure_profile()
                .iter()
                .zip(snd.temperature_profile())
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
                })
            {
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
                .pressure_profile()
                .iter()
                .zip(snd.dew_point_profile())
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
                })
            {
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
                .pressure_profile()
                .iter()
                .zip(snd.pvv_profile())
                .filter_map(|(p, omega)| {
                    if let (Some(p), Some(o)) = (p.into_option(), omega.into_option()) {
                        if p < config::MINP {
                            None
                        } else {
                            Some(WPCoords {
                                w: o.abs().max(config::MIN_ABS_W),
                                p,
                            })
                        }
                    } else {
                        None
                    }
                })
            {
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

            for pair in izip!(snd.pressure_profile(), snd.wind_profile()).filter_map(|(p, wind)| {
                if let (Some(p), Some(w)) = (p.into_option(), wind.into_option()) {
                    if p < acp.config.borrow().min_hodo_pressure {
                        None
                    } else {
                        Some(SDCoords { spd_dir: w })
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

        acp.skew_t.set_xy_envelope(skew_t_xy_envelope);
        acp.hodo.set_xy_envelope(hodo_xy_envelope);

        acp.rh_omega.set_xy_envelope(rh_omega_xy_envelope);

        let mut cloud_envelope = rh_omega_xy_envelope;
        cloud_envelope.lower_left.x = 0.0;
        cloud_envelope.upper_right.x = 1.0;
        acp.cloud.set_xy_envelope(cloud_envelope);

        acp.fit_to_data();
        acp.set_currently_displayed(0);

        // Once everything we need for this thread is taken care of, fill in any missing data
        // in the analysis.
        let pool = threadpool::ThreadPool::default();
        let (tx, rx) = crossbeam_channel::unbounded();

        for (i, anal) in acp.list.borrow().iter().enumerate() {
            let anal = anal.borrow().clone();
            let tx = tx.clone();
            pool.execute(move || {
                let mut anal = anal;
                anal.fill_in_missing_analysis_mut();
                tx.send((i, anal)).unwrap();
            });
        }

        let acp = Rc::clone(&acp);
        acp.analyzed_count.set(0);
        gtk::idle_add(move || {
            for (i, anal) in rx.try_iter() {
                *(*acp).list.borrow_mut()[i].borrow_mut() = anal;
                acp.analyzed_count.set(acp.analyzed_count.get() + 1);

                if (*acp).currently_displayed_index.get() == i {
                    acp.mark_data_dirty();
                    acp.update_all_gui();
                }
            }

            if acp.analyzed_count.get() == acp.list.borrow().len() {
                glib::Continue(false)
            } else {
                glib::Continue(true)
            }
        });
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
        self.update_sample();
        self.mark_data_dirty();
        self.update_all_gui();
    }

    fn update_sample(&self) {
        if let Some(sample) = self.last_sample.get() {
            if let Some(p) = sample.pressure.into() {
                self.last_sample.set(
                    self.list
                        .borrow()
                        .get(self.currently_displayed_index.get())
                        .and_then(|anal| {
                            sounding_analysis::linear_interpolate_sounding(
                                anal.borrow().sounding(),
                                p,
                            )
                            .ok()
                        }),
                );
            } else {
                self.last_sample.set(None);
            }
        }
        self.mark_overlay_dirty();
    }

    // Update all the gui elements
    fn update_all_gui(&self) {
        gui::draw_all(&self);
        gui::update_text_views(&self);
    }

    /// Get the analysis for drawing, etc.
    pub fn get_sounding_for_display(&self) -> Option<Rc<RefCell<Analysis>>> {
        self.list
            .borrow()
            .get(self.currently_displayed_index.get())
            .map(Rc::clone)
    }

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
}
