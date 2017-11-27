use std::rc::Rc;

use gtk::{DrawingArea, WidgetExt};

use app::AppContextPointer;

mod callbacks;

pub fn set_up_hodograph_area(hodo_area: &DrawingArea, app_context: &AppContextPointer) {

    hodo_area.set_hexpand(true);
    hodo_area.set_vexpand(true);

    let ac = Rc::clone(app_context);
    hodo_area.connect_draw(move |_da, cr| callbacks::draw_hodo(cr, &ac));
}
