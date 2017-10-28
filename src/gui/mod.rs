//! Module for the GUI components of the application.
#![allow(dead_code)] // for now

pub mod hodograph;
pub mod index_area;
pub mod control_area;
pub mod main_window;
pub mod sounding;

use gtk::{DrawingArea, WidgetExt, Window, WindowType};

use app::AppContextPointer;
use coords::ScreenCoords;

/// Aggregation of the GUI components need for later reference.
///
/// Note: This is cloneable because Gtk+ Gui objects are cheap to clone, and just increment a
/// reference count in the gtk-rs library. So cloning this after it is initialized does not copy
/// the GUI, but instead gives a duplicate of the references to the objects.
#[derive(Clone)]
pub struct Gui {
    sounding_area: DrawingArea,
    hodograph_area: DrawingArea,
    index_area: DrawingArea,
    control_area: DrawingArea,
    window: Window,
    app_context: AppContextPointer,
}

impl Gui {
    pub fn new(acp: AppContextPointer) -> Gui {
        let gui = Gui {
            sounding_area: DrawingArea::new(),
            hodograph_area: DrawingArea::new(),
            index_area: DrawingArea::new(),
            control_area: DrawingArea::new(),
            window: Window::new(WindowType::Toplevel),
            app_context: acp.clone(),
        };

        sounding::set_up_sounding_area(&gui.get_sounding_area(), acp.clone());
        hodograph::set_up_hodograph_area(&gui.get_hodograph_area());
        control_area::set_up_control_area(&gui.get_control_area());
        index_area::set_up_index_area(&gui.get_index_area());

        main_window::layout(gui.clone(), acp);

        gui
    }

    pub fn get_sounding_area(&self) -> DrawingArea {
        self.sounding_area.clone()
    }

    pub fn get_hodograph_area(&self) -> DrawingArea {
        self.hodograph_area.clone()
    }

    pub fn get_index_area(&self) -> DrawingArea {
        self.index_area.clone()
    }

    pub fn get_control_area(&self) -> DrawingArea {
        self.control_area.clone()
    }

    pub fn get_window(&self) -> Window {
        self.window.clone()
    }

    pub fn draw_all(&self) {
        self.sounding_area.queue_draw();
        self.hodograph_area.queue_draw();
        self.index_area.queue_draw();
        self.control_area.queue_draw();
    }
}

#[derive(Clone, Copy)]
pub struct ScreenRect {
    lower_left: ScreenCoords,
    upper_right: ScreenCoords,
}

impl ScreenRect {
    pub fn overlaps(&self, other: &ScreenRect) -> bool {
        let (xmin_s, ymin_s) = self.lower_left;
        let (xmax_s, ymax_s) = self.upper_right;
        let (xmin_o, ymin_o) = other.lower_left;
        let (xmax_o, ymax_o) = other.upper_right;

        if xmin_s > xmax_o {
            return false;
        }
        if xmax_s < xmin_o {
            return false;
        }
        if ymin_s > ymax_o {
            return false;
        }
        if ymax_s < ymin_o {
            return false;
        }

        true
    }

    pub fn inside(&self, big_rect: &ScreenRect) -> bool {
        let (xmin_s, ymin_s) = self.lower_left;
        let (xmax_s, ymax_s) = self.upper_right;
        let (xmin_o, ymin_o) = big_rect.lower_left;
        let (xmax_o, ymax_o) = big_rect.upper_right;

        if xmin_s < xmin_o {
            return false;
        }
        if xmax_s > xmax_o {
            return false;
        }
        if ymin_s < ymin_o {
            return false;
        }
        if ymax_s > ymax_o {
            return false;
        }

        true
    }

    pub fn width(&self) -> f64 {
        self.upper_right.0 - self.lower_left.0
    }

    pub fn height(&self) -> f64 {
        self.upper_right.1 - self.lower_left.1
    }

    pub fn add_padding(&self, padding: f64) -> ScreenRect {
        ScreenRect {
            lower_left: (self.lower_left.0 - padding, self.lower_left.1 - padding),
            upper_right: (self.upper_right.0 + padding, self.upper_right.1 + padding),
        }
    }
}
