//! Module for the GUI components of the application.
use std::cell::Cell;
use std::rc::Rc;

mod plot_context;
pub use self::plot_context::{PlotContext, HasGenericContext};
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
use gtk::{DrawingArea, Notebook, Window, WindowType, TextView};

use app::{AppContextPointer, AppContext};
use coords::{DeviceCoords, ScreenCoords, ScreenRect, Rect};

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
        self.hodograph_area.queue_draw();

        // TODO: Add here as needed.
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
pub struct DrawingArgs<'a, 'b, 'c> {
    pub ac: &'a AppContext,
    pub cr: &'b Context,
    pub da: &'c DrawingArea,
}

impl<'a, 'b, 'c> DrawingArgs<'a, 'b, 'c> {
    pub fn new(ac: &'a AppContext, cr: &'b Context, da: &'c DrawingArea) -> Self {
        DrawingArgs { ac, cr, da }
    }
}

#[derive(Clone, Copy)]
pub enum LazyDrawingCacheVar {
    SkewTLabelPadding,
    SkewTEdgePadding,
    SkewTZoomFactor,
    SkewTScaleFactor,
    OmegaLabelPadding,
    // OmegaEdgePadding,
    // HodoLabelPadding,
    // HodoEdgePadding,
}

#[derive(Clone, Default)]
pub struct LazyDrawingCache {
    skew_t_label_padding: Cell<Option<f64>>,
    skew_t_edge_padding: Cell<Option<f64>>,
    skew_t_zoom_factor: Cell<Option<f64>>,
    skew_t_scale_factor: Cell<Option<f64>>,
    omega_label_padding: Cell<Option<f64>>,
    omega_edge_padding: Cell<Option<f64>>,
    hodo_label_padding: Cell<Option<f64>>,
    hodo_edge_padding: Cell<Option<f64>>,
}

impl LazyDrawingCache {
    pub fn get(&self, var: LazyDrawingCacheVar, args: DrawingArgs) -> f64 {
        use self::LazyDrawingCacheVar::*;

        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        macro_rules! make_cache_getter {
            ($var:ident, $val:expr) => {
                match self.$var.get() {
                    Some(val) => val,
                    None => {
                        let val = $val;
                        self.$var.set(Some(val));
                        val
                    }
                }
            }
        }

        match var {
            SkewTLabelPadding => {
                make_cache_getter!(
                    skew_t_label_padding,
                    cr.device_to_user_distance(config.label_padding, 0.0).0
                )
            }
            SkewTEdgePadding => {
                make_cache_getter!(
                    skew_t_edge_padding,
                    cr.device_to_user_distance(config.edge_padding, 0.0).0
                )
            }
            SkewTZoomFactor => make_cache_getter!(skew_t_zoom_factor, ac.skew_t.get_zoom_factor()),
            SkewTScaleFactor => {
                make_cache_getter!(skew_t_scale_factor, {
                    if let Some(ref gui) = *ac.gui.borrow() {
                        let da = &gui.get_sounding_area();
                        SkewTContext::scale_factor(da)
                    } else {
                        1.0
                    }
                })
            }
            OmegaLabelPadding => {
                make_cache_getter!(
                    omega_label_padding,
                    cr.device_to_user_distance(config.label_padding, 0.0).0
                )
            }
            // OmegaEdgePadding => {
            //     make_cache_getter!(
            //         omega_edge_padding,
            //         cr.device_to_user_distance(config.edge_padding, 0.0).0
            //     )
            // }
            // HodoLabelPadding => {
            //     make_cache_getter!(
            //         hodo_label_padding,
            //         cr.device_to_user_distance(config.label_padding, 0.0).0
            //     )
            // }
            // HodoEdgePadding => {
            //     make_cache_getter!(
            //         hodo_edge_padding,
            //         cr.device_to_user_distance(config.edge_padding, 0.0).0
            //     )
            // }
        }
    }

    pub fn reset(&self) {
        self.skew_t_label_padding.set(None);
        self.skew_t_edge_padding.set(None);
        self.skew_t_zoom_factor.set(None);
        self.skew_t_scale_factor.set(None);
        self.omega_label_padding.set(None);
        self.omega_edge_padding.set(None);
        self.hodo_label_padding.set(None);
        self.hodo_edge_padding.set(None);
    }
}

fn set_font_size<T: PlotContext>(
    pc: &T,
    da: &DrawingArea,
    size_in_pnts: f64,
    cr: &Context,
    ac: &AppContext,
) {

    let dpi = match ac.get_dpi() {
        None => 72.0,
        Some(value) => value,
    };

    let font_size = size_in_pnts / 72.0 * dpi;
    let ScreenCoords { x: font_size, .. } = pc.convert_device_to_screen(
        da,
        DeviceCoords {
            col: font_size,
            row: 0.0,
        },
    );

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
