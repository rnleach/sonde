#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;

use sounding_base::Sounding;

use errors::*;
use sonde_widgets::SondeWidgets;
use sounding::{XYCoords, SoundingContext};

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

        self.list = src.into_iter().collect();
        self.currently_displayed = 0;

        self.lower_left = (0.45, 0.45);
        self.upper_right = (0.55, 0.55);

        for snd in &self.list {
            for pair in snd.pressure.iter().zip(&snd.temperature).filter_map(|p| {
                if let (Some(p), Some(t)) = (p.0.as_option(), p.1.as_option()) {
                    Some((t, p))
                } else {
                    None
                }
            })
            {
                let (x, y) = SoundingContext::convert_tp_to_xy(pair);
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
        }

        self.widgets.draw_all();
        unimplemented!();
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
}
