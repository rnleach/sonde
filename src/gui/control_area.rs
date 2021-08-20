#![macro_use]

use crate::{app::AppContextPointer, errors::SondeError};
use gtk::{prelude::*, Notebook};

macro_rules! build_config_color_and_check {
    ($v_box:ident, $label:expr, $acp_in:expr, $show_var:ident, $color_var:ident) => {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
        let check = gtk::CheckButton::with_label($label);
        let color = gtk::ColorButton::new();
        color.set_use_alpha(true);

        // Inner scope to borrow acp
        {
            let config = $acp_in.config.borrow();
            check.set_active(config.$show_var);

            let rgba = config.$color_var;
            color.set_rgba(&RGBA {
                red: rgba.0,
                green: rgba.1,
                blue: rgba.2,
                alpha: rgba.3,
            });
        }

        // Create check button callback
        let acp = Rc::clone(&$acp_in);
        check.connect_toggled(move |button| {
            acp.config.borrow_mut().$show_var = button.is_active();
            acp.mark_background_dirty();
            crate::gui::draw_all(&acp);
            crate::gui::text_area::update_text_highlight(&acp);
        });

        // Create color button callback
        let acp = Rc::clone(&$acp_in);
        gtk::ColorButton::connect_property_notify_event(&color, move |button, _event| {
            let rgba = button.rgba();

            acp.config.borrow_mut().$color_var = (rgba.red, rgba.green, rgba.blue, rgba.alpha);
            acp.mark_background_dirty();
            crate::gui::draw_all(&acp);
            crate::gui::text_area::update_text_highlight(&acp);

            Inhibit(false)
        });

        // Layout
        hbox.pack_end(&color, false, true, PADDING);
        hbox.pack_start(&check, false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

macro_rules! build_config_check {
    ($v_box:ident, $label:expr, $acp:ident, $show_var:ident) => {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
        let check = gtk::CheckButton::with_label($label);

        check.set_active($acp.config.borrow().$show_var);

        // Create check button callback
        let acp = $acp.clone();
        check.connect_toggled(move |button| {
            acp.config.borrow_mut().$show_var = button.is_active();
            acp.mark_background_dirty();
            crate::gui::draw_all(&acp);
            crate::gui::text_area::update_text_highlight(&acp);
        });

        // Layout
        hbox.pack_start(&check, false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

macro_rules! build_config_color {
    ($v_box:ident, $label:expr, $acp_in:expr, $color_var:ident) => {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
        let color = gtk::ColorButton::new();
        color.set_use_alpha(true);

        // Inner scope to borrow acp
        {
            let config = $acp_in.config.borrow();
            let rgba = config.$color_var;
            color.set_rgba(&RGBA {
                red: rgba.0,
                green: rgba.1,
                blue: rgba.2,
                alpha: rgba.3,
            });
        }

        // Create color button callback
        let acp = Rc::clone(&$acp_in);
        WidgetExt::connect_property_notify_event(&color, move |button, _event| {
            let rgba = button.rgba();

            acp.config.borrow_mut().$color_var = (rgba.red, rgba.green, rgba.blue, rgba.alpha);
            acp.mark_background_dirty();
            acp.mark_data_dirty();
            acp.mark_overlay_dirty();

            crate::gui::draw_all(&acp);
            crate::gui::text_area::update_text_highlight(&acp);
            crate::gui::indexes_area::update_indexes_area(&acp);

            Inhibit(false)
        });

        // Layout
        hbox.pack_end(&color, false, true, PADDING);
        hbox.pack_start(&gtk::Label::new(Some($label)), false, true, PADDING);
        $v_box.pack_start(&hbox, false, true, PADDING);
    };
}

mod active_readout;
mod background_options;
mod data_options;
mod overlay_options;

const PADDING: u32 = 2;
const BOX_SPACING: i32 = 5;

pub fn set_up_control_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    let control_area: Notebook = acp.fetch_widget("control_area")?;
    control_area.set_hexpand(true);
    control_area.set_vexpand(true);
    control_area.set_scrollable(true);

    let data_options = data_options::make_data_option_frame(acp);
    control_area.add(&data_options);
    control_area.set_tab_label_text(&data_options, "Data");

    let background_options = background_options::make_background_frame(acp);
    control_area.add(&background_options);
    control_area.set_tab_label_text(&background_options, "Background");

    let active_readout_options = active_readout::make_active_readout_frame(acp);
    control_area.add(&active_readout_options);
    control_area.set_tab_label_text(&active_readout_options, "Active Readout");

    let overlay_options = overlay_options::make_overlay_frame(acp);
    control_area.add(&overlay_options);
    control_area.set_tab_label_text(&overlay_options, "Overlays");

    Ok(())
}
