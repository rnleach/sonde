use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
    gui::Drawable,
};
use gtk::{prelude::*, DrawingArea};

pub mod cloud;
pub mod rh_omega;
pub mod wind_speed;

pub use self::cloud::CloudContext;
pub use self::rh_omega::RHOmegaContext;
pub use self::wind_speed::WindSpeedContext;

pub fn draw_profiles(acp: &AppContext) {
    const DRAWING_AREAS: [&str; 3] = ["rh_omega_area", "cloud_area", "wind_speed_area"];

    for &da in DRAWING_AREAS.iter() {
        if let Ok(da) = acp.fetch_widget::<DrawingArea>(da) {
            da.queue_draw();
        }
    }
}

pub fn initialize_profiles(acp: &AppContextPointer) -> Result<(), SondeError> {
    RHOmegaContext::set_up_drawing_area(acp)?;
    CloudContext::set_up_drawing_area(acp)?;
    WindSpeedContext::set_up_drawing_area(acp)?;

    Ok(())
}
