use std::rc::Rc;

use gdk::{EventMask, EventMotion, EventScroll, ScrollDirection};
use gtk::prelude::*;
use gtk::DrawingArea;

use app::AppContextPointer;

use gui::DrawingArgs;
use gui::plot_context::{Drawable, GenericContext, HasGenericContext, MasterDrawable, PlotContext,
                        PlotContextExt};

use app::config;
use coords::{DeviceCoords, ScreenCoords, TPCoords, XYCoords};

mod drawing;

pub struct SkewTContext {
    generic: GenericContext,
}

impl SkewTContext {
    pub fn new() -> Self {
        SkewTContext {
            generic: GenericContext::new(),
        }
    }

    pub fn convert_tp_to_xy(coords: TPCoords) -> XYCoords {
        use std::f64;

        let y = (f64::log10(config::MAXP) - f64::log10(coords.pressure))
            / (f64::log10(config::MAXP) - f64::log10(config::MINP));
        let x = (coords.temperature - config::MINT) / (config::MAXT - config::MINT);

        // do the skew
        let x = x + y;
        XYCoords { x, y }
    }

    pub fn convert_xy_to_tp(coords: XYCoords) -> TPCoords {
        use app::config;
        use std::f64;

        // undo the skew
        let x = coords.x - coords.y;
        let y = coords.y;

        let t = x * (config::MAXT - config::MINT) + config::MINT;
        let p = 10.0f64.powf(
            f64::log10(config::MAXP) - y * (f64::log10(config::MAXP) - f64::log10(config::MINP)),
        );

        TPCoords {
            temperature: t,
            pressure: p,
        }
    }

    pub fn convert_tp_to_screen(&self, coords: TPCoords) -> ScreenCoords {
        let xy = Self::convert_tp_to_xy(coords);
        self.convert_xy_to_screen(xy)
    }

    pub fn convert_screen_to_tp(&self, coords: ScreenCoords) -> TPCoords {
        let xy = self.convert_screen_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }

    pub fn convert_device_to_tp(&self, coords: DeviceCoords) -> TPCoords {
        let xy = self.convert_device_to_xy(coords);
        Self::convert_xy_to_tp(xy)
    }
}

impl HasGenericContext for SkewTContext {
    fn get_generic_context(&self) -> &GenericContext {
        &self.generic
    }
}

impl PlotContextExt for SkewTContext {}

impl Drawable for SkewTContext {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer) {
        da.set_hexpand(true);
        da.set_vexpand(true);

        let ac = Rc::clone(acp);
        da.connect_draw(move |_da, cr| ac.skew_t.draw_callback(cr, &ac));

        let ac = Rc::clone(acp);
        da.connect_scroll_event(move |_da, ev| ac.skew_t.scroll_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_button_press_event(move |_da, ev| ac.skew_t.button_press_event(ev));

        let ac = Rc::clone(acp);
        da.connect_button_release_event(move |_da, ev| ac.skew_t.button_release_event(ev));

        let ac = Rc::clone(acp);
        da.connect_motion_notify_event(move |da, ev| ac.skew_t.mouse_motion_event(da, ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_leave_notify_event(move |_da, _ev| ac.skew_t.leave_event(&ac));

        let ac = Rc::clone(acp);
        da.connect_key_press_event(move |_da, ev| SkewTContext::key_press_event(ev, &ac));

        let ac = Rc::clone(acp);
        da.connect_configure_event(move |_da, ev| ac.skew_t.configure_event(ev));

        let ac = Rc::clone(acp);
        da.connect_size_allocate(move |da, _ev| ac.skew_t.size_allocate_event(da));

        da.set_can_focus(true);

        da.add_events((EventMask::SCROLL_MASK | EventMask::BUTTON_PRESS_MASK
            | EventMask::BUTTON_RELEASE_MASK
            | EventMask::POINTER_MOTION_HINT_MASK
            | EventMask::POINTER_MOTION_MASK | EventMask::LEAVE_NOTIFY_MASK
            | EventMask::KEY_PRESS_MASK)
            .bits() as i32);
    }

    /// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
    fn scroll_event(&self, event: &EventScroll, ac: &AppContextPointer) -> Inhibit {
        const DELTA_SCALE: f64 = 1.05;
        const MIN_ZOOM: f64 = 1.0;
        const MAX_ZOOM: f64 = 10.0;

        let pos = self.convert_device_to_xy(DeviceCoords::from(event.get_position()));
        let dir = event.get_direction();

        let old_zoom = self.get_zoom_factor();
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
        self.set_zoom_factor(new_zoom);

        let mut translate = self.get_translate();
        translate = XYCoords {
            x: pos.x - old_zoom / new_zoom * (pos.x - translate.x),
            y: pos.y - old_zoom / new_zoom * (pos.y - translate.y),
        };
        self.set_translate(translate);
        self.bound_view();
        ac.mark_background_dirty();

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
                translate.x -= delta.0;
                translate.y -= delta.1;
                self.set_translate(translate);
                self.bound_view();
                ac.mark_background_dirty();
                ac.update_all_gui();

                ac.set_sample(None);
            }
        } else if ac.plottable() {
            let position: DeviceCoords = event.get_position().into();

            self.set_last_cursor_position(Some(position));
            let tp_position = self.convert_device_to_tp(position);
            let sample = ::sounding_analysis::linear_interpolate(
                &ac.get_sounding_for_display().unwrap(), // ac.plottable() call ensures this won't panic
                tp_position.pressure,
            );
            ac.set_sample(Some(sample));
            ac.mark_overlay_dirty();
            ac.update_all_gui();
        }
        Inhibit(false)
    }

    fn draw_background(&self, args: DrawingArgs) {
        self::drawing::draw_background(args);
    }

    fn draw_data(&self, args: DrawingArgs) {
        self::drawing::draw_data(args);
    }

    fn draw_overlays(&self, args: DrawingArgs) {
        self::drawing::draw_overlays(args);
    }
}

impl MasterDrawable for SkewTContext {}
