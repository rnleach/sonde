use std::rc::Rc;

use gdk::EventMask;
use gtk::prelude::*;
use gtk::DrawingArea;

use app::AppContextPointer;

pub mod hodo_context;

mod callbacks;

pub fn set_up_hodograph_area(hodo_area: &DrawingArea, app_context: &AppContextPointer) {

    hodo_area.set_hexpand(true);
    hodo_area.set_vexpand(true);

    let ac = Rc::clone(app_context);
    hodo_area.connect_draw(move |_da, cr| callbacks::draw_hodo(cr, &ac));

    let ac = Rc::clone(app_context);
    hodo_area.connect_scroll_event(move |da, ev| callbacks::scroll_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    hodo_area.connect_button_press_event(move |da, ev| callbacks::button_press_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    hodo_area.connect_button_release_event(move |da, ev| {
        callbacks::button_release_event(da, ev, &ac)
    });

    let ac = Rc::clone(app_context);
    hodo_area.connect_motion_notify_event(move |da, ev| callbacks::mouse_motion_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    hodo_area.connect_leave_notify_event(move |da, ev| callbacks::leave_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    hodo_area.connect_key_release_event(move |da, ev| callbacks::key_release_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    hodo_area.connect_key_press_event(move |da, ev| callbacks::key_press_event(da, ev, &ac));

    hodo_area.set_can_focus(true);

    hodo_area.add_events(
        (EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK |
             EventMask::POINTER_MOTION_HINT_MASK |
             EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK |
             EventMask::KEY_RELEASE_MASK | EventMask::KEY_PRESS_MASK)
            .bits() as i32,
    );
}
