use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::Context;
use cairo::enums::{FontSlant, FontWeight};

// TODO: Data type to hold the hodo area state

pub fn set_up_hodograph_area(hodo_area: &DrawingArea) {

    hodo_area.set_hexpand(true);
    hodo_area.set_vexpand(true);

    hodo_area.connect_draw(draw_hodo);
}

fn draw_hodo(hodo_area: &DrawingArea, cr: &Context) -> Inhibit {
    // Draw some text to mark this area.
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(40.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Hodograph Goes Here");

    // Draw a green border
    let alloc = hodo_area.get_allocation();
    cr.rectangle(0.0, 0.0, alloc.width as f64, alloc.height as f64);
    cr.set_source_rgb(0.0, 1.0, 0.0);
    cr.set_line_width(9.0);
    cr.stroke();

    Inhibit(false)
}
