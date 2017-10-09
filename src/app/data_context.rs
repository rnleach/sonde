//! Globally shareable sounding data.
#![allow(dead_code)] // for now.

use std::rc::Rc;
use std::cell::RefCell;

use sounding_base::Sounding;

use errors::*;
use gui::sonde_widgets::SondeWidgets;
use gui::sounding::XYCoords;
use app;

/// Smart pointer for globally shareable data
pub type DataContextPointer = Rc<RefCell<DataContext>>;

pub struct DataContext {
    list: Vec<Sounding>,
    lower_left: XYCoords,
    upper_right: XYCoords,

    currently_displayed: usize,
    widgets: SondeWidgets,
}

impl DataContext {
    pub fn new(widgets: SondeWidgets) -> DataContextPointer {
        Rc::new(RefCell::new(DataContext {
            list: vec![],
            lower_left: (0.0, 0.0),
            upper_right: (1.0, 1.0),
            currently_displayed: 0,
            widgets: widgets,
        }))
    }

    /// Load data from a source.
    pub fn load_data(&mut self, src: &mut Iterator<Item = Sounding>) -> Result<()> {
        use config;

        self.list = src.into_iter().collect();
        self.currently_displayed = 0;

        self.lower_left = (0.45, 0.45);
        self.upper_right = (0.55, 0.55);

        for snd in &self.list {
            for pair in snd.pressure.iter().zip(&snd.temperature).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
                    if p < config::MINP { None } else { Some((t, p)) }
                } else {
                    None
                }
            })
            {
                let (x, y) = app::sounding_context::SoundingContext::convert_tp_to_xy(pair);
                if x < self.lower_left.0 {
                    self.lower_left.0 = x;
                }
                if y < self.lower_left.1 {
                    self.lower_left.1 = y;
                }
                if x > self.upper_right.0 {
                    self.upper_right.0 = x;
                }
                if y > self.upper_right.1 {
                    self.upper_right.1 = y;
                }
            }

            // TODO: Once you get surface temperature, make a station pressure-surface temp
            // point and throw that in the mix.
            // if let Some(p) = snd.station_pres.as_option() {

            // }

            for pair in snd.pressure.iter().zip(&snd.dew_point).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
                    if p < config::MINP { None } else { Some((t, p)) }
                } else {
                    None
                }
            })
            {
                let (x, y) = app::sounding_context::SoundingContext::convert_tp_to_xy(pair);
                if x < self.lower_left.0 {
                    self.lower_left.0 = x;
                }
                if y < self.lower_left.1 {
                    self.lower_left.1 = y;
                }
                if x > self.upper_right.0 {
                    self.upper_right.0 = x;
                }
                if y > self.upper_right.1 {
                    self.upper_right.1 = y;
                }
            }
        }

        self.widgets.draw_all();

        Ok(())
    }

    /// Is there any data to plot?
    pub fn plottable(&self) -> bool {
        !self.list.is_empty()
    }

    /// Set the next one as the one to display, or wrap to the beginning.
    pub fn display_next(&mut self) {
        if self.plottable() {
            if self.currently_displayed < self.list.len() - 1 {
                self.currently_displayed += 1;
            } else {
                self.currently_displayed = 0;
            }
        }

        self.widgets.draw_all();
    }

    /// Set the previous one as the one to display, or wrap to the end.
    pub fn display_previous(&mut self) {
        if self.plottable() {
            if self.currently_displayed > 0 {
                self.currently_displayed -= 1;
            } else {
                self.currently_displayed = self.list.len() - 1;
            }
        }

        self.widgets.draw_all();
    }

    /// Get the sounding to draw.
    pub fn get_sounding_for_display(&self) -> Option<&Sounding> {
        if self.plottable() {
            Some(&self.list[self.currently_displayed])
        } else {
            None
        }
    }

    /// Get the lower left
    pub fn get_lower_left(&self) -> XYCoords {
        self.lower_left
    }

    /// Get the upper right XY coordinate
    pub fn get_upper_right(&self) -> XYCoords {
        self.upper_right
    }
}
