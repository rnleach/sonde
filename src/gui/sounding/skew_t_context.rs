use std::cell::{Cell, RefCell};

use cairo::{Context, Format, ImageSurface, Matrix, MatrixTrait, Operator};

use gtk::DrawingArea;
use gtk::prelude::*;

use gui::DrawingArgs;
use gui::plot_context::{GenericContext, HasGenericContext, PlotContext};

use app::config;
use coords::{DeviceCoords, DeviceRect, TPCoords, ScreenCoords, XYCoords};

pub struct SkewTContext {
    generic: GenericContext,
    
    pub matrix: Cell<Matrix>,

    dirty_background: Cell<bool>,
    background_layer: RefCell<ImageSurface>,

    dirty_data: Cell<bool>,
    data_layer: RefCell<ImageSurface>,

    dirty_overlay: Cell<bool>,
    overlay_layer: RefCell<ImageSurface>,
}

impl SkewTContext {
    
    pub fn new() -> Self {
        SkewTContext {
            generic: GenericContext::new(),

            matrix: Cell::new(Matrix::identity()),

            dirty_background: Cell::new(true),
            background_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),

            dirty_data: Cell::new(true),
            data_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),

            dirty_overlay: Cell::new(true),
            overlay_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),
        }
    }

    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.pressure))
            / (f64::log10(config::MAXP) - f64::log10(config::MINP));
        let x = (coords.temperature - config::MINT) / (config::MAXT - config::MINT);

        // do the skew
        let x = x + y;
        XYCoords { x, y }
    }

    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        use app::config;
        use std::f64;

        // undo the skew
        let x = coords.x - coords.y;
        let y = coords.y;

        let t = x * (config::MAXT - config::MINT) + config::MINT;
        let p = 10.0f64.powf(
            f64::log10(config::MAXP) - y * (f64::log10(config::MAXP) - f64::log10(config::MINP)),
        );

        TPCoords {
            temperature: t,
            pressure: p,
        }
    }

    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = Self::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        let xy = self.convert_screen_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }

    pub fn convert_device_to_tp(&self, coords: DeviceCoords) -> TPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }
}

impl HasGenericContext for SkewTContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl SkewTContext {
    pub fn mark_background_dirty(&self) {
        self.dirty_background.set(true);
        self.dirty_data.set(true);
        self.dirty_overlay.set(true);
    }

    pub fn mark_data_dirty(&self) {
        self.dirty_background.set(true);
        self.dirty_data.set(true);
    }

    pub fn mark_overlay_dirty(&self) {
        self.dirty_overlay.set(true);
    }

    pub fn init_matrix(&self, args: DrawingArgs) {
        use gui::plot_context::PlotContext;

        let cr = args.cr;

        cr.save();

        let (x1, y1, x2, y2) = cr.clip_extents();
        let width = f64::abs(x2 - x1);
        let height = f64::abs(y2 - y1);

        let device_rect = DeviceRect {
            upper_left: DeviceCoords { row: 0.0, col: 0.0 },
            width,
            height,
        };
        self.set_device_rect(device_rect);
        let scale_factor = self.scale_factor();

        // Start fresh
        cr.identity_matrix();
        // Set the scale factor
        cr.scale(scale_factor, scale_factor);
        // Set origin at lower left.
        cr.transform(Matrix {
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: -1.0,
            x0: 0.0,
            y0: device_rect.height / scale_factor,
        });

        self.matrix.set(cr.get_matrix());
        cr.restore();
    }

    pub fn update_cache_allocations(&self, da: &DrawingArea) {
        // Mark everything as dirty
        self.mark_background_dirty(); // Marks everything

        // Get the size
        let (width, height) = (da.get_allocation().width, da.get_allocation().height);

        // Make the new allocations
        *self.background_layer.borrow_mut() =
            ImageSurface::create(Format::ARgb32, width, height).unwrap();
        *self.data_layer.borrow_mut() =
            ImageSurface::create(Format::ARgb32, width, height).unwrap();
        *self.overlay_layer.borrow_mut() =
            ImageSurface::create(Format::ARgb32, width, height).unwrap();

    }

    pub fn draw_background_cached(&self, args: DrawingArgs) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if self.dirty_background.get() {
            let tmp_cr = Context::new(&self.background_layer.borrow().clone());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            let rgba = config.background_rgba;
            tmp_cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.matrix.get());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.bound_view();

            super::drawing::draw_background(tmp_args);

            self.dirty_background.set(false);
        }

        cr.set_source_surface(&self.background_layer.borrow().clone(), 0.0, 0.0);
        cr.paint();
    }

    pub fn draw_data_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.dirty_data.get() {
            let tmp_cr = Context::new(&self.data_layer.borrow().clone());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.matrix.get());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            super::drawing::draw_data(tmp_args);

            self.dirty_data.set(false);
        }

        cr.set_source_surface(&self.data_layer.borrow().clone(), 0.0, 0.0);
        cr.paint();
    }

    pub fn draw_overlay_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.dirty_overlay.get() {
            let tmp_cr = Context::new(&self.overlay_layer.borrow().clone());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.matrix.get());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            super::drawing::draw_overlays(tmp_args);

            self.dirty_overlay.set(false);
        }

        cr.set_source_surface(&self.overlay_layer.borrow().clone(), 0.0, 0.0);
        cr.paint();
    }
}
