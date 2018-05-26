use std::rc::Rc;

use gtk::prelude::*;
use gtk::{self, CheckButton, DrawingArea};

use app::{AppContext, AppContextPointer};
use errors::SondeError;
use gui::{Drawable, PlotContext};

pub mod cloud;
pub mod lapse_rate;
pub mod rh_omega;
pub mod wind_speed;

pub use self::cloud::CloudContext;
pub use self::lapse_rate::LapseRateContext;
pub use self::rh_omega::RHOmegaContext;
pub use self::wind_speed::WindSpeedContext;

macro_rules! build_profile {
    ($c_box:ident, $drawing_area:expr, $acp:ident, $label:expr, $show_var:ident) => {
        let da = $drawing_area;
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
    (
        $c_box:ident,
        $drawing_area:expr,
        $acp:ident,
        $d_context:ident,
        $label:expr,
        $show_var:ident,
        $label2:expr,
        $show_var2:ident
    ) => {
        let da = $drawing_area;

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
    (
        $c_box:ident,
        $drawing_area:expr,
        $acp:ident,
        $d_context:ident,
        $label:expr,
        $show_var:ident,
        $label2:expr,
        $show_var2:ident,
        $label3:expr,
        $show_var3:ident
    ) => {
        let da = $drawing_area;

        let check_button = CheckButton::new_with_label($label);
        $c_box.pack_start(&check_button, false, false, 0);
        let show_da1 = $acp.config.borrow().$show_var;
        check_button.set_active(show_da1);

        let check_button2 = CheckButton::new_with_label($label2);
        $c_box.pack_start(&check_button2, false, false, 0);
        let show_da2 = $acp.config.borrow().$show_var2;
        check_button2.set_active(show_da2);

        let check_button3 = CheckButton::new_with_label($label3);
        $c_box.pack_start(&check_button3, false, false, 0);
        let show_da3 = $acp.config.borrow().$show_var3;
        check_button3.set_active(show_da3);

        let show_da = show_da1 || show_da2 || show_da3;

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
            let show_da =
                button_state || ac.config.borrow().$show_var2 || ac.config.borrow().$show_var3;
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
            let show_da =
                button_state || ac.config.borrow().$show_var || ac.config.borrow().$show_var3;
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
        check_button3.connect_toggled(move |button| {
            let button_state = button.get_active();
            ac.config.borrow_mut().$show_var3 = button_state;
            let show_da =
                button_state || ac.config.borrow().$show_var || ac.config.borrow().$show_var2;
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

pub fn set_up_profiles_box(acp: &AppContextPointer) -> Result<(), SondeError> {
    let control_box: gtk::Box = acp.fetch_widget("profile_control_box")?;

    let rh_omega: DrawingArea = acp.fetch_widget("rh_omega_area")?;
    let cloud: DrawingArea = acp.fetch_widget("cloud_area")?;
    let wind_speed: DrawingArea = acp.fetch_widget("wind_speed_area")?;
    let lapse_rate: DrawingArea = acp.fetch_widget("lapse_rate_area")?;

    build_profile!(
        control_box,
        rh_omega,
        acp,
        rh_omega,
        "RH",
        show_rh,
        "Omega",
        show_omega
    );
    build_profile!(control_box, cloud, acp, "Clouds", show_cloud_frame);
    build_profile!(
        control_box,
        wind_speed,
        acp,
        "Wind Spd",
        show_wind_speed_profile
    );

    build_profile!(
        control_box,
        lapse_rate,
        acp,
        lapse_rate,
        "Lapse rate",
        show_lapse_rate_profile,
        "Sfc to * lapse rate",
        show_sfc_avg_lapse_rate_profile,
        "Ml to * lapse rate",
        show_ml_avg_lapse_rate_profile
    );

    Ok(())
}

pub fn draw_profiles(acp: &AppContext) {
    let config = acp.config.borrow();

    const DRAWING_AREAS: [&str; 4] = [
        "rh_omega_area",
        "cloud_area",
        "wind_speed_area",
        "lapse_rate_area",
    ];

    let do_draw = [
        config.show_rh || config.show_omega,
        config.show_cloud_frame,
        config.show_wind_speed_profile,
        config.show_lapse_rate_profile || config.show_sfc_avg_lapse_rate_profile,
    ];

    for (&da, &show) in izip!(DRAWING_AREAS.iter(), do_draw.iter()) {
        if let Ok(da) = acp.fetch_widget::<DrawingArea>(da) {
            if show {
                da.show();
                da.queue_draw();
            } else {
                da.hide();
            }
        }
    }
}

pub fn initialize_profiles(acp: &AppContextPointer) -> Result<(), SondeError> {
    RHOmegaContext::set_up_drawing_area(acp)?;
    CloudContext::set_up_drawing_area(acp)?;
    WindSpeedContext::set_up_drawing_area(acp)?;
    LapseRateContext::set_up_drawing_area(acp)?;

    set_up_profiles_box(acp)?;

    Ok(())
}
