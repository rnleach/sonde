
use super::plot_context::{PlotContext, GenericContext, HasGenericContext};

use coords::{ScreenCoords, SDCoords, XYCoords};

pub struct HodoContext {
    generic: GenericContext,

    // Maximum plot value for the speed
    pub max_speed: f64,
}

impl HodoContext {
    // Create a new instance of HodoContext
    pub fn new() -> Self {
        HodoContext {
            generic: GenericContext::new(),
            max_speed: 100.0,
        }
    }

    /// Conversion from speed and direction to (x,y) coords
    pub fn convert_sd_to_xy(&self, coords: SDCoords) -> XYCoords {
        let radius = coords.speed / 2.0 / self.max_speed;
        let angle = (270.0 - coords.dir).to_radians();

        let x = radius * angle.cos() + 0.5;
        let y = radius * angle.sin() + 0.5;
        XYCoords { x, y }
    }

    /// Conversion from speed and direction to (x,y) coords
    pub fn convert_sd_to_screen(&self, coords: SDCoords) -> ScreenCoords {
        let xy = self.convert_sd_to_xy(coords);
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
