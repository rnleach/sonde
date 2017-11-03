use gtk;
use gtk::prelude::*;
use gtk::{Frame, Notebook, ScrolledWindow, ColorButton, CheckButton};
use gdk::RGBA;

use app::AppContextPointer;


pub fn set_up_control_area(control_area: &Notebook, acp: AppContextPointer) {

    control_area.set_hexpand(true);
    control_area.set_vexpand(true);
    control_area.set_scrollable(true);

    let data_options = make_data_option_frame(acp.clone());
    control_area.add(&data_options);
    control_area.set_tab_label_text(&data_options, "Data");

    let background_options = make_background_frame(acp.clone());
    control_area.add(&background_options);
    control_area.set_tab_label_text(&background_options, "Background");
}

const PADDING: u32 = 2;
const BOX_SPACING: i32 = 5;

macro_rules! build_config_color_and_check {
    ($v_box:ident, $label:expr, $acp:ident, $show_var:ident, $color_var:ident) => {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
        let check = CheckButton::new_with_label($label);
        let color = ColorButton::new();

        // Inner scope to borrow acp
        {
            let ac = $acp.borrow();
            check.set_active(ac.config.$show_var);

            let rgba = ac.config.$color_var;
            color.set_rgba(&RGBA{red:rgba.0, green:rgba.1, blue:rgba.2, alpha:rgba.3});
        }

        // Create check button callback
        let acp1 = $acp.clone();
        check.connect_toggled(move|button|{
            let mut ac = acp1.borrow_mut();
            ac.config.$show_var = button.get_active();

            if let Some(ref gui) = ac.gui {
                gui.get_sounding_area().queue_draw();
            }
        });

        // Create color button callback
        let acp2 = $acp.clone();
        ColorButtonExt::connect_property_rgba_notify(&color, move|button|{
            let mut ac = acp2.borrow_mut();
            let rgba = button.get_rgba();

            ac.config.$color_var = (rgba.red, rgba.green, rgba.blue, rgba.alpha);

            if let Some(ref gui) = ac.gui {
                gui.get_sounding_area().queue_draw();
            }
        });

        // Layout
        hbox.pack_start(&color, false, true, PADDING);
        hbox.pack_start(&check, false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

fn make_data_option_frame(acp: AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);

    build_config_color_and_check!(v_box, "Wet Bulb", acp, show_wet_bulb, wet_bulb_rgba);
    build_config_color_and_check!(v_box, "Dew Point", acp, show_dew_point, dew_point_rgba);
    build_config_color_and_check!(
        v_box,
        "Temperature",
        acp,
        show_temperature,
        temperature_rgba
    );
    build_config_color_and_check!(v_box, "Wind", acp, show_wind_profile, wind_rgba);

    //
    // Layout boxes in the frame
    //
    f.add(&v_box);
    let sw = ScrolledWindow::new(None, None);
    sw.add(&f);

    sw
}


fn make_background_frame(acp: AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);

    // First set is background fills
    let fills_frame = gtk::Frame::new(Some("Shading"));
    let fills_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    fills_frame.add(&fills_box);

    // Fills buttons
    build_config_color_and_check!(
        fills_box,
        "Dendritic Zone",
        acp,
        show_dendritic_zone,
        dendritic_zone_rgba
    );
    build_config_color_and_check!(
        fills_box,
        "Hail Growth Zone",
        acp,
        show_hail_zone,
        hail_zone_rgba
    );
    build_config_color_and_check!(
        fills_box,
        "Striping",
        acp,
        show_background_bands,
        background_band_rgba
    );

    // Second set is background lines
    let lines_frame = gtk::Frame::new(Some("Lines"));
    let lines_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    lines_frame.add(&lines_box);

    // Lines buttons
    build_config_color_and_check!(
        lines_box,
        "Dry Adiabats",
        acp,
        show_isentrops,
        isentrop_rgba
    );
    build_config_color_and_check!(
        lines_box,
        "Moist Adiabats",
        acp,
        show_iso_theta_e,
        iso_theta_e_rgba
    );
    build_config_color_and_check!(
        lines_box,
        "Mixing Ratio",
        acp,
        show_iso_mixing_ratio,
        iso_mixing_ratio_rgba
    );
    build_config_color_and_check!(lines_box, "Temperature", acp, show_isotherms, isotherm_rgba);
    build_config_color_and_check!(lines_box, "Pressure", acp, show_isobars, isobar_rgba);

    //
    // Layout boxes in the frame
    //
    f.add(&v_box);
    v_box.pack_start(&lines_frame, true, true, PADDING);
    v_box.pack_start(&fills_frame, true, true, PADDING);
    let sw = ScrolledWindow::new(None, None);
    sw.add(&f);

    sw
}
