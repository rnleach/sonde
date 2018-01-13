use std::rc::Rc;

use gdk::EventMask;
use gtk::prelude::*;
use gtk::DrawingArea;

use gui::DrawingArgs;
use gui::plot_context::{Drawable, GenericContext, HasGenericContext, MasterDrawable,
                        PlotContextExt};

use app::{config, AppContextPointer};
use coords::{SDCoords, ScreenCoords, XYCoords};

mod drawing;

pub struct HodoContext {
    generic: GenericContext,
}

impl HodoContext {
    pub fn new() -> Self {
        HodoContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_sd_to_xy(coords: SDCoords) -> XYCoords {
        let radius = coords.speed / 2.0 / config::MAX_SPEED;
        let angle = (270.0 - coords.dir).to_radians();

        let x = radius * angle.cos() + 0.5;
        let y = radius * angle.sin() + 0.5;
        XYCoords { x, y }
    }

    pub fn convert_sd_to_screen(&self, coords: SDCoords) -> ScreenCoords {
        let xy = HodoContext::convert_sd_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }
}

impl HasGenericContext for HodoContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for HodoContext {}

impl Drawable for HodoContext {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.hodo.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.hodo.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.hodo.button_press_event(ev));

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| ac.hodo.button_release_event(ev));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.hodo.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.hodo.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| HodoContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.hodo.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.hodo.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    fn draw_background(&self, args: DrawingArgs) {
        drawing::draw_background(args);
    }

    fn draw_data(&self, args: DrawingArgs) {
        drawing::draw_data(args);
    }

    fn draw_overlays(&self, args: DrawingArgs) {
        drawing::draw_overlays(args);
    }
}

impl MasterDrawable for HodoContext {}
