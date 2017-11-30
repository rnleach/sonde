use std::rc::Rc;

use gdk::EventMask;
use gtk::prelude::*;
use gtk::DrawingArea;

use app::AppContextPointer;

mod callbacks;

pub fn set_up_hodograph_area(hodo_area: &DrawingArea, app_context: &AppContextPointer) {

    hodo_area.set_hexpand(true);
    hodo_area.set_vexpand(true);

    let ac = Rc::clone(app_context);
    hodo_area.connect_draw(move |_da, cr| callbacks::draw_hodo(cr, &ac));

    hodo_area.add_events(
        (EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK |
             EventMask::POINTER_MOTION_HINT_MASK |
             EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK |
             EventMask::KEY_RELEASE_MASK | EventMask::KEY_PRESS_MASK)
            .bits() as i32,
    );
}
