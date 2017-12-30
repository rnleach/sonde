use std::cell::Cell;

use gui::plot_context::{GenericContext, PlotContext};
use app::config;

use coords::{DeviceCoords, DeviceRect, ScreenCoords, WPCoords, XYCoords, XYRect};

pub struct RHOmegaContext {
    x_zoom: Cell<f64>,
    skew_t_scale: Cell<f64>,
    generic: GenericContext,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            x_zoom: Cell::new(1.0),
            skew_t_scale: Cell::new(1.0),
            generic: GenericContext::new(),
        }
    }

    /// Conversion from omega (w) and pressure (p) to (x,y) coords
    pub fn convert_wp_to_xy(coords: WPCoords) -> XYCoords {
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.p))
            / (f64::log10(config::MAXP) - f64::log10(config::MINP));

        // The + sign below looks weird, but is correct.
        let x = (coords.w + config::MAX_ABS_W) / (2.0 * config::MAX_ABS_W);

        XYCoords { x, y }
    }

    /// Conversion from `XYCoords` to `WPCoords`
    pub fn convert_xy_to_wp(coords: XYCoords) -> WPCoords {
        use std::f64;

        let p = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP))
                + f64::log10(config::MAXP),
        );
        let w = coords.x * (2.0 * config::MAX_ABS_W) - config::MAX_ABS_W;

        WPCoords { w, p }
    }

    /// Converstion from screen to `WPCoords`
    pub fn convert_screen_to_wp(&self, coords: ScreenCoords) -> WPCoords {
        let xy = self.convert_screen_to_xy(coords);
        RHOmegaContext::convert_xy_to_wp(xy)
    }

    /// Conversion from `DeviceCoords` to `WPCoords`
    // pub fn convert_device_to_wp(&self, coords: DeviceCoords) -> WPCoords {
    //     let mut screen = self.convert_device_to_screen(coords);
    //     self.convert_screen_to_wp(screen)
    // }
    /// Conversion from omega/pressure to screen coordinates.
    pub fn convert_wp_to_screen(&self, coords: WPCoords) -> ScreenCoords {
        let xy = RHOmegaContext::convert_wp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.generic.set_translate(translate);
    }

    pub fn set_skew_t_scale(&self, scale: f64) {
        self.skew_t_scale.set(scale);
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
        let y = y * self.get_zoom_factor() / self.scale_factor() * self.skew_t_scale.get();

        ScreenCoords { x, y }
    }

    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x / self.x_zoom.get();
        let y = coords.y / self.get_zoom_factor() * self.scale_factor() / self.skew_t_scale.get();

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
