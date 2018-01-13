use std::cell::Cell;
use std::rc::Rc;

use gdk::{EventMask, EventMotion};
use gtk::prelude::*;
use gtk::DrawingArea;

use app::{config, AppContextPointer};
use coords::{DeviceCoords, Rect, ScreenCoords, WPCoords, XYCoords};
use gui::DrawingArgs;
use gui::plot_context::{Drawable, GenericContext, HasGenericContext, PlotContext, PlotContextExt,
                        SlaveProfileDrawable};

mod drawing;

#[derive(Debug)]
pub struct RHOmegaContext {
    x_zoom: Cell<f64>,
    generic: GenericContext,
}

impl RHOmegaContext {
    pub fn new() -> Self {
        RHOmegaContext {
            x_zoom: Cell::new(1.0),
            generic: GenericContext::new(),
        }
    }

    pub fn convert_wp_to_xy(coords: WPCoords) -> XYCoords {
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.p))
            / (f64::log10(config::MAXP) - f64::log10(config::MINP));

        // The + sign below looks weird, but is correct.
        let x = (coords.w + config::MAX_ABS_W) / (2.0 * config::MAX_ABS_W);

        XYCoords { x, y }
    }

    pub fn convert_xy_to_wp(coords: XYCoords) -> WPCoords {
        use std::f64;

        let p = 10.0f64.powf(
            -coords.y * (f64::log10(config::MAXP) - f64::log10(config::MINP))
                + f64::log10(config::MAXP),
        );
        let w = coords.x * (2.0 * config::MAX_ABS_W) - config::MAX_ABS_W;

        WPCoords { w, p }
    }

    pub fn convert_screen_to_wp(&self, coords: ScreenCoords) -> WPCoords {
        let xy = self.convert_screen_to_xy(coords);
        RHOmegaContext::convert_xy_to_wp(xy)
    }

    pub fn convert_wp_to_screen(&self, coords: WPCoords) -> ScreenCoords {
        let xy = RHOmegaContext::convert_wp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_device_to_wp(&self, coords: DeviceCoords) -> WPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_wp(xy)
    }
}

impl HasGenericContext for RHOmegaContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for RHOmegaContext {
    fn zoom_to_envelope(&self) {
        let xy_envelope = &self.get_xy_envelope();
        self.set_translate(xy_envelope.lower_left);

        let width = xy_envelope.width();
        let width_scale = 1.0 / width;

        self.x_zoom.set(width_scale);
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
        let x = coords.x - self.get_translate().x;
        let y = coords.y - self.get_translate().y;

        // Apply scaling
        let x = x * self.x_zoom.get();
        let y = y * self.get_zoom_factor();

        ScreenCoords { x, y }
    }

    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Unapply scaling first
        let x = coords.x / self.x_zoom.get();
        let y = coords.y / self.get_zoom_factor();

        // Unapply translation
        let x = x + self.get_translate().x;
        let y = y + self.get_translate().y;

        XYCoords { x, y }
    }
}

impl Drawable for RHOmegaContext {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.rh_omega.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.rh_omega.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.rh_omega.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| RHOmegaContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.rh_omega.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.rh_omega.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        event: &EventMotion,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if ac.plottable() {
            let position: DeviceCoords = event.get_position().into();

            self.set_last_cursor_position(Some(position));
            let wp_position = self.convert_device_to_wp(position);
            let sample = ::sounding_analysis::linear_interpolate(
                &ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
                wp_position.p,
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

impl SlaveProfileDrawable for RHOmegaContext {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64 {
        acp.skew_t.get_zoom_factor()
    }

    fn set_translate_y(&self, new_translate: XYCoords) {
        let mut translate = self.get_translate();
        translate.y = new_translate.y;
        self.set_translate(translate);
    }
}
