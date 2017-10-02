#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::{Context, Matrix};

pub fn set_up_sounding_area(sounding_area: &DrawingArea, sounding_context: SoundingContextPointer) {

    sounding_area.set_hexpand(true);
    sounding_area.set_vexpand(true);

    sounding_area.connect_draw(move |da, cr| {
        draw_sounding(da, cr, sounding_context.clone())
    });
}

pub type SoundingContextPointer = Rc<RefCell<SoundingContext>>;

pub fn create_sounding_context() -> SoundingContextPointer {
    Rc::new(RefCell::new(SoundingContext {
        zoom_factor: 1.0,
        translate_x: 0.0,
        translate_y: 0.0,
    }))
}

pub struct SoundingContext {
    // Standard x-y coords
    zoom_factor: f32, // Multiply by this after translating
    translate_x: f32, // subtract this from x before converting to screen coords.
    translate_y: f32, // subtract this from y before converting to screen coords.
}

pub type TPCoords = (f32, f32);
pub type XYCoords = (f32, f32);
pub type ScreenCoords = (f64, f64);

impl SoundingContext {
    // Constants for defining a standard x-y coordinate system
    const MAXP: f32 = 1050.0; // mb
    const MINP: f32 = 90.0; // mb
    const MINT: f32 = -46.5; // C - at MAXP
    const MAXT: f32 = 50.5; // C - at MAXP

    // Conversion from temperature (t) and pressure (p) to a standard (x,y) coords
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

    // TODO: implement Conversion from a standard x,y coords to temperature and pressure.
    #[inline]
    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        unimplemented!();
    }

    // Conversion from standard x,y coords to screen coords
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

    // TODO: implement Conversion from standard x,y coords to screen coords
    #[inline]
    pub fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Screen coords go 0 -> 1 down the y axis and 0 -> aspect_ratio right along the x axis.

        // Calculate the translation in XYCoords and apply that first

        // Calculate the scaling to get the x-range to fit on screen
        // Calculate the scaling to get the y-range to fit on screen
        // Choose the scaling that is smaller.
        unimplemented!();
    }

    // Conversion from temperature/pressure to screen coordinates.
    #[inline]
    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = SoundingContext::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    // TODO: implement Conversion from screen coordinates to temperature, pressure.
    #[inline]
    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        unimplemented!();
    }

    // TODO: implement Adjust the translate & scale values for a zoom.
    #[inline]
    pub fn zoom_to(&mut self, center: ScreenCoords) {
        unimplemented!();
    }

    // TODO: implement Fit to the given x-y max coords.
    #[inline]
    pub fn fit_to(&mut self, lower_left: XYCoords, upper_right: XYCoords) {
        unimplemented!();
    }
}

fn draw_sounding(sounding_area: &DrawingArea, cr: &Context, sc: SoundingContextPointer) -> Inhibit {

    let sc = sc.borrow();

    // Get the dimensions of the DrawingArea
    let alloc = sounding_area.get_allocation();
    let width = alloc.width;
    let height = alloc.height;
    let aspect_ratio = width as f64 / height as f64;

    // Make coordinates x: 0 -> aspect_ratio and y: 0 -> 1.0
    cr.scale(height as f64, height as f64);
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
    end_points = ISO_BARS
        .into_iter()
        .map(|p| ((-150.0, *p), (60.0, *p)))
        .collect();
    plot_straight_lines(cr, &sc, 1.0, (1.0, 1.0, 1.0, 0.5), &end_points);

    Inhibit(false)
}

#[inline]
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

const ISO_BARS: [f32; 9] = [
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
