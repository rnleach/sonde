use gtk;
use gtk::prelude::*;
use gtk::{Frame, Notebook, ScrolledWindow};

use app::AppContextPointer;


pub fn set_up_control_area(control_area: &Notebook, acp: AppContextPointer) {

    control_area.set_hexpand(true);
    control_area.set_vexpand(true);
    control_area.set_scrollable(true);

    let background_options = make_background_frame(acp.clone());
    control_area.add(&background_options);
    control_area.set_tab_label_text(&background_options, "Background");
}

const PADDING: u32 = 2;
const BOX_SPACING: i32 = 5;

// TODO: Add color button, put all in HBOX, then add to *V_BOX, will need color variable to change too

macro_rules! build_config_check_button {
    ($button:ident, $label:expr, $acp:ident, $setting:ident) => {
        let $button = gtk::CheckButton::new_with_label($label);
        {
            let ac = $acp.borrow();
            $button.set_active(ac.config.$setting);
        }
        let acp1 = $acp.clone();
        $button.connect_toggled(move|button|{
            let mut ac = acp1.borrow_mut();
            ac.config.$setting = button.get_active();

            if let Some(ref gui) = ac.gui {
                gui.get_sounding_area().queue_draw();
            }
        });
    };
}
fn make_background_frame(acp: AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout in columns
    let h_box = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
    h_box.set_baseline_position(gtk::BaselinePosition::Top);

    // First column is background fills
    let fills_frame = gtk::Frame::new(Some("Shading"));
    let fills_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);

    // Fills buttons
    build_config_check_button!(dendritic_zone, "Dendritic Zone", acp, show_dendritic_zone);
    build_config_check_button!(hail_zone, "Hail Growth Zone", acp, show_hail_zone);
    build_config_check_button!(stripes, "Striping", acp, show_background_bands);

    // Pack fills column
    fills_frame.add(&fills_box);
    fills_box.pack_start(&dendritic_zone, false, false, PADDING);
    fills_box.pack_start(&hail_zone, false, false, PADDING);
    fills_box.pack_start(&stripes, false, false, PADDING);

    // Second column is background lines
    let lines_frame = gtk::Frame::new(Some("Lines"));
    let lines_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);

    // Lines buttons
    build_config_check_button!(isentrops, "Dry Adiabats", acp, show_isentrops);
    build_config_check_button!(moist_adiabats, "Moist Adiabats", acp, show_iso_theta_e);
    build_config_check_button!(mixing_ratio, "Mixing Ratio", acp, show_iso_mixing_ratio);
    build_config_check_button!(isotherms, "Temperature", acp, show_isotherms);
    build_config_check_button!(isobars, "Pressure", acp, show_isobars);

    // Pack the lines column
    lines_frame.add(&lines_box);
    lines_box.pack_start(&isentrops, false, false, PADDING);
    lines_box.pack_start(&moist_adiabats, false, false, PADDING);
    lines_box.pack_start(&mixing_ratio, false, false, PADDING);
    lines_box.pack_start(&isotherms, false, false, PADDING);
    lines_box.pack_start(&isobars, false, false, PADDING);

    //
    // Layout boxes in the frame
    //
    f.add(&h_box);
    h_box.pack_start(&lines_frame, true, true, PADDING);
    h_box.pack_start(&fills_frame, true, true, PADDING);
    let sw = ScrolledWindow::new(None, None);
    sw.add(&f);

    sw
}
