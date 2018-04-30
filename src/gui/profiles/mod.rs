use std::rc::Rc;

use gtk;
use gtk::prelude::*;
use gtk::CheckButton;

use app::AppContextPointer;
use gui::{Drawable, Gui};

pub mod cloud;
pub mod rh_omega;
pub mod wind_speed;
pub mod lapse_rate;

pub use self::cloud::CloudContext;
pub use self::rh_omega::RHOmegaContext;
pub use self::wind_speed::WindSpeedContext;
pub use self::lapse_rate::LapseRateContext;

macro_rules! build_profile{
    ($p_box:ident, $label:expr, $c_box:ident, $drawing_area:expr, $acp:ident, $show_var:ident) => {

        let da = $drawing_area;
        $p_box.pack_start(&da, true, true, 0);
        let check_button = CheckButton::new_with_label($label);
        $c_box.pack_start(&check_button, false, false, 0);
        let show_da = $acp.config.borrow().$show_var;
        check_button.set_active(show_da);
        if show_da {
            da.show();
        } else {
            da.hide();
        }

        let ac = Rc::clone(&$acp);
        check_button.connect_toggled(move |button| {
            let button_state = button.get_active();
            ac.config.borrow_mut().$show_var = button_state;
            if button_state {
                da.show();
            } else {
                da.hide()
            }
        });
    };
}

pub fn set_up_profiles_box(gui: &Gui, acp: &AppContextPointer, box_spacing: i32) -> gtk::Box {
    let profile_box = gtk::Box::new(gtk::Orientation::Horizontal, box_spacing);
    let control_box = gtk::Box::new(gtk::Orientation::Vertical, box_spacing);

    build_profile!(
        profile_box,
        "RH-Omega",
        control_box,
        gui.get_rh_omega_area(),
        acp,
        show_rh_omega_frame
    );
    build_profile!(
        profile_box,
        "Clouds",
        control_box,
        gui.get_cloud_area(),
        acp,
        show_cloud_frame
    );
    build_profile!(
        profile_box,
        "Wind Spd",
        control_box,
        gui.get_wind_speed_profile_area(),
        acp,
        show_wind_speed_profile
    );

    build_profile!(
        profile_box,
        "Lapse rate",
        control_box,
        gui.get_lapse_rate_profile_area(),
        acp,
        show_lapse_rate_profile
    );

    profile_box.pack_start(&control_box, false, false, 0);
    profile_box.show_all();
    profile_box
}

macro_rules! draw_profile {
    ($config:ident, $da:expr, $show_var:ident) => {
        let da = $da;
        if $config.$show_var {
            da.show_all();
            da.queue_draw();
        } else {
            da.hide();
        }
    };
}
pub fn draw_profiles(gui: &Gui, acp: &AppContextPointer) {
    
    let config = acp.config.borrow();

    draw_profile!(config, gui.get_rh_omega_area(), show_rh_omega_frame);
    draw_profile!(config, gui.get_cloud_area(), show_cloud_frame);
    draw_profile!(config, gui.get_wind_speed_profile_area(), show_wind_speed_profile);
    draw_profile!(config, gui.get_lapse_rate_profile_area(), show_lapse_rate_profile);
}

pub fn initialize_profiles(gui: &Gui, acp: &AppContextPointer) {
    RHOmegaContext::set_up_drawing_area(&gui.get_rh_omega_area(), acp);
    CloudContext::set_up_drawing_area(&gui.get_cloud_area(), acp);
    WindSpeedContext::set_up_drawing_area(&gui.get_wind_speed_profile_area(), acp);
    LapseRateContext::set_up_drawing_area(&gui.get_lapse_rate_profile_area(), acp);
}
