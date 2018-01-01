use std::cell::Cell;

use cairo::{Context, ImageSurface, Matrix, Operator};

use gui::DrawingArgs;
use gui::plot_context::{GenericContext, PlotContext};

use app::config;
use coords::{DeviceCoords, DeviceRect, ScreenCoords, WPCoords, XYCoords, XYRect};

pub struct RHOmegaContext {
    x_zoom: Cell<f64>,
    generic: GenericContext,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            x_zoom: Cell::new(1.0),
            generic: GenericContext::new(),
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

    fn get_matrix(&self) -> Matrix {
        self.generic.get_matrix()
    }

    fn set_matrix(&self, matrix: Matrix) {
        self.generic.set_matrix(matrix);
    }

    fn mark_background_dirty(&self) {
        self.generic.mark_background_dirty();
    }

    fn clear_background_dirty(&self) {
        self.generic.clear_background_dirty();
    }

    fn is_background_dirty(&self) -> bool {
        self.generic.is_background_dirty()
    }

    fn mark_data_dirty(&self) {
        self.generic.mark_data_dirty();
    }

    fn clear_data_dirty(&self) {
        self.generic.clear_data_dirty();
    }

    fn is_data_dirty(&self) -> bool {
        self.generic.is_data_dirty()
    }

    fn mark_overlay_dirty(&self) {
        self.generic.mark_overlay_dirty();
    }

    fn clear_overlay_dirty(&self) {
        self.generic.clear_overlay_dirty();
    }

    fn is_overlay_dirty(&self) -> bool {
        self.generic.is_overlay_dirty()
    }

    fn get_background_layer(&self) -> ImageSurface {
        self.generic.get_background_layer()
    }

    fn set_background_layer(&self, new_surface: ImageSurface) {
        self.generic.set_background_layer(new_surface);
    }

    fn get_data_layer(&self) -> ImageSurface {
        self.generic.get_data_layer()
    }

    fn set_data_layer(&self, new_surface: ImageSurface) {
        self.generic.set_data_layer(new_surface);
    }

    fn get_overlay_layer(&self) -> ImageSurface {
        self.generic.get_overlay_layer()
    }

    fn set_overlay_layer(&self, new_surface: ImageSurface) {
        self.generic.set_overlay_layer(new_surface);
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
    pub fn draw_background_cached(&self, args: DrawingArgs) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if self.is_background_dirty() {
            let tmp_cr = Context::new(&self.get_background_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            let rgba = config.background_rgba;
            tmp_cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.bound_view();

            super::drawing::draw_background(tmp_args);

            self.clear_background_dirty();
        }

        cr.set_source_surface(&self.get_background_layer(), 0.0, 0.0);
        cr.paint();
    }

    pub fn draw_data_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_data_dirty() {
            let tmp_cr = Context::new(&self.get_data_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            super::drawing::draw_data(tmp_args);

            self.clear_data_dirty();
        }

        cr.set_source_surface(&self.get_data_layer(), 0.0, 0.0);
        cr.paint();
    }

    pub fn draw_overlay_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_overlay_dirty() {
            let tmp_cr = Context::new(&self.get_overlay_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            super::drawing::draw_overlays(tmp_args);

            self.clear_overlay_dirty();
        }

        cr.set_source_surface(&self.get_overlay_layer(), 0.0, 0.0);
        cr.paint();
    }
}
