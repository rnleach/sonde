//! Module for the GUI components of the application.
#![allow(dead_code)] // for now

use std::rc::Rc;

use sounding_base::Sounding;

pub mod hodograph;
pub mod index_area;
pub mod control_area;
pub mod main_window;
pub mod sounding;
pub mod text_area;

use gtk::prelude::*;
use gtk::{DrawingArea, Notebook, Window, WindowType, TextView};

use app::AppContextPointer;

/// Aggregation of the GUI components need for later reference.
///
/// Note: This is cloneable because Gtk+ Gui objects are cheap to clone, and just increment a
/// reference count in the gtk-rs library. So cloning this after it is initialized does not copy
/// the GUI, but instead gives a duplicate of the references to the objects.
#[derive(Clone)]
pub struct Gui {
    // Left pane
    sounding_area: DrawingArea,
    omega_area: DrawingArea,

    // Right pane
    hodograph_area: DrawingArea,
    index_area: DrawingArea,
    control_area: Notebook,
    text_area: TextView,

    // Main window
    window: Window,

    // Smart pointer.
    app_context: AppContextPointer,
}

impl Gui {
    pub fn new(acp: &AppContextPointer) -> Gui {
        let gui = Gui {
            sounding_area: DrawingArea::new(),
            omega_area: DrawingArea::new(),

            hodograph_area: DrawingArea::new(),
            index_area: DrawingArea::new(),
            control_area: Notebook::new(),
            text_area: TextView::new(),

            window: Window::new(WindowType::Toplevel),
            app_context: Rc::clone(acp),
        };

        sounding::set_up_sounding_area(&gui.get_sounding_area(), acp);
        sounding::set_up_rh_omega_area(&gui.get_omega_area(), acp);
        hodograph::set_up_hodograph_area(&gui.get_hodograph_area());
        control_area::set_up_control_area(&gui.get_control_area(), acp);
        index_area::set_up_index_area(&gui.get_index_area());
        text_area::set_up_text_area(&gui.get_text_area(), acp);

        main_window::layout(&gui, acp);

        gui
    }

    pub fn get_sounding_area(&self) -> DrawingArea {
        self.sounding_area.clone()
    }

    pub fn get_omega_area(&self) -> DrawingArea {
        self.omega_area.clone()
    }

    pub fn get_hodograph_area(&self) -> DrawingArea {
        self.hodograph_area.clone()
    }

    pub fn get_index_area(&self) -> DrawingArea {
        self.index_area.clone()
    }

    pub fn get_control_area(&self) -> Notebook {
        self.control_area.clone()
    }

    pub fn get_text_area(&self) -> TextView {
        self.text_area.clone()
    }

    pub fn get_window(&self) -> Window {
        self.window.clone()
    }

    pub fn draw_all(&self) {
        self.sounding_area.queue_draw();
        self.omega_area.queue_draw();

        self.draw_right_pane();
    }

    pub fn draw_right_pane(&self) {
        macro_rules! check_draw {
            ($gui_element:ident) => {
                if self.$gui_element.get_visible() {
                    self.$gui_element.queue_draw();
                }
            };
        }

        check_draw!(hodograph_area);
        check_draw!(index_area);
        check_draw!(control_area);
    }

    pub fn update_text_view(&self, snd: Option<&Sounding>) {
        if self.text_area.is_visible() {
            self::text_area::update_text_area(&self.text_area, snd);
        }
    }
}
