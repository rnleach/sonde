//! This module is for a container to hold all of the widgets in the application that may need to
//! be referenced later and in many parts of the program.

use ::gtk::{DrawingArea, WidgetExt};

#[derive(Clone)]
pub struct SondeWidgets {
    sounding_area: DrawingArea,
    hodograph_area: DrawingArea,
    index_area1: DrawingArea,
    index_area2: DrawingArea,
}

impl SondeWidgets {

    pub fn new() -> SondeWidgets {
        SondeWidgets {
            sounding_area: DrawingArea::new(),
            hodograph_area: DrawingArea::new(),
            index_area1: DrawingArea::new(),
            index_area2: DrawingArea::new(),
        }
    }

    pub fn get_sounding_area(&self) -> DrawingArea {
        self.sounding_area.clone()
    }

    pub fn get_hodograph_area(&self) -> DrawingArea {
        self.hodograph_area.clone()
    }

    pub fn get_index_areas(&self) -> (DrawingArea, DrawingArea) {
        (self.index_area1.clone(), self.index_area2.clone())
    }

    pub fn draw_all(&self) {
        self.sounding_area.queue_draw();
        self.hodograph_area.queue_draw();
        self.index_area1.queue_draw();
        self.index_area2.queue_draw();
    }
}
