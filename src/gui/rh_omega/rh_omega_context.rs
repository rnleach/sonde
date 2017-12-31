use std::cell::{Cell, RefCell};

use cairo::{Context, Format, ImageSurface, Matrix, MatrixTrait, Operator};

use gtk::DrawingArea;
use gtk::prelude::*;

use gui::DrawingArgs;
use gui::plot_context::{GenericContext, PlotContext};

use app::config;
use coords::{DeviceCoords, DeviceRect, WPCoords, ScreenCoords, XYCoords, XYRect};

pub struct RHOmegaContext {
    x_zoom: Cell<f64>,
    generic: GenericContext,
    
    pub matrix: Cell<Matrix>,

    dirty_background: Cell<bool>,
    background_layer: RefCell<ImageSurface>,

    dirty_data: Cell<bool>,
    data_layer: RefCell<ImageSurface>,

    dirty_overlay: Cell<bool>,
    overlay_layer: RefCell<ImageSurface>,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            x_zoom: Cell::new(1.0),
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

    pub fn convert_wp_to_xy(coords: WPCoords) -> XYCoords {
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.p))
            / (f64::log10(config::MAXP) - f64::log10(config::MINP));

        // The + sign below looks weird, but is correct.
        let x = (coords.w + config::MAX_ABS_W) / (2.0 * config::MAX_ABS_W);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_wp(coords: XYCoords) -> WPCoords {
        use std::f64;

        let p = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP))
                + f64::log10(config::MAXP),
        );
        let w = coords.x * (2.0 * config::MAX_ABS_W) - config::MAX_ABS_W;

        WPCoords { w, p }
    }

    pub fn convert_screen_to_wp(&self, coords: ScreenCoords) -> WPCoords {
        let xy = self.convert_screen_to_xy(coords);
        RHOmegaContext::convert_xy_to_wp(xy)
    }

    pub fn convert_wp_to_screen(&self, coords: WPCoords) -> ScreenCoords {
        let xy = RHOmegaContext::convert_wp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_wp(&self, coords: DeviceCoords) -> WPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_wp(xy)
    }

    pub fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.generic.set_translate(translate);
    }
}

impl PlotContext for RHOmegaContext {
    fn set_device_rect(&self, rect: DeviceRect) {
        self.generic.set_device_rect(rect)
    }

    fn get_device_rect(&self) -> DeviceRect {
        self.generic.get_device_rect()
    }

    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        // Apply translation first
        let x = coords.x - self.generic.get_translate().x;
        let y = coords.y - self.generic.get_translate().y;

        // Apply scaling
        let x = x * self.x_zoom.get();
        let y = y * self.get_zoom_factor();

        ScreenCoords { x, y }
    }

    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x / self.x_zoom.get();
        let y = coords.y / self.get_zoom_factor();

        // Unapply translation
        let x = x + self.generic.get_translate().x;
        let y = y + self.generic.get_translate().y;

XYCoords { x, y }
    }

    fn get_xy_envelope(&self) -> XYRect {
        self.generic.get_xy_envelope()
    }

    fn set_xy_envelope(&self, new_envelope: XYRect) {
        self.generic.set_xy_envelope(new_envelope);
    }

    fn get_zoom_factor(&self) -> f64 {
        self.generic.get_zoom_factor()
    }

    fn set_zoom_factor(&self, new_zoom_factor: f64) {
        self.generic.set_zoom_factor(new_zoom_factor);
    }

    fn get_translate(&self) -> XYCoords {
        self.generic.get_translate()
    }

    fn set_translate(&self, new_translate: XYCoords) {
        self.generic.set_translate(new_translate);
    }

    fn get_left_button_pressed(&self) -> bool {
        self.generic.get_left_button_pressed()
    }

    fn set_left_button_pressed(&self, pressed: bool) {
        self.generic.set_left_button_pressed(pressed);
    }

    fn get_last_cursor_position(&self) -> Option<DeviceCoords> {
        self.generic.get_last_cursor_position()
    }

    fn set_last_cursor_position<U>(&self, new_position: U)
    where
        Option<DeviceCoords>: From<U>,
    {
        self.generic.set_last_cursor_position(new_position);
    }

    fn zoom_to_envelope(&self) {
        let xy_envelope = self.get_xy_envelope();

        let lower_left = xy_envelope.lower_left;
        self.set_translate(lower_left);

        let width = xy_envelope.upper_right.x - xy_envelope.lower_left.x;
        let width_scale = 1.0 / width;

        self.x_zoom.set(width_scale);
    }

    fn bound_view(&self) {
        let device_rect = self.get_device_rect();

        let bounds = DeviceCoords {
            col: device_rect.width,
            row: device_rect.height,
        };
        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy(device_rect.upper_left);
        let height = upper_left.y - lower_right.y;

        let mut translate = self.get_translate();
        if height < 1.0 {
            if translate.y < 0.0 {
                translate.y = 0.0;
            }
            let max_y = 1.0 - height;
            if translate.y > max_y {
                translate.y = max_y;
            }
        } else {
            translate.y = -(height - 1.0) / 2.0;
        }
        self.set_translate(translate);
    }
}

impl RHOmegaContext {
    
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
