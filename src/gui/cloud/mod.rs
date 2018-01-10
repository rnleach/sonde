use std::rc::Rc;

use gdk::{EventMask, EventMotion, EventScroll, ScrollDirection};
use gtk::prelude::*;
use gtk::DrawingArea;

use app::{config, AppContextPointer};
use coords::{DeviceCoords, ScreenCoords, PPCoords, XYCoords};
use gui::DrawingArgs;
use gui::plot_context::{Drawable, GenericContext, HasGenericContext, PlotContext, PlotContextExt};

mod drawing;

pub struct CloudContext {
    generic: GenericContext,
}

impl CloudContext {
    pub fn new() -> Self {
        CloudContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_pp_to_xy(coords: PPCoords) -> XYCoords {
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.press))
            / (f64::log10(config::MAXP) - f64::log10(config::MINP));

        // The + sign below looks weird, but is correct.
        let x = coords.pcnt;

        XYCoords { x, y }
    }

    pub fn convert_xy_to_pp(coords: XYCoords) -> PPCoords {
        use std::f64;

        let press = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP))
                + f64::log10(config::MAXP),
        );
        let pcnt = coords.x;

        PPCoords { pcnt, press }
    }

    pub fn convert_screen_to_pp(&self, coords: ScreenCoords) -> PPCoords {
        let xy = self.convert_screen_to_xy(coords);
        CloudContext::convert_xy_to_pp(xy)
    }

    pub fn convert_pp_to_screen(&self, coords: PPCoords) -> ScreenCoords {
        let xy = CloudContext::convert_pp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_pp(&self, coords: DeviceCoords) -> PPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_pp(xy)
    }

    pub fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.set_translate(translate);
    }
}

impl HasGenericContext for CloudContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for CloudContext {
    fn zoom_to_envelope(&self) {
        let xy_envelope = &self.get_xy_envelope();
        self.set_translate(xy_envelope.lower_left);

        let height = xy_envelope.upper_right.y - xy_envelope.lower_left.y;

        let device_height = self.get_device_rect().height;
        let device_width = self.get_device_rect().width;
        let aspect_ratio = device_height / device_width;
        let height = height / aspect_ratio;
        let height_scale = 1.0 / height;

        self.set_zoom_factor(height_scale);
        self.bound_view();
    }

    fn bound_view(&self) {
        let device_rect = self.get_device_rect();

        let bounds = DeviceCoords {
            col: device_rect.width,
            row: device_rect.height,
        };
        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy(device_rect.upper_left);
        let height = upper_left.y - lower_right.y;

        let mut translate = self.get_translate();
        if height < 1.0 {
            if translate.y < 0.0 {
                translate.y = 0.0;
            }
            let max_y = 1.0 - height;
            if translate.y > max_y {
                translate.y = max_y;
            }
        } else {
            translate.y = -(height - 1.0) / 2.0;
        }
        self.set_translate(translate);
    }

    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {
        // Apply translation first
        let x = coords.x;
        let y = coords.y - self.get_translate().y;

        // Apply scaling
        let x = x;
        let y = y * self.get_zoom_factor();

        ScreenCoords { x, y }
    }

    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x;
        let y = coords.y / self.get_zoom_factor();

        // Unapply translation
        let x = x;
        let y = y + self.get_translate().y;

        XYCoords { x, y }
    }
}

impl Drawable for CloudContext {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.cloud.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.cloud.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.cloud.button_press_event(ev));

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| ac.cloud.button_release_event(ev));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.cloud.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.cloud.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| CloudContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.cloud.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.cloud.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    fn scroll_event(&self, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
        const DELTA_SCALE: f64 = 1.05;
        const MIN_ZOOM: f64 = 1.0;
        const MAX_ZOOM: f64 = 10.0;

        let pos = ac.rh_omega
            .convert_device_to_xy(DeviceCoords::from(event.get_position()));
        let dir = event.get_direction();

        let old_zoom = ac.cloud.get_zoom_factor();
        let mut new_zoom = old_zoom;

        match dir {
            ScrollDirection::Up => {
                new_zoom *= DELTA_SCALE;
            }
            ScrollDirection::Down => {
                new_zoom /= DELTA_SCALE;
            }
            _ => {}
        }

        if new_zoom < MIN_ZOOM {
            new_zoom = MIN_ZOOM;
        } else if new_zoom > MAX_ZOOM {
            new_zoom = MAX_ZOOM;
        }
        ac.cloud.set_zoom_factor(new_zoom);

        let mut translate = ac.cloud.get_translate();
        translate.y = pos.y - old_zoom / new_zoom * (pos.y - translate.y);
        ac.cloud.set_translate(translate);
        ac.cloud.bound_view();
        ac.cloud.mark_background_dirty();

        ac.update_all_gui();

        Inhibit(true)
    }

    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        event: &EventMotion,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);
                let new_position = DeviceCoords::from(event.get_position());
                self.set_last_cursor_position(Some(new_position));

                let new_position = self.convert_device_to_xy(new_position);
                let delta = (
                    new_position.x - old_position.x,
                    new_position.y - old_position.y,
                );
                let mut translate = self.get_translate();
                translate.y -= delta.1;
                self.set_translate_y(translate);
                self.bound_view();
                self.mark_background_dirty();
                ac.update_all_gui();

                ac.set_sample(None);
            }
        } else if ac.plottable() {
            let position: DeviceCoords = event.get_position().into();

            self.set_last_cursor_position(Some(position));
            let pp_position = self.convert_device_to_pp(position);
            let sample = ::sounding_analysis::linear_interpolate(
                &ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
                pp_position.press,
            );
            ac.set_sample(Some(sample));
            ac.mark_overlay_dirty();
            ac.update_all_gui();
        }
        Inhibit(false)
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
