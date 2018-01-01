use cairo::{Context, Operator};

use gui::DrawingArgs;
use gui::plot_context::{GenericContext, HasGenericContext, PlotContext};

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

impl SkewTContext {
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
