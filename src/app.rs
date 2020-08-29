//! Module for storing and manipulating the application state. This state is globally shared
//! via smart pointers.
use crate::{
    analysis::Analysis,
    errors::SondeError,
    gui::{
        self,
        profiles::{CloudContext, RHOmegaContext, WindSpeedContext},
        FirePlumeContext, FirePlumeEnergyContext, HodoContext, PlotContext, PlotContextExt,
        SkewTContext,
    },
};
use crossbeam_channel::TryRecvError;
use gtk::prelude::BuilderExtManual;
use sounding_analysis::{self};
use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

// Module for configuring application
pub mod config;
use self::config::Config;

// Module for dealing with sample data from the program
pub mod sample;
use sample::{create_sample_sounding, Sample};

/// Smart pointer for globally shareable data
pub type AppContextPointer = Rc<AppContext>;

/// Holds the application state. This is a singleton (not enforced) that is shared globally.
pub struct AppContext {
    // Configuration, style and layout settings.
    pub config: RefCell<Config>,

    // Lists of soundings and currently displayed one
    list: RefCell<Vec<Rc<RefCell<Analysis>>>>,
    currently_displayed_index: Cell<usize>,
    last_sample: RefCell<Sample>,

    // The number of the times we've called open. Helps keep threads synced.
    load_calls: Cell<usize>,

    // Last Drawing area to have focus, for use with focus buttons
    last_focus: Cell<ZoomableDrawingAreas>,

    // Handle to the GUI
    gui: gtk::Builder,

    // Handle to skew-t context
    pub skew_t: SkewTContext,

    // Handle to Hodograph context
    pub hodo: HodoContext,

    // Handle to FirePlume context
    pub fire_plume: FirePlumeContext,

    // Handle to FirePlumeEnergy context
    pub fire_plume_energy: FirePlumeEnergyContext,
    // Handle to RH Omega Context
    pub rh_omega: RHOmegaContext,

    // Handle to Cloud profile context
    pub cloud: CloudContext,

    // Handle to wind speed profile context
    pub wind_speed: WindSpeedContext,
}

#[derive(Clone, Copy, Debug)]
pub enum ZoomableDrawingAreas {
    SkewT,
    Hodo,
    FirePlume,
    FirePlumeEnergy,
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
            list: RefCell::new(vec![]),
            currently_displayed_index: Cell::new(0),
            last_sample: RefCell::new(Sample::None),
            load_calls: Cell::new(0),
            last_focus: Cell::new(ZoomableDrawingAreas::SkewT),
            gui: gtk::Builder::from_string(glade_src),
            skew_t: SkewTContext::new(),
            rh_omega: RHOmegaContext::new(),
            cloud: CloudContext::new(),
            hodo: HodoContext::new(),
            fire_plume: FirePlumeContext::new(),
            fire_plume_energy: FirePlumeEnergyContext::new(),
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

        acp.set_currently_displayed(0);
        acp.mark_background_dirty();

        // Once everything we need for this thread is taken care of, fill in any missing data
        // in the analysis.
        let pool = threadpool::ThreadPool::default();
        let (tx, rx) = crossbeam_channel::unbounded();
        let num_loads = acp.load_calls.get() + 1;
        acp.load_calls.set(num_loads);

        for (i, anal) in acp.list.borrow().iter().enumerate() {
            let anal = anal.borrow().clone();
            let tx = tx.clone();
            pool.execute(move || {
                let mut anal = anal;
                anal.fill_in_missing_analysis_mut();
                tx.send((num_loads, i, anal)).unwrap();
            });
        }

        let acp = Rc::clone(&acp);
        glib::idle_add_local(move || loop {
            match rx.try_recv() {
                Ok((loaded_on, i, anal)) => {
                    if loaded_on == acp.load_calls.get() {
                        *(*acp).list.borrow_mut()[i].borrow_mut() = anal;

                        if (*acp).currently_displayed_index.get() == i {
                            acp.mark_data_dirty();
                            acp.update_all_gui();
                        }
                    }
                }
                Err(TryRecvError::Empty) => return glib::Continue(true),
                Err(TryRecvError::Disconnected) => return glib::Continue(false),
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

    /// Display the first sounding in the series.
    pub fn display_first(&self) {
        if self.plottable() {
            self.set_currently_displayed(0);
        }
    }

    /// Display the last sounding in the series.
    pub fn display_last(&self) {
        if self.plottable() {
            let last_index = self.list.borrow().len() - 1;
            self.set_currently_displayed(last_index);
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
        let sample: &mut Sample = &mut self.last_sample.borrow_mut();

        match sample {
            Sample::Sounding { data, .. } => {
                *sample = data
                    .pressure
                    .into_option()
                    .and_then(|p| {
                        self.list
                            .borrow()
                            .get(self.currently_displayed_index.get())
                            .and_then(|anal| {
                                sounding_analysis::linear_interpolate_sounding(
                                    anal.borrow().sounding(),
                                    p,
                                )
                                .ok()
                                .map(|dr| create_sample_sounding(dr, &anal.borrow()))
                            })
                    })
                    .unwrap_or(Sample::None);
            }
            Sample::FirePlume { .. } => *sample = Sample::None,
            Sample::None => {}
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

    pub fn get_sample(&self) -> Ref<Sample> {
        self.last_sample.borrow()
    }

    pub fn set_sample(&self, sample: Sample) {
        *self.last_sample.borrow_mut() = sample;
        gui::update_text_highlight(&self);
        self.mark_overlay_dirty();
    }

    pub fn set_last_focus(&self, zoomable: ZoomableDrawingAreas) {
        self.last_focus.set(zoomable);
    }

    pub fn zoom_in(&self) {
        use ZoomableDrawingAreas::*;

        match self.last_focus.get() {
            SkewT => self.skew_t.zoom_in(),
            Hodo => self.hodo.zoom_in(),
            FirePlume => self.fire_plume.zoom_in(),
            FirePlumeEnergy => self.fire_plume_energy.zoom_in(),
        }

        self.mark_background_dirty();
        gui::draw_all(self);
    }

    pub fn zoom_out(&self) {
        use ZoomableDrawingAreas::*;

        match self.last_focus.get() {
            SkewT => self.skew_t.zoom_out(),
            Hodo => self.hodo.zoom_out(),
            FirePlume => self.fire_plume.zoom_out(),
            FirePlumeEnergy => self.fire_plume_energy.zoom_out(),
        }

        self.mark_background_dirty();
        gui::draw_all(self);
    }

    pub fn mark_data_dirty(&self) {
        self.hodo.mark_data_dirty();
        self.fire_plume.mark_data_dirty();
        self.fire_plume_energy.mark_data_dirty();
        self.skew_t.mark_data_dirty();
        self.rh_omega.mark_data_dirty();
        self.cloud.mark_data_dirty();
        self.wind_speed.mark_data_dirty();
    }

    pub fn mark_overlay_dirty(&self) {
        self.hodo.mark_overlay_dirty();
        self.fire_plume.mark_overlay_dirty();
        self.fire_plume_energy.mark_overlay_dirty();
        self.skew_t.mark_overlay_dirty();
        self.rh_omega.mark_overlay_dirty();
        self.cloud.mark_overlay_dirty();
        self.wind_speed.mark_overlay_dirty();
    }

    pub fn mark_background_dirty(&self) {
        self.hodo.mark_background_dirty();
        self.fire_plume.mark_background_dirty();
        self.fire_plume_energy.mark_background_dirty();
        self.skew_t.mark_background_dirty();
        self.rh_omega.mark_background_dirty();
        self.cloud.mark_background_dirty();
        self.wind_speed.mark_background_dirty();
    }
}
