use std::rc::Rc;

use gtk;
use gtk::prelude::*;
use gtk::CheckButton;

use app::AppContextPointer;
use gui::{Drawable, Gui, PlotContext};

pub mod cloud;
pub mod rh_omega;
pub mod wind_speed;
pub mod lapse_rate;

pub use self::cloud::CloudContext;
pub use self::rh_omega::RHOmegaContext;
pub use self::wind_speed::WindSpeedContext;
pub use self::lapse_rate::LapseRateContext;

macro_rules! build_profile{
    ($p_box:ident, $c_box:ident, $drawing_area:expr, $acp:ident, $label:expr, $show_var:ident) => {

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
    ($p_box:ident, $c_box:ident, $drawing_area:expr, $acp:ident, $d_context:ident, $label:expr,
        $show_var:ident, $label2:expr, $show_var2:ident) => {

        let da = $drawing_area;
        $p_box.pack_start(&da, true, true, 0);

        let check_button = CheckButton::new_with_label($label);
        $c_box.pack_start(&check_button, false, false, 0);
        let show_da1 = $acp.config.borrow().$show_var;
        check_button.set_active(show_da1);

        let check_button2 = CheckButton::new_with_label($label2);
        $c_box.pack_start(&check_button2, false, false, 0);
        let show_da2 = $acp.config.borrow().$show_var2;
        check_button2.set_active(show_da2);

        let show_da = show_da1 || show_da2;

        if show_da {
            da.show();
        } else {
            da.hide();
        }

        let ac = Rc::clone(&$acp);
        let dac = da.clone();
        check_button.connect_toggled(move |button| {
            let button_state = button.get_active();
            ac.config.borrow_mut().$show_var = button_state;
            let show_da = button_state || ac.config.borrow().$show_var2;
            if show_da {
                dac.show();
                dac.queue_draw();
                ac.$d_context.mark_data_dirty();
            } else {
                dac.hide()
            }
        });

        let ac = Rc::clone(&$acp);
        let dac = da.clone();
        check_button2.connect_toggled(move |button| {
            let button_state = button.get_active();
            ac.config.borrow_mut().$show_var2 = button_state;
            let show_da = button_state || ac.config.borrow().$show_var;
            if show_da {
                dac.show();
                dac.queue_draw();
                ac.$d_context.mark_data_dirty();
            } else {
                dac.hide()
            }
        });
    };
}

pub fn set_up_profiles_box(gui: &Gui, acp: &AppContextPointer, box_spacing: i32) -> gtk::Box {
    let profile_box = gtk::Box::new(gtk::Orientation::Horizontal, box_spacing);
    let control_box = gtk::Box::new(gtk::Orientation::Vertical, box_spacing);

    build_profile!(
        profile_box,
        control_box,
        gui.get_rh_omega_area(),
        acp,
        rh_omega,
        "RH",
        show_rh,
        "Omega",
        show_omega
    );
    build_profile!(
        profile_box,
        control_box,
        gui.get_cloud_area(),
        acp,
        "Clouds",
        show_cloud_frame
    );
    build_profile!(
        profile_box,
        control_box,
        gui.get_wind_speed_profile_area(),
        acp,
        "Wind Spd",
        show_wind_speed_profile
    );

    build_profile!(
        profile_box,
        control_box,
        gui.get_lapse_rate_profile_area(),
        acp,
        lapse_rate,
        "Lapse rate",
        show_lapse_rate_profile,
        "Theta-e lapse rate",
        show_theta_e_lapse_rate_profile
    );

    profile_box.pack_start(&control_box, false, false, 0);
    profile_box.show_all();
    profile_box
}

macro_rules! draw_profile {
    ($da:expr, $show:expr) => {
        let da = $da;
        if $show {
            da.show();
            da.queue_draw();
        } else {
            da.hide();
        }
    };
}
pub fn draw_profiles(gui: &Gui, acp: &AppContextPointer) {
    let config = acp.config.borrow();

    draw_profile!(gui.get_rh_omega_area(), config.show_rh || config.show_omega);
    draw_profile!(gui.get_cloud_area(), config.show_cloud_frame);
    draw_profile!(
        gui.get_wind_speed_profile_area(),
        config.show_wind_speed_profile
    );
    draw_profile!(
        gui.get_lapse_rate_profile_area(),
        config.show_lapse_rate_profile || config.show_theta_e_lapse_rate_profile
    );
}

pub fn initialize_profiles(gui: &Gui, acp: &AppContextPointer) {
    RHOmegaContext::set_up_drawing_area(&gui.get_rh_omega_area(), acp);
    CloudContext::set_up_drawing_area(&gui.get_cloud_area(), acp);
    WindSpeedContext::set_up_drawing_area(&gui.get_wind_speed_profile_area(), acp);
    LapseRateContext::set_up_drawing_area(&gui.get_lapse_rate_profile_area(), acp);
}
