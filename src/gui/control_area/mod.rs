#![macro_use]

use gtk::prelude::*;
use gtk::Notebook;

use app::AppContextPointer;

macro_rules! build_config_color_and_check {
    ($v_box:ident, $label:expr, $acp_in:expr, $show_var:ident, $color_var:ident) => {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
        let check = CheckButton::new_with_label($label);
        let color = ColorButton::new();

        // Inner scope to borrow acp
        {
            let config = $acp_in.config.borrow();
            check.set_active(config.$show_var);

            let rgba = config.$color_var;
            color.set_rgba(&RGBA{red:rgba.0, green:rgba.1, blue:rgba.2, alpha:rgba.3});
        }

        // Create check button callback
        let acp = Rc::clone(&$acp_in);
        check.connect_toggled(move|button|{
            acp.config.borrow_mut().$show_var = button.get_active();
            acp.mark_background_dirty();
            acp.update_all_gui();
        });

        // Create color button callback
        let acp = Rc::clone(&$acp_in);
        ColorButtonExt::connect_property_rgba_notify(&color, move|button|{
            let rgba = button.get_rgba();

            acp.config.borrow_mut().$color_var = (rgba.red, rgba.green, rgba.blue, rgba.alpha);
            acp.mark_background_dirty();
            acp.update_all_gui();
        });

        // Layout
        hbox.pack_end(&color, false, true, PADDING);
        hbox.pack_start(&check, false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

macro_rules! build_config_check{
    ($v_box:ident, $label:expr, $acp:ident, $show_var:ident) => {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
        let check = CheckButton::new_with_label($label);

        check.set_active($acp.config.borrow().$show_var);

        // Create check button callback
        let acp = $acp.clone();
        check.connect_toggled(move|button|{
            acp.config.borrow_mut().$show_var = button.get_active();
            acp.mark_background_dirty();
            acp.update_all_gui();
        });

        // Layout
        hbox.pack_start(&check, false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

mod data_options;
mod background_options;

const PADDING: u32 = 2;
const BOX_SPACING: i32 = 5;

pub fn set_up_control_area(control_area: &Notebook, acp: &AppContextPointer) {
    control_area.set_hexpand(true);
    control_area.set_vexpand(true);
    control_area.set_scrollable(true);

    let data_options = data_options::make_data_option_frame(acp);
    control_area.add(&data_options);
    control_area.set_tab_label_text(&data_options, "Data");

    let background_options = background_options::make_background_frame(acp);
    control_area.add(&background_options);
    control_area.set_tab_label_text(&background_options, "Background");
}
