
use super::plot_context::{PlotContext, GenericContext};
use app::config;

use coords::{DeviceCoords, ScreenCoords, WPCoords, XYCoords, XYRect};

pub struct RHOmegaContext {
    // Bound for the omega plot
    pub max_abs_omega: f64,

    pub skew_t_scale_factor: f64,

    generic: GenericContext,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            max_abs_omega: config::MAX_ABS_W,
            skew_t_scale_factor: 1.0,

            generic: GenericContext::new(),
        }
    }

    /// Conversion from omega (w) and pressure (p) to (x,y) coords
    pub fn convert_wp_to_xy(&self, coords: WPCoords) -> XYCoords {
        use app::config;
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.p)) /
            (f64::log10(config::MAXP) - f64::log10(config::MINP));

        // The + sign below looks weird, but is correct.
        let x = (coords.w + self.max_abs_omega) / (2.0 * self.max_abs_omega);

        XYCoords { x, y }
    }

    /// Conversion from `XYCoords` to `WPCoords`
    pub fn convert_xy_to_wp(&self, coords: XYCoords) -> WPCoords {
        use app::config;
        use std::f64;

        let p = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP)) +
                f64::log10(config::MAXP),
        );
        let w = coords.x * (2.0 * self.max_abs_omega) - self.max_abs_omega;

        WPCoords { w, p }
    }

    /// Converstion from screen to `WPCoords`
    pub fn convert_screen_to_wp(&self, coords: ScreenCoords) -> WPCoords {
        let xy = self.convert_screen_to_xy(coords);
        self.convert_xy_to_wp(xy)
    }

    /// Conversion from `DeviceCoords` to `WPCoords`
    pub fn convert_device_to_wp(&self, coords: DeviceCoords) -> WPCoords {
        let screen = self.convert_device_to_screen(coords);
        self.convert_screen_to_wp(screen)
    }

    /// Conversion from omega/pressure to screen coordinates.
    pub fn convert_wp_to_screen(&self, coords: WPCoords) -> ScreenCoords {
        let xy = self.convert_wp_to_xy(coords);

        self.convert_xy_to_screen(xy)
    }

    /// Get maximum absolute omega
    pub fn get_max_abs_omega(&self) -> f64 {
        self.max_abs_omega
    }
}

impl PlotContext for RHOmegaContext {
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {

        // Apply translation first
        let x = coords.x;
        let y = coords.y - self.generic.get_translate().y;

        // Apply scaling
        let x = x / self.skew_t_scale_factor * self.scale_factor();
        let y = self.get_zoom_factor() * y;
        ScreenCoords { x, y }
    }

    /// Conversion from device to screen coordinates.
    fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords {
        let scale_factor = self.skew_t_scale_factor;
        ScreenCoords {
            x: coords.col / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            y: -(coords.row / scale_factor) +
                f64::from(self.generic.get_device_height()) / scale_factor,
        }
    }

    /// Conversion from screen coords to xy
    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x * self.skew_t_scale_factor / self.scale_factor();
        let y = coords.y / self.get_zoom_factor();

        // Unapply translation
        let x = x;
        let y = y + self.generic.get_translate().y;

        XYCoords { x, y }
    }

    fn set_device_width(&mut self, new_width: i32) {
        self.generic.set_device_width(new_width);
    }

    fn set_device_height(&mut self, new_height: i32) {
        self.generic.set_device_height(new_height);
    }

    fn get_device_width(&self) -> i32 {
        self.generic.get_device_width()
    }

    fn get_device_height(&self) -> i32 {
        self.generic.get_device_height()
    }

    fn get_xy_envelope(&self) -> XYRect {
        self.generic.get_xy_envelope()
    }

    fn set_xy_envelope(&mut self, new_envelope: XYRect) {
        self.generic.set_xy_envelope(new_envelope);
    }

    fn get_zoom_factor(&self) -> f64 {
        self.generic.get_zoom_factor()
    }

    fn set_zoom_factor(&mut self, new_zoom_factor: f64) {
        self.generic.set_zoom_factor(new_zoom_factor);
    }

    fn get_translate(&self) -> XYCoords {
        self.generic.get_translate()
    }

    fn set_translate(&mut self, new_translate: XYCoords) {
        self.generic.set_translate(new_translate);
    }

    fn get_left_button_pressed(&self) -> bool {
        self.generic.get_left_button_pressed()
    }

    fn set_left_button_pressed(&mut self, pressed: bool) {
        self.generic.set_left_button_pressed(pressed);
    }

    fn get_last_cursor_position(&self) -> Option<DeviceCoords> {
        self.generic.get_last_cursor_position()
    }

    fn set_last_cursor_position<U>(&mut self, new_position: U)
    where
        Option<DeviceCoords>: From<U>,
    {
        self.generic.set_last_cursor_position(new_position);
    }

    fn get_label_padding(&self) -> f64 {
        self.generic.get_label_padding()
    }

    fn set_label_padding(&mut self, new_padding: f64) {
        self.generic.set_label_padding(new_padding);
    }

    fn get_edge_padding(&self) -> f64 {
        self.generic.get_edge_padding()
    }

    fn set_edge_padding(&mut self, new_padding: f64) {
        self.generic.set_edge_padding(new_padding);
    }
}
