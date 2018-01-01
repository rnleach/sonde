use gui::DrawingArgs;
use gui::plot_context::{Drawable, GenericContext, HasGenericContext, PlotContextExt};

use app::config;
use coords::{SDCoords, ScreenCoords, XYCoords};

pub struct HodoContext {
    generic: GenericContext,
}

impl HodoContext {
    pub fn new() -> Self {
        HodoContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_sd_to_xy(coords: SDCoords) -> XYCoords {
        let radius = coords.speed / 2.0 / config::MAX_SPEED;
        let angle = (270.0 - coords.dir).to_radians();

        let x = radius * angle.cos() + 0.5;
        let y = radius * angle.sin() + 0.5;
        XYCoords { x, y }
    }

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

impl PlotContextExt for HodoContext {}

impl Drawable for HodoContext {
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
