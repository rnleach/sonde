use gui::DrawingArgs;
use gui::plot_context::{Drawable, GenericContext, HasGenericContext, PlotContextExt};

use app::config;
use coords::{DeviceCoords, ScreenCoords, TPCoords, XYCoords};

pub struct SkewTContext {
    generic: GenericContext,
}

impl SkewTContext {
    pub fn new() -> Self {
        SkewTContext {
            generic: GenericContext::new(),
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

impl PlotContextExt for SkewTContext {}

impl Drawable for SkewTContext {
    fn draw_background(&self, args: DrawingArgs) {
        super::drawing::draw_background(args);
    }

    fn draw_data(&self, args: DrawingArgs) {
        super::drawing::draw_data(args);
    }

    fn draw_overlays(&self, args: DrawingArgs) {
        super::drawing::draw_overlays(args);
    }
}
