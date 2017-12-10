//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.

use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;

use sounding_base::{Sounding, DataRow};

use coords::{TPCoords, WPCoords, SDCoords, XYCoords, XYRect};
use errors::*;
use gui::Gui;
use gui::hodograph::hodo_context::HodoContext;
use gui::sounding::skew_t_context::SkewTContext;
use gui::sounding::rh_omega_context::RHOmegaContext;
use gui::{PlotContext, LazyDrawingCache};

// Module for configuring application
pub mod config;
use app::config::Config;

/// Smart pointer for globally shareable data
pub type AppContextPointer = Rc<RefCell<AppContext>>;

/// Holds the application state. This is a singleton (not enforced) that is shared globally.
pub struct AppContext {
    // Configuration, style and layout settings.
    pub config: Config,

    // Source description is used in the legend if it is present. Not all file formats include a
    // station name or model name or base time. In bufkit files this is usually part of the file
    // name. So whatever function loads a sounding should set this to reflect where it came from.
    source_description: Option<String>,

    list: Vec<Sounding>,
    currently_displayed_index: usize,
    last_sample: Option<DataRow>,

    // Handle to the GUI
    pub gui: Option<Gui>,

    // Handle to skew-t context
    pub skew_t: SkewTContext,

    // Handle to RH Omega Context
    pub rh_omega: RHOmegaContext,

    // Handle to Hodograph context
    pub hodo: HodoContext,

    // Cache for values when drawing
    pub drawing_cache: LazyDrawingCache,
}

impl AppContext {
    /// Create a new instance of AppContext and return a smart pointer to it.
    ///
    /// Note: It is important at a later time to call set_gui, otherwise nothing will ever be
    /// drawn on the GUI.
    pub fn new() -> AppContextPointer {
        Rc::new(RefCell::new(AppContext {
            config: Config::default(),
            source_description: None,
            list: vec![],
            currently_displayed_index: 0,
            last_sample: None,
            gui: None,
            skew_t: SkewTContext::new(),
            rh_omega: RHOmegaContext::new(),
            hodo: HodoContext::new(),
            drawing_cache: LazyDrawingCache::default(),
        }))
    }

    pub fn set_gui(&mut self, gui: Gui) {
        self.gui = Some(gui);
    }

    pub fn load_data(&mut self, src: &mut Iterator<Item = Sounding>) -> Result<()> {
        use app::config;
        use sounding_base::Profile::*;

        self.list = src.into_iter().collect();
        self.currently_displayed_index = 0;
        self.source_description = None;

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

        for snd in &self.list {
            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(Temperature))
                .filter_map(|p| if let (Some(p), Some(t)) =
                    (p.0.as_option(), p.1.as_option())
                {
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

            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(DewPoint))
                .filter_map(|p| if let (Some(p), Some(t)) =
                    (p.0.as_option(), p.1.as_option())
                {
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

            for pair in snd.get_profile(Pressure)
                .iter()
                .zip(snd.get_profile(PressureVerticalVelocity))
                .filter_map(|p| if let (Some(p), Some(o)) =
                    (p.0.as_option(), p.1.as_option())
                {
                    if p < config::MINP {
                        None
                    } else {
                        Some(WPCoords { w: o.abs(), p })
                    }
                } else {
                    None
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

            for pair in izip!(
                snd.get_profile(Pressure),
                snd.get_profile(WindSpeed),
                snd.get_profile(WindDirection)
            ).filter_map(|tuple| if let (Some(p), Some(s), Some(d)) =
                (
                    tuple.0.as_option(),
                    tuple.1.as_option(),
                    tuple.2.as_option(),
                )
            {
                if p < config::MINP {
                    None
                } else {
                    Some(SDCoords { speed: s, dir: d })
                }
            } else {
                None
            })
            {
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

        self.fit_to_data();

        if let Some(ref gui) = self.gui {
            gui.draw_all();
            gui.update_text_view(self);
        }

        Ok(())
    }

    /// Is there any data to plot?
    pub fn plottable(&self) -> bool {
        !self.list.is_empty()
    }

    /// Set the next one as the one to display, or wrap to the beginning.
    pub fn display_next(&mut self) {
        if self.plottable() {
            if self.currently_displayed_index < self.list.len() - 1 {
                self.currently_displayed_index += 1;
            } else {
                self.currently_displayed_index = 0;
            }
            self.update_sample();
        }

        self.update_all_gui();
    }

    /// Set the previous one as the one to display, or wrap to the end.
    pub fn display_previous(&mut self) {
        if self.plottable() {
            if self.currently_displayed_index > 0 {
                self.currently_displayed_index -= 1;
            } else {
                self.currently_displayed_index = self.list.len() - 1;
            }
            self.update_sample();
        }

        self.update_all_gui();
    }

    fn update_sample(&mut self) {
        if let Some(sample) = self.last_sample.clone() {
            if let Some(p) = sample.pressure.as_option() {
                self.last_sample = Some(::sounding_analysis::linear_interpolate(
                    &self.list[self.currently_displayed_index],
                    p,
                ));
            } else {
                self.last_sample = None;
            }
        }
    }

    // Update all the gui elements
    pub fn update_all_gui(&self) {
        if let Some(ref gui) = self.gui {
            gui.draw_all();
            gui.update_text_view(self);
        }
    }

    /// Get the sounding to draw.
    pub fn get_sounding_for_display(&self) -> Option<&Sounding> {
        if self.plottable() {
            Some(&self.list[self.currently_displayed_index])
        } else {
            None
        }
    }

    /// Set the source name
    pub fn set_source_description(&mut self, new_name: Option<String>) {
        self.source_description = new_name;
    }

    /// Get the source name
    pub fn get_source_description(&self) -> Option<String> {
        match self.source_description {
            Some(ref name) => Some(name.clone()),
            None => None,
        }
    }

    /// Get the screen resolution in dpi
    pub fn get_dpi(&self) -> Option<f64> {
        use gtk::WidgetExt;
        use gdk::ScreenExt;

        match self.gui {
            None => None,
            Some(ref gui) => {
                match gui.get_sounding_area().get_screen() {
                    None => None,
                    Some(ref screen) => Some(screen.get_resolution()),
                }
            }
        }
    }

    /// Fit to the given x-y max coords. SHOULD NOT BE PUBLIC - DO NOT USE IN DRAWING CALLBACKS.
    fn fit_to_data(&mut self) {

        if let Some(ref gui) = self.gui {

            self.skew_t.zoom_to_envelope(&gui.get_sounding_area());
            self.hodo.zoom_to_envelope(&gui.get_hodograph_area());
            self.rh_omega.zoom_to_envelope(&gui.get_omega_area());

            self.rh_omega.set_translate_y(self.skew_t.get_translate());
        }
    }

    /// Get the zoom factor
    pub fn get_zoom_factor(&self) -> f64 {
        self.skew_t.get_zoom_factor()
    }

    pub fn show_hide_rh_omega(&self) {
        if let Some(ref gui) = self.gui {
            if self.config.show_rh_omega_frame {
                gui.get_omega_area().show();
            } else {
                gui.get_omega_area().hide();
            }
        }
    }

    pub fn get_sample(&self) -> Option<DataRow> {
        self.last_sample.clone()
    }

    pub fn set_sample<T>(&mut self, sample: T)
    where
        Option<DataRow>: From<T>,
    {
        self.last_sample = Option::from(sample);

        if let Some(ref gui) = self.gui {
            let ta = gui.get_text_area();
            ::gui::text_area::update_text_highlight(&ta, self);
        }
    }
}
