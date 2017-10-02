//! Module holds the code for drawing the skew-t plot area.

#![allow(dead_code)] // For now.

use std::rc::Rc;
use std::cell::RefCell;

use gdk::{EventScroll, SCROLL_MASK, ScrollDirection};

use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::{Context, Matrix};

/// Initialize the drawing area and connect signal handlers.
pub fn set_up_sounding_area(sounding_area: &DrawingArea, sounding_context: SoundingContextPointer) {

    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    let sc1 = sounding_context.clone();
    sounding_area.connect_draw(move |da, cr| draw_sounding(da, cr, &sc1));

    let sc1 = sounding_context.clone();
    sounding_area.connect_scroll_event(move |da, ev| scroll_event(da, ev, &sc1));

    sounding_area.add_events(SCROLL_MASK.bits() as i32);

}

/// `Rc<RefCell<SoundingContext>>` so that this type can be easily shared as global state.
pub type SoundingContextPointer = Rc<RefCell<SoundingContext>>;

/// Used during program initialization to create the SoundingContext and smart pointer.
pub fn create_sounding_context() -> SoundingContextPointer {
    Rc::new(RefCell::new(SoundingContext {
        zoom_factor: 1.0,
        translate_x: 0.0,
        translate_y: 0.0,
        device_height: 100,
        device_width: 100,
    }))
}

/// Stores state of the sounding view between function, method, and callback calls.
pub struct SoundingContext {
    // Standard x-y coords
    zoom_factor: f32, // Multiply by this after translating
    translate_x: f32, // subtract this from x before converting to screen coords.
    translate_y: f32, // subtract this from y before converting to screen coords.
    device_height: i32,
    device_width: i32,
}

/// Temperature-Pressure coordinates.
pub type TPCoords = (f32, f32);
/// XY coordinates of the skew-t graph.
pub type XYCoords = (f32, f32);
/// On screen coordinates.
pub type ScreenCoords = (f64, f64);
/// Device coordinates (pixels)
pub type DeviceCoords = (f64, f64);

impl SoundingContext {
    // Constants for defining a standard x-y coordinate system
    /// Maximum pressure plotted on skew-t (bottom edge)
    const MAXP: f32 = 1050.0; // mb
    /// Minimum pressure plotted on skew-t (top edge)
    const MINP: f32 = 90.0; // mb
    /// Coldest temperature plotted at max pressure, on the bottom edge.
    const MINT: f32 = -46.5; // C - at MAXP
    /// Warmest temperature plotted at max pressure, on the bottom edge.
    const MAXT: f32 = 50.5; // C - at MAXP

    /// Conversion from temperature (t) and pressure (p) to (x,y) coords
    #[inline]
    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use std::f32;

        let y = (f32::log10(SoundingContext::MAXP) - f32::log10(coords.1)) /
            (f32::log10(SoundingContext::MAXP) - f32::log10(SoundingContext::MINP));
        // let y = f32::log10(-(coords.1 - SoundingContext::MAXP) + 0.00001) /
        //     f32::log10(-(SoundingContext::MINP - SoundingContext::MAXP) + 0.00001);
        let x = (coords.0 - SoundingContext::MINT) /
            (SoundingContext::MAXT - SoundingContext::MINT);
        // do the skew
        let x = x + y;
        (x, y)
    }

    /// Convert device coords to (x,y) coords
    #[inline]
    pub fn convert_device_to_xy(&self, coords: DeviceCoords) -> XYCoords {
        let screen_coords = (
            coords.0 / self.device_height as f64,
            -(coords.1 / self.device_height as f64) + 1.0,
        );

        self.convert_screen_to_xy(screen_coords)
    }

    /// TODO: implement Conversion from  (x,y) coords to temperature and pressure.
    #[inline]
    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        unimplemented!();
    }

    /// Conversion from (x,y) coords to screen coords
    #[inline]
    pub fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        // Screen coords go 0 -> 1 up the y axis and 0 -> aspect_ratio right along the x axis.

        // Apply translation first
        let x = coords.0 - self.translate_x;
        let y = coords.1 - self.translate_y;

        // Apply scaling
        let x = (self.zoom_factor * x) as f64;
        let y = (self.zoom_factor * y) as f64;
        (x, y)
    }

    /// Conversion from (x,y) coords to screen coords
    #[inline]
    pub fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Screen coords go 0 -> 1 down the y axis and 0 -> aspect_ratio right along the x axis.

        let x = coords.0 as f32 / self.zoom_factor + self.translate_x;
        let y = coords.1 as f32 / self.zoom_factor + self.translate_y;
        (x, y)
    }

    /// Conversion from temperature/pressure to screen coordinates.
    #[inline]
    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = SoundingContext::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    /// TODO: implement Conversion from screen coordinates to temperature, pressure.
    #[inline]
    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        unimplemented!();
    }

    /// TODO: implement Fit to the given x-y max coords.
    #[inline]
    pub fn fit_to(&mut self, lower_left: XYCoords, upper_right: XYCoords) {
        unimplemented!();
    }
}

/// Draws the sounding, connected to the on-draw event signal.
fn draw_sounding(
    sounding_area: &DrawingArea,
    cr: &Context,
    sc: &SoundingContextPointer,
) -> Inhibit {

    let mut sc = sc.borrow_mut();

    // Get the dimensions of the DrawingArea
    let alloc = sounding_area.get_allocation();
    sc.device_width = alloc.width;
    sc.device_height = alloc.height;
    let aspect_ratio = sc.device_width as f64 / sc.device_height as f64;

    // Make coordinates x: 0 -> aspect_ratio and y: 0 -> 1.0
    cr.scale(sc.device_height as f64, sc.device_height as f64);
    // Set origin at lower right.
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: -1.0,
        x0: 0.0,
        y0: 1.0,
    });

    // Draw black backgound
    cr.rectangle(0.0, 0.0, aspect_ratio, 1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.fill();

    // Draw isentrops
    for theta in &ISENTROPS {
        plot_curve_from_points(
            cr,
            &sc,
            1.0,
            (0.6, 0.6, 0.0, 0.5),
            generate_isentrop(*theta),
        );
    }

    // Draw blue lines below freezing
    let mut end_points: Vec<_> = COLD_ISOTHERMS
        .into_iter()
        .map(|t| {
            ((*t, SoundingContext::MAXP), (*t, SoundingContext::MINP))
        })
        .collect();
    plot_straight_lines(cr, &sc, 1.0, (0.0, 0.0, 1.0, 0.5), &end_points);

    // Draw red lines above freezing
    end_points = WARM_ISOTHERMS
        .into_iter()
        .map(|t| {
            ((*t, SoundingContext::MAXP), (*t, SoundingContext::MINP))
        })
        .collect();
    plot_straight_lines(cr, &sc, 1.0, (1.0, 0.0, 0.0, 0.5), &end_points);

    // Draw pressure lines
    end_points = ISOBARS
        .into_iter()
        .map(|p| ((-150.0, *p), (60.0, *p)))
        .collect();
    plot_straight_lines(cr, &sc, 1.0, (1.0, 1.0, 1.0, 0.5), &end_points);

    Inhibit(false)
}

/// Handles zooming from the mouse whell. Connected to the scroll-event signal.
fn scroll_event(
    sounding_area: &DrawingArea,
    event: &EventScroll,
    sc: &SoundingContextPointer,
) -> Inhibit {

    const DELTA_SCALE: f32 = 1.05;
    const MIN_ZOOM: f32 = 0.75;
    const MAX_ZOOM: f32 = 10.0;

    let mut sc = sc.borrow_mut();

    let pos = sc.convert_device_to_xy(event.get_position());
    let dir = event.get_direction();

    let old_zoom = sc.zoom_factor;

    match dir {
        ScrollDirection::Up => {
            sc.zoom_factor *= DELTA_SCALE;
        }
        ScrollDirection::Down => {
            sc.zoom_factor /= DELTA_SCALE;
        }
        _ => {}
    }

    if sc.zoom_factor < MIN_ZOOM {
        sc.zoom_factor = MIN_ZOOM;
    } else if sc.zoom_factor > MAX_ZOOM {
        sc.zoom_factor = MAX_ZOOM;
    }

    sc.translate_x = pos.0 - old_zoom / sc.zoom_factor * (pos.0 - sc.translate_x);
    sc.translate_y = pos.1 - old_zoom / sc.zoom_factor * (pos.1 - sc.translate_y);

    sounding_area.queue_draw();

    Inhibit(true)
}

#[inline]
/// Draw a straight line on the graph.
fn plot_straight_lines(
    cr: &Context,
    sc: &SoundingContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    end_points: &[(TPCoords, TPCoords)],
) {
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);
    for &(start, end) in end_points {
        let start = sc.convert_tp_to_screen(start);
        let end = sc.convert_tp_to_screen(end);
        cr.move_to(start.0, start.1);
        cr.line_to(end.0, end.1);
    }
    cr.stroke();
}

/// Draw a curve connecting a list of points.
fn plot_curve_from_points(
    cr: &Context,
    sc: &SoundingContext,
    line_width_pixels: f64,
    rgba: (f64, f64, f64, f64),
    points: Vec<TPCoords>,
) {
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(cr.device_to_user_distance(line_width_pixels, 0.0).0);

    let mut iter = points.into_iter();
    let start = sc.convert_tp_to_screen(iter.by_ref().next().unwrap());
    cr.move_to(start.0, start.1);
    for end in iter {
        let end = sc.convert_tp_to_screen(end);
        cr.line_to(end.0, end.1);
    }

    cr.stroke();
}

/// Generate a list of Temperature, Pressure points along an isentrope.
fn generate_isentrop(theta: f32) -> Vec<TPCoords> {
    use std::f32;
    const P0: f32 = 1000.0; // For calcuating theta
    const PS: f32 = SoundingContext::MAXP; // This and the next one are the range to calc for.
    const PF: f32 = 300.0;
    const NUM_PNTS: u32 = 30;

    let mut result = vec![];

    let mut p = SoundingContext::MAXP;
    while p >= 300.0 {
        let t = theta * f32::powf(P0 / p, -0.286) - 273.15;
        result.push((t, p));
        p += (PF - PS) / (NUM_PNTS as f32);
    }

    result
}

/// Isotherms to plot on the chart, freezing and below.
const COLD_ISOTHERMS: [f32; 19] = [
    -150.0,
    -140.0,
    -130.0,
    -120.0,
    -110.0,
    -100.0,
    -90.0,
    -80.0,
    -70.0,
    -60.0,
    -50.0,
    -40.0,
    -30.0,
    -25.0,
    -20.0,
    -15.0,
    -10.0,
    -5.0,
    0.0,
];

/// Isotherms to plot on the chart, above freezing.
const WARM_ISOTHERMS: [f32; 12] = [
    5.0,
    10.0,
    15.0,
    20.0,
    25.0,
    30.0,
    35.0,
    40.0,
    45.0,
    50.0,
    55.0,
    60.0,
];

/// Isobars to plot on the chart background.
const ISOBARS: [f32; 9] = [
    1050.0,
    1000.0,
    925.0,
    850.0,
    700.0,
    500.0,
    300.0,
    200.0,
    100.0,
];

/// Isentrops to plot on the chart background.
const ISENTROPS: [f32; 17] = [
    230.0,
    240.0,
    250.0,
    260.0,
    270.0,
    280.0,
    290.0,
    300.0,
    310.0,
    320.0,
    330.0,
    340.0,
    350.0,
    360.0,
    370.0,
    380.0,
    390.0,
];
