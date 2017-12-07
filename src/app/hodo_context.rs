
use super::plot_context::{PlotContext, GenericContext, HasGenericContext};

use app::config;
use coords::{ScreenCoords, SDCoords, XYCoords};

pub struct HodoContext {
    generic: GenericContext,
}

impl HodoContext {
    // Create a new instance of HodoContext
    pub fn new() -> Self {
        HodoContext { generic: GenericContext::new() }
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

    fn get_generic_context_mut(&mut self) -> &mut GenericContext {
        &mut self.generic
    }
}
