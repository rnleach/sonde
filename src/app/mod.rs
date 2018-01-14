//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.

use std::rc::Rc;
use std::cell::{Cell, RefCell};

use sounding_base::{DataRow, Sounding};

use coords::{SDCoords, TPCoords, WPCoords, XYCoords, XYRect};
use errors::*;
use gui::{CloudContext, Gui, HodoContext, PlotContext, PlotContextExt, RHOmegaContext,
          SkewTContext};

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

    list: RefCell<Vec<Rc<Sounding>>>,
    currently_displayed_index: Cell<usize>,
    last_sample: Cell<Option<DataRow>>,

    // Handle to the GUI
    pub gui: RefCell<Option<Gui>>,

    // Handle to skew-t context
    pub skew_t: SkewTContext,

    // Handle to Hodograph context
    pub hodo: HodoContext,

    // Handle to RH Omega Context
    pub rh_omega: RHOmegaContext,

    // Handle to Cloud profile context
    pub cloud: CloudContext,
}

impl AppContext {
    /// Create a new instance of AppContext and return a smart pointer to it.
    ///
    /// Note: It is important at a later time to call set_gui, otherwise nothing will ever be
    /// drawn on the GUI.
    pub fn new() -> AppContextPointer {
        Rc::new(AppContext {
            config: RefCell::new(Config::default()),
            source_description: RefCell::new(None),
            list: RefCell::new(vec![]),
            currently_displayed_index: Cell::new(0),
            last_sample: Cell::new(None),
            gui: RefCell::new(None),
            skew_t: SkewTContext::new(),
            rh_omega: RHOmegaContext::new(),
            cloud: CloudContext::new(),
            hodo: HodoContext::new(),
        })
    }

    pub fn set_gui(&self, gui: Gui) {
        *self.gui.borrow_mut() = Some(gui);
    }

    pub fn load_data(&self, src: &mut Iterator<Item = Sounding>) -> Result<()> {
        use app::config;
        use sounding_base::Profile::*;

        *self.list.borrow_mut() = src.into_iter().map(Rc::new).collect();
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

        for snd in snd_list.iter() {
            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(Temperature))
                .filter_map(|p| {
                    if let (Some(p), Some(t)) = (*p.0, *p.1) {
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

            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(DewPoint))
                .filter_map(|p| {
                    if let (Some(p), Some(t)) = (*p.0, *p.1) {
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

            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(PressureVerticalVelocity))
                .filter_map(|p| {
                    if let (Some(p), Some(o)) = (*p.0, *p.1) {
                        if p < config::MINP {
                            None
                        } else {
                            Some(WPCoords { w: o.abs(), p })
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
            ).filter_map(|tuple| {
                if let (Some(p), Some(s), Some(d)) = (*tuple.0, *tuple.1, *tuple.2) {
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
        self.rh_omega.set_xy_envelope(rh_omega_xy_envelope);
        self.hodo.set_xy_envelope(hodo_xy_envelope);
        let mut cloud_envelope = rh_omega_xy_envelope;
        cloud_envelope.lower_left.x = 0.0;
        cloud_envelope.upper_right.x = 1.0;
        self.cloud.set_xy_envelope(cloud_envelope);

        self.fit_to_data();

        if let Some(ref gui) = *self.gui.borrow() {
            gui.draw_all();
            gui.update_text_view(self);
        }

        Ok(())
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
            self.currently_displayed_index.set(curr_index);
            self.update_sample();
        }

        self.mark_data_dirty();

        self.update_all_gui();
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
            self.currently_displayed_index.set(curr_index);
            self.update_sample();
        }

        self.mark_data_dirty();

        self.update_all_gui();
    }

    fn update_sample(&self) {
        if let Some(sample) = self.last_sample.get() {
            if let Some(p) = sample.pressure {
                self.last_sample
                    .set(Some(::sounding_analysis::linear_interpolate(
                        &self.list.borrow()[self.currently_displayed_index.get()],
                        p,
                    )));
            } else {
                self.last_sample.set(None);
            }
        }
        self.mark_overlay_dirty();
    }

    // Update all the gui elements
    pub fn update_all_gui(&self) {
        if let Some(ref gui) = *self.gui.borrow() {
            gui.draw_all();
            gui.update_text_view(self);
        }
    }

    /// Get the sounding to draw.
    pub fn get_sounding_for_display(&self) -> Option<Rc<Sounding>> {
        if self.plottable() {
            let shared_ptr = Rc::clone(&self.list.borrow()[self.currently_displayed_index.get()]);
            Some(shared_ptr)
        } else {
            None
        }
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

        if let Some(ref gui) = *self.gui.borrow() {
            let ta = gui.get_text_area();
            ::gui::update_text_highlight(&ta, self);
        }

        self.mark_overlay_dirty();
    }

    pub fn mark_data_dirty(&self) {
        self.hodo.mark_data_dirty();
        self.skew_t.mark_data_dirty();
        self.rh_omega.mark_data_dirty();
        self.cloud.mark_data_dirty();
        // TODO: Mark others as I can
    }

    pub fn mark_overlay_dirty(&self) {
        self.hodo.mark_overlay_dirty();
        self.skew_t.mark_overlay_dirty();
        self.rh_omega.mark_overlay_dirty();
        self.cloud.mark_overlay_dirty();
        // TODO: Mark others as I can
    }

    pub fn mark_background_dirty(&self) {
        self.hodo.mark_background_dirty();
        self.skew_t.mark_background_dirty();
        self.rh_omega.mark_background_dirty();
        self.cloud.mark_background_dirty();
        // TODO: Mark others as I can
    }
}
