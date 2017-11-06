#![macro_use]
use gtk::prelude::*;
use gtk::Notebook;

use app::AppContextPointer;

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
            ac.queue_draw_skew_t_rh_omega();
        });

        // Create color button callback
        let acp2 = $acp.clone();
        ColorButtonExt::connect_property_rgba_notify(&color, move|button|{
            let mut ac = acp2.borrow_mut();
            let rgba = button.get_rgba();

            ac.config.$color_var = (rgba.red, rgba.green, rgba.blue, rgba.alpha);
            ac.queue_draw_skew_t_rh_omega();
        });

        // Layout
        hbox.pack_end(&color, false, true, PADDING);
        hbox.pack_start(&check, false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

mod data_options;
mod background_options;

const PADDING: u32 = 2;
const BOX_SPACING: i32 = 5;

pub fn set_up_control_area(control_area: &Notebook, acp: AppContextPointer) {

    control_area.set_hexpand(true);
    control_area.set_vexpand(true);
    control_area.set_scrollable(true);

    let data_options = data_options::make_data_option_frame(acp.clone());
    control_area.add(&data_options);
    control_area.set_tab_label_text(&data_options, "Data");

    let background_options = background_options::make_background_frame(acp.clone());
    control_area.add(&background_options);
    control_area.set_tab_label_text(&background_options, "Background");
}
