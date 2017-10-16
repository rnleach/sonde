//! Module for the GUI components of the application.
#![allow(dead_code)] // for now

pub mod hodograph;
pub mod index_areas;
pub mod main_window;
pub mod sounding;

use gtk::{DrawingArea, WidgetExt, Window, WindowType};

use app::AppContextPointer;

/// Aggregation of the GUI components need for later reference.
///
/// Note: This is cloneable because Gtk+ Gui objects are cheap to clone, and just increment a
/// reference count in the gtk-rs library. So cloning this after it is initialized does not copy
/// the GUI, but instead gives a duplicate of the references to the objects.
#[derive(Clone)]
pub struct Gui {
    sounding_area: DrawingArea,
    hodograph_area: DrawingArea,
    index_area1: DrawingArea,
    index_area2: DrawingArea,
    window: Window,
    app_context: AppContextPointer,
}

impl Gui {
    pub fn new(acp: AppContextPointer) -> Gui {
        let gui = Gui {
            sounding_area: DrawingArea::new(),
            hodograph_area: DrawingArea::new(),
            index_area1: DrawingArea::new(),
            index_area2: DrawingArea::new(),
            window: Window::new(WindowType::Toplevel),
            app_context: acp.clone(),
        };

        sounding::set_up_sounding_area(&gui.get_sounding_area(), acp.clone());
        hodograph::set_up_hodograph_area(&gui.get_hodograph_area());

        let (ia1, ia2) = gui.get_index_areas();
        index_areas::set_up_index_areas(&ia1, &ia2);

        main_window::layout(gui.clone(), acp);

        gui
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

    pub fn get_window(&self) -> Window {
        self.window.clone()
    }

    pub fn draw_all(&self) {
        self.sounding_area.queue_draw();
        self.hodograph_area.queue_draw();
        self.index_area1.queue_draw();
        self.index_area2.queue_draw();
    }
}