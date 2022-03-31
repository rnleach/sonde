use crate::{
    app::AppContextPointer,
    gui::control_area::{BOX_SPACING, PADDING},
};
use gdk::RGBA;
use gtk::{self, prelude::*, Adjustment, ColorButton, Frame, ScrolledWindow};
use std::rc::Rc;

pub fn make_background_frame(acp: &AppContextPointer) -> ScrolledWindow {
    let f = Frame::new(None);
    f.set_shadow_type(gtk::ShadowType::None);
    f.set_hexpand(true);
    f.set_vexpand(true);

    // Layout vertically
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    v_box.set_baseline_position(gtk::BaselinePosition::Top);

    // First set is background lines
    let lines_frame = build_lines_frame(acp);

    // Second set is background fills
    let fills_frame = build_fills_frame(acp);

    // Third set is for font
    let font_frame = build_font_frame(acp);

    // Layout boxes in the frame
    f.add(&v_box);
    v_box.pack_start(&lines_frame, true, true, PADDING);
    v_box.pack_start(&fills_frame, true, true, PADDING);
    v_box.pack_start(&font_frame, true, true, PADDING);
    let sw = ScrolledWindow::new(Adjustment::NONE, Adjustment::NONE);
    sw.add(&f);

    sw
}

fn build_lines_frame(acp: &AppContextPointer) -> gtk::Frame {
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
    build_config_color_and_check!(
        lines_box,
        "Freezing Line",
        acp,
        show_freezing_line,
        freezing_line_color
    );
    build_config_color_and_check!(
        lines_box,
        "Wet Bulb Zero Line",
        acp,
        show_wet_bulb_zero_line,
        wet_bulb_zero_line_color
    );

    build_config_color_and_check!(
        lines_box,
        "Hodograph Lines",
        acp,
        show_iso_speed,
        iso_speed_rgba
    );

    lines_frame
}

fn build_fills_frame(acp: &AppContextPointer) -> gtk::Frame {
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

    add_background_color_button(&fills_box, acp);

    fills_frame
}

fn add_background_color_button(target_box: &gtk::Box, acp: &AppContextPointer) {
    // Background color
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_SPACING);
    let color = ColorButton::new();

    let rgba = acp.config.borrow().background_rgba;
    color.set_rgba(&RGBA::new(rgba.0, rgba.1, rgba.2, rgba.3));

    // Create color button callback
    let ac = Rc::clone(acp);
    WidgetExt::connect_property_notify_event(&color, move |button, _event| {
        let rgba = button.rgba();

        ac.config.borrow_mut().background_rgba =
            (rgba.red(), rgba.green(), rgba.blue(), rgba.alpha());
        crate::gui::draw_all(&ac);
        crate::gui::text_area::update_text_highlight(&ac);

        Inhibit(false)
    });

    // Layout
    hbox.pack_end(&color, false, true, PADDING);
    hbox.pack_start(&gtk::Label::new(Some("Background")), false, true, PADDING);

    target_box.pack_start(&hbox, false, true, PADDING);
}

fn build_font_frame(acp: &AppContextPointer) -> gtk::Frame {
    let font_frame = gtk::Frame::new(Some("Labels"));
    let font_box = gtk::Box::new(gtk::Orientation::Vertical, BOX_SPACING);
    font_frame.add(&font_box);

    build_config_color_and_check!(font_box, "Labels", acp, show_labels, label_rgba);
    build_config_check!(font_box, "Show Legend", acp, show_legend);

    font_frame
}
