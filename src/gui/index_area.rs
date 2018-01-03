use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::Context;
use cairo::enums::{FontSlant, FontWeight};

pub fn set_up_index_area(index_area: &DrawingArea) {
    index_area.set_hexpand(true);
    index_area.set_vexpand(true);

    index_area.connect_draw(draw_index);
}

fn draw_index(index_area: &DrawingArea, cr: &Context) -> Inhibit {
    // Draw some text to mark this area.
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(40.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Index Data Goes Here");

    // Draw a blue border
    let alloc = index_area.get_allocation();
    cr.rectangle(0.0, 0.0, f64::from(alloc.width), f64::from(alloc.height));
    cr.set_source_rgb(0.0, 0.0, 1.0);
    cr.set_line_width(9.0);
    cr.stroke();

    Inhibit(false)
}
