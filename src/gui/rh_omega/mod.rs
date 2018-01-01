use std::rc::Rc;

use gdk::EventMask;
use gtk::prelude::*;
use gtk::DrawingArea;

use app::AppContextPointer;

pub mod rh_omega_context;

mod drawing;
mod callbacks;

pub fn set_up_rh_omega_area(da: &DrawingArea, app_context: &AppContextPointer) {
    da.set_hexpand(true);
    da.set_vexpand(true);

    let ac = Rc::clone(app_context);
    da.connect_draw(move |_da, cr| callbacks::draw_rh_omega(cr, &ac));

    let ac = Rc::clone(app_context);
    da.connect_scroll_event(move |da, ev| callbacks::scroll_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_button_press_event(move |da, ev| callbacks::button_press_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_button_release_event(move |da, ev| callbacks::button_release_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_motion_notify_event(move |da, ev| callbacks::mouse_motion_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_leave_notify_event(move |da, ev| callbacks::leave_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_key_release_event(move |da, ev| callbacks::key_release_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_key_press_event(move |da, ev| callbacks::key_press_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_configure_event(move |da, ev| callbacks::configure_event(da, ev, &ac));

    let ac = Rc::clone(app_context);
    da.connect_size_allocate(move |da, ev| callbacks::size_allocate_event(da, ev, &ac));

    da.set_can_focus(true);

    da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
        | EventMask::BUTTON_RELEASE_MASK | EventMask::POINTER_MOTION_HINT_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
        | EventMask::KEY_RELEASE_MASK | EventMask::KEY_PRESS_MASK)
        .bits() as i32);
}
