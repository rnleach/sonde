use gtk;
use gtk::prelude::*;

use app::AppContextPointer;
use gui::{Gui, Drawable};

pub mod cloud;
pub mod rh_omega;
pub mod wind_speed;

pub use self::cloud::CloudContext;
pub use self::rh_omega::RHOmegaContext;
pub use self::wind_speed::WindSpeedContext;

pub fn set_up_profiles_box(gui: &Gui, acp: &AppContextPointer, box_spacing: i32) -> gtk::Box {
    // Set up hbox for profiles
    let profile_box = gtk::Box::new(gtk::Orientation::Horizontal, box_spacing);
    profile_box.pack_start(&gui.get_rh_omega_area(), true, true, 0);
    profile_box.pack_start(&gui.get_cloud_area(), true, true, 0);
    profile_box.pack_start(&gui.get_wind_speed_profile_area(), true, true, 0);

    profile_box
}

pub fn initialize_profiles(gui: &Gui, acp: &AppContextPointer) {
    RHOmegaContext::set_up_drawing_area(&gui.get_rh_omega_area(), acp);
    CloudContext::set_up_drawing_area(&gui.get_cloud_area(), acp);
    WindSpeedContext::set_up_drawing_area(&gui.get_wind_speed_profile_area(), acp);
}
