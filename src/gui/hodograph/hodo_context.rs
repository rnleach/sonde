
use std::cell::{Cell, RefCell};

use cairo::{Format, ImageSurface, Matrix, MatrixTrait, Context, Operator};

use gtk::DrawingArea;
use gtk::prelude::*;

use gui::DrawingArgs;
use gui::plot_context::{PlotContext, GenericContext, HasGenericContext};

use app::config;
use coords::{ScreenCoords, SDCoords, XYCoords, DeviceRect, DeviceCoords};

pub struct HodoContext {
    generic: GenericContext,

    pub reset_allocation: Cell<bool>,

    pub matrix: Cell<Matrix>,

    dirty_background: Cell<bool>,
    background_layer: RefCell<ImageSurface>,

    dirty_data: Cell<bool>,
    data_layer: RefCell<ImageSurface>,

    dirty_overlay: Cell<bool>,
    overlay_layer: RefCell<ImageSurface>,
}

impl HodoContext {
    // Create a new instance of HodoContext
    pub fn new() -> Self {
        HodoContext {
            generic: GenericContext::new(),

            reset_allocation: Cell::new(true),

            matrix: Cell::new(Matrix::identity()),

            dirty_background: Cell::new(true),
            background_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),

            dirty_data: Cell::new(true),
            data_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),

            dirty_overlay: Cell::new(true),
            overlay_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),
        }
    }

    /// Conversion from speed and direction to (x,y) coords
    pub fn convert_sd_to_xy(coords: SDCoords) -> XYCoords {
        let radius = coords.speed / 2.0 / config::MAX_SPEED;
        let angle = (270.0 - coords.dir).to_radians();

        let x = radius * angle.cos() + 0.5;
        let y = radius * angle.sin() + 0.5;
        XYCoords { x, y }
    }

    /// Conversion from speed and direction to (x,y) coords
    pub fn convert_sd_to_screen(&self, coords: SDCoords) -> ScreenCoords {
        let xy = HodoContext::convert_sd_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }
}

impl HasGenericContext for HodoContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl HodoContext {
    pub fn reset_allocation(&self) {
        self.reset_allocation.set(true);
        self.mark_background_dirty();
    }

    pub fn mark_background_dirty(&self) {
        self.dirty_background.set(true);
        self.dirty_data.set(true);
        self.dirty_overlay.set(true);
    }

    pub fn mark_data_dirty(&self) {
        self.dirty_background.set(true);
        self.dirty_overlay.set(true);
    }

    pub fn mark_overlay_dirty(&self) {
        self.dirty_overlay.set(true);
    }

    pub fn init_matrix(&self, args: DrawingArgs) {
        use gui::plot_context::PlotContext;

        let (ac, cr) = (args.ac, args.cr);

        cr.save();

        let (x1, y1, x2, y2) = cr.clip_extents();
        let width = f64::abs(x2 - x1);
        let height = f64::abs(y2 - y1);

        let device_rect = DeviceRect {
            upper_left: DeviceCoords { row: 0.0, col: 0.0 },
            width,
            height,
        };
        ac.hodo.set_device_rect(device_rect);
        let scale_factor = ac.hodo.scale_factor();

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
        *self.background_layer.borrow_mut() = ImageSurface::create(Format::ARgb32, width, height)
            .unwrap();
        *self.data_layer.borrow_mut() = ImageSurface::create(Format::ARgb32, width, height)
            .unwrap();
        *self.overlay_layer.borrow_mut() = ImageSurface::create(Format::ARgb32, width, height)
            .unwrap();

        // Mark allocations as updated.
        self.reset_allocation.set(false);
    }

    pub fn draw_background_cached(&self, args: DrawingArgs) {

        let (ac, cr) = (args.ac, args.cr);

        if self.dirty_background.get() {

            let tmp_cr = Context::new(&self.background_layer.borrow().clone());
            tmp_cr.transform(self.matrix.get());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            super::drawing::draw_hodo_background(tmp_args);
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

            super::drawing::draw_hodo_line(tmp_args);
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

            super::drawing::draw_active_readout(tmp_args);
        }

        cr.set_source_surface(&self.overlay_layer.borrow().clone(), 0.0, 0.0);
        cr.paint();

    }
}
