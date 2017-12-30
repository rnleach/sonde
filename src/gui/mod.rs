//! Module for the GUI components of the application.

use std::rc::Rc;

mod plot_context;
pub use self::plot_context::{HasGenericContext, PlotContext};
pub use self::sounding::skew_t_context::SkewTContext;
pub use self::sounding::rh_omega_context::RHOmegaContext;
pub use self::hodograph::hodo_context::HodoContext;

pub mod hodograph;
pub mod index_area;
pub mod control_area;
pub mod main_window;
pub mod sounding;
pub mod text_area;

use cairo::{Context, Matrix};
use gtk::prelude::*;
use gtk::{DrawingArea, Notebook, TextView, Window, WindowType};

use app::{AppContext, AppContextPointer};
use coords::{Rect, ScreenCoords, ScreenRect};

/// Aggregation of the GUI components need for later reference.
///
/// Note: This is cloneable because Gtk+ Gui objects are cheap to clone, and just increment a
/// reference count in the gtk-rs library. So cloning this after it is initialized does not copy
/// the GUI, but instead gives a duplicate of the references to the objects.
#[derive(Clone)]
pub struct Gui {
    // Left pane
    sounding_area: DrawingArea,

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

            hodograph_area: DrawingArea::new(),
            index_area: DrawingArea::new(),
            control_area: Notebook::new(),
            text_area: TextView::new(),

            window: Window::new(WindowType::Toplevel),
            app_context: Rc::clone(acp),
        };

        sounding::set_up_sounding_area(&gui.get_sounding_area(), acp);
        hodograph::set_up_hodograph_area(&gui.get_hodograph_area(), acp);
        control_area::set_up_control_area(&gui.get_control_area(), acp);
        index_area::set_up_index_area(&gui.get_index_area());
        text_area::set_up_text_area(&gui.get_text_area(), acp);

        main_window::layout(&gui, acp);

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
        self.hodograph_area.queue_draw();
    }

    pub fn update_text_view(&self, ac: &AppContext) {
        if self.text_area.is_visible() {
            self::text_area::update_text_area(&self.text_area, ac);
            self::text_area::update_text_highlight(&self.text_area, ac);
        }
    }
}

// Draw a curve connecting a list of points.
fn plot_curve_from_points<I>(
    cr: &Context,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: I,
) where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);

    let mut points = points;
    if let Some(start) = points.by_ref().next() {
        cr.move_to(start.x, start.y);
        for end in points {
            cr.line_to(end.x, end.y);
        }

        cr.stroke();
    }
}

// Draw a dashed line on the graph.
fn plot_dashed_curve_from_points<I>(
    cr: &Context,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: I,
) where
    I: Iterator<Item = ScreenCoords>,
{
    cr.set_dash(&[0.02], 0.0);
    plot_curve_from_points(cr, line_width_pixels, rgba, points);
    cr.set_dash(&[], 0.0);
}

fn check_overlap_then_add(
    cr: &Context,
    ac: &AppContext,
    vector: &mut Vec<(String, ScreenRect)>,
    plot_edges: &ScreenRect,
    label_pair: (String, ScreenRect),
) {
    let padding = cr.device_to_user_distance(ac.config.borrow().label_padding, 0.0)
        .0;
    let padded_rect = label_pair.1.add_padding(padding);

    // Make sure it is on screen - but don't add padding to this check cause the screen already
    // has padding.
    if !label_pair.1.inside(plot_edges) {
        return;
    }

    // Check for overlap
    for &(_, ref rect) in vector.iter() {
        if padded_rect.overlaps(rect) {
            return;
        }
    }

    vector.push(label_pair);
}

#[derive(Clone, Copy)]
pub struct DrawingArgs<'a, 'b> {
    pub ac: &'a AppContext,
    pub cr: &'b Context,
}

impl<'a, 'b> DrawingArgs<'a, 'b> {
    pub fn new(ac: &'a AppContext, cr: &'b Context) -> Self {
        DrawingArgs { ac, cr }
    }
}

fn set_font_size<T: PlotContext>(pc: &T, size_in_pct: f64, cr: &Context) {
    let height = pc.get_device_rect().height();

    let mut font_size = size_in_pct / 100.0 * height;
    font_size = cr.device_to_user_distance(font_size, 0.0).0;

    // Flip the y-coordinate so it displays the font right side up
    cr.set_font_matrix(Matrix {
        xx: 1.0 * font_size,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0 * font_size, // Reflect it to be right side up!
        x0: 0.0,
        y0: 0.0,
    });
}
