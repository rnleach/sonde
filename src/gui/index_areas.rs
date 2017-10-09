use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::Context;
use cairo::enums::{FontSlant, FontWeight};

// TODO: Data type to hold the index area

pub fn set_up_index_areas(index_area1: &DrawingArea, index_area2: &DrawingArea) {

    // configure index area 1
    index_area1.set_hexpand(true);
    index_area1.set_vexpand(true);

    index_area1.connect_draw(draw_index);

    // configure index area 2
    index_area2.set_hexpand(true);
    index_area2.set_vexpand(true);

    index_area2.connect_draw(draw_index);
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
    cr.rectangle(0.0, 0.0, alloc.width as f64, alloc.height as f64);
    cr.set_source_rgb(0.0, 0.0, 1.0);
    cr.set_line_width(9.0);
    cr.stroke();

    Inhibit(false)
}
