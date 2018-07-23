use gtk::prelude::*;
use gtk::DrawingArea;

use app::{AppContext, AppContextPointer};
use errors::SondeError;
use gui::Drawable;

pub mod cloud;
pub mod rh_omega;
pub mod wind_speed;

pub use self::cloud::CloudContext;
pub use self::rh_omega::RHOmegaContext;
pub use self::wind_speed::WindSpeedContext;

pub fn draw_profiles(acp: &AppContext) {
    let config = acp.config.borrow();

    const DRAWING_AREAS: [&str; 3] = ["rh_omega_area", "cloud_area", "wind_speed_area"];

    let do_draw = [
        config.show_rh || config.show_omega,
        config.show_cloud_frame,
        config.show_wind_speed_profile,
    ];

    for (&da, &show) in izip!(DRAWING_AREAS.iter(), do_draw.iter()) {
        if let Ok(da) = acp.fetch_widget::<DrawingArea>(da) {
            if show {
                da.queue_draw();
            }
        }
    }
}

pub fn initialize_profiles(acp: &AppContextPointer) -> Result<(), SondeError> {
    RHOmegaContext::set_up_drawing_area(acp)?;
    CloudContext::set_up_drawing_area(acp)?;
    WindSpeedContext::set_up_drawing_area(acp)?;

    Ok(())
}
