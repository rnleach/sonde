//! Module for the GUI components of the application.

use cairo::{Context, FontExtents, FontFace, FontSlant, FontWeight, Matrix, Operator};
use gdk::{keyval_from_name, EventButton, EventConfigure, EventKey, EventMotion, EventScroll,
          ScrollDirection};
use gtk::DrawingArea;
use gtk::prelude::*;

use sounding_analysis::Layer;
use sounding_analysis::layers::{warm_temperature_layer_aloft, warm_wet_bulb_layer_aloft};
use sounding_base::DataRow;

use app::config::Rgba;
use app::{AppContext, AppContextPointer};
use coords::{convert_pressure_to_y, DeviceCoords, DeviceRect, Rect, ScreenCoords, ScreenRect,
             XYCoords};
use errors::SondeError;

mod control_area;
mod hodograph;
mod main_window;
mod plot_context;
pub mod profiles;
mod sounding;
mod text_area;
mod utility;

pub use self::hodograph::HodoContext;
pub use self::plot_context::{PlotContext, PlotContextExt};
pub use self::sounding::SkewTContext;
pub use self::text_area::update_text_highlight;

use self::utility::DrawingArgs;

pub fn initialize(app: &AppContextPointer) -> Result<(), SondeError> {
    sounding::SkewTContext::set_up_drawing_area(&app)?;
    hodograph::HodoContext::set_up_drawing_area(&app)?;
    control_area::set_up_control_area(&app)?;
    text_area::set_up_text_area(&app)?;
    profiles::initialize_profiles(&app)?;
    main_window::set_up_main_window(&app)?;

    Ok(())
}

pub fn draw_all(app: &AppContext) {
    const DRAWING_AREAS: [&str; 2] = ["skew_t", "hodograph_area"];

    for &da in &DRAWING_AREAS {
        if let Ok(da) = app.fetch_widget::<DrawingArea>(da) {
            da.queue_draw();
        }
    }

    profiles::draw_profiles(&app);
}

pub fn update_text_view(app: &AppContext) {
    self::text_area::update_text_area(app);
    self::text_area::update_text_highlight(app);
}

trait Drawable: PlotContext + PlotContextExt {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    /// Required to implement.
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError>;

    /// Not recommended to override.
    fn init_matrix(&self, args: DrawingArgs) {
        let cr = args.cr;

        cr.save();

        let (x1, y1, x2, y2) = cr.clip_extents();
        let width = f64::abs(x2 - x1);
        let height = f64::abs(y2 - y1);

        let device_rect = DeviceRect {
            upper_left: DeviceCoords { row: 0.0, col: 0.0 },
            width,
            height,
        };
        self.set_device_rect(device_rect);
        let scale_factor = self.scale_factor();

        // Start fresh
        cr.identity_matrix();
        // Set the scale factor
        cr.scale(scale_factor, scale_factor);
        // Set origin at lower left.
        cr.transform(Matrix {
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: -1.0,
            x0: 0.0,
            y0: device_rect.height / scale_factor,
        });

        self.set_matrix(cr.get_matrix());
        cr.restore();
    }

    /// Not recommended to override.
    fn prepare_to_make_text(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let font_face =
            FontFace::toy_create(&config.font_name, FontSlant::Normal, FontWeight::Bold);
        cr.set_font_face(font_face);

        self.set_font_size(config.label_font_size, cr);
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    /// Override for background fill.
    fn draw_background_fill(&self, _args: DrawingArgs) {}

    /// Override for background lines.
    fn draw_background_lines(&self, _args: DrawingArgs) {}

    /// Override for background labels.
    fn collect_labels(&self, _args: DrawingArgs) -> Vec<(String, ScreenRect)> {
        vec![]
    }

    /// Not recommended to override.
    fn draw_background_labels(&self, args: DrawingArgs) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        if config.show_labels {
            let labels = self.collect_labels(args);
            let padding = cr.device_to_user_distance(config.label_padding, 0.0).0;

            for (label, rect) in labels {
                let ScreenRect { lower_left, .. } = rect;

                let mut rgba = config.background_rgba;
                cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
                cr.rectangle(
                    lower_left.x - padding,
                    lower_left.y - padding,
                    rect.width() + 2.0 * padding,
                    rect.height() + 2.0 * padding,
                );
                cr.fill();

                // Setup label colors
                rgba = config.label_rgba;
                cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
                cr.move_to(lower_left.x, lower_left.y);
                cr.show_text(&label);
            }
        }
    }

    /// Not recommended to override.
    fn draw_background(&self, args: DrawingArgs) {
        let config = args.ac.config.borrow();

        self.draw_background_fill(args);
        self.draw_background_lines(args);

        if config.show_labels {
            self.prepare_to_make_text(args);
            self.draw_background_labels(args);
        }
    }

    /***********************************************************************************************
     * Data Drawing.
     **********************************************************************************************/
    /// Override to draw data
    fn draw_data(&self, args: DrawingArgs);

    fn draw_data_and_legend(&self, args: DrawingArgs) {
        self.draw_data(args);

        if args.ac.config.borrow().show_legend {
            self.prepare_to_make_text(args);
            self.draw_legend(args);
        }
    }

    /// Override for for a legend.
    fn build_legend_strings(_ac: &AppContext) -> Vec<(String, Rgba)> {
        vec![]
    }

    /// Not recommended to override.
    fn draw_legend(&self, args: DrawingArgs) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if !ac.plottable() {
            return;
        }

        let mut upper_left = self.convert_device_to_screen(self.get_device_rect().upper_left);

        let padding = cr.device_to_user_distance(config.edge_padding, 0.0).0;
        upper_left.x += padding;
        upper_left.y -= padding;

        // Make sure we stay on the x-y coords domain
        let ScreenCoords { x: xmin, y: ymax } =
            self.convert_xy_to_screen(XYCoords { x: 0.0, y: 1.0 });
        let edge_offset = upper_left.x;
        if ymax - edge_offset < upper_left.y {
            upper_left.y = ymax - edge_offset;
        }

        if xmin + edge_offset > upper_left.x {
            upper_left.x = xmin + edge_offset;
        }

        let font_extents = cr.font_extents();

        let legend_text = Self::build_legend_strings(ac);

        let (box_width, box_height) =
            Self::calculate_legend_box_size(args, &font_extents, &legend_text);

        let legend_rect = ScreenRect {
            lower_left: ScreenCoords {
                x: upper_left.x,
                y: upper_left.y - box_height,
            },
            upper_right: ScreenCoords {
                x: upper_left.x + box_width,
                y: upper_left.y,
            },
        };

        Self::draw_legend_rectangle(args, &legend_rect);

        Self::draw_legend_text(args, &upper_left, &font_extents, &legend_text);
    }

    /// Not recommended to override.
    fn calculate_legend_box_size(
        args: DrawingArgs,
        font_extents: &FontExtents,
        legend_text: &[(String, Rgba)],
    ) -> (f64, f64) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let mut box_width: f64 = 0.0;
        let mut box_height: f64 = 0.0;

        for &(ref line, _) in legend_text {
            let extents = cr.text_extents(line);
            if extents.width > box_width {
                box_width = extents.width;
            }
            box_height += font_extents.height;
        }

        // Add padding last
        let (padding_x, padding_y) =
            cr.device_to_user_distance(config.edge_padding, -config.edge_padding);
        let padding_x = f64::max(padding_x, font_extents.max_x_advance);

        // Add room for the last line's descent and padding
        box_height += f64::max(font_extents.descent, padding_y);
        box_height += padding_y;
        box_width += 2.0 * padding_x;

        (box_width, box_height)
    }

    /// Not recommended to override.
    fn draw_legend_rectangle(args: DrawingArgs, screen_rect: &ScreenRect) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let ScreenRect { lower_left, .. } = *screen_rect;

        cr.rectangle(
            lower_left.x,
            lower_left.y,
            screen_rect.width(),
            screen_rect.height(),
        );

        let rgb = config.label_rgba;
        cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
        cr.stroke_preserve();
        let rgba = config.background_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.fill();
    }

    /// Not recommended to override.
    fn draw_legend_text(
        args: DrawingArgs,
        upper_left: &ScreenCoords,
        font_extents: &FontExtents,
        legend_text: &[(String, Rgba)],
    ) {
        let (config, cr) = (args.ac.config.borrow(), args.cr);

        let (padding_x, padding_y) =
            cr.device_to_user_distance(config.edge_padding, -config.edge_padding);
        let padding_x = f64::max(padding_x, font_extents.max_x_advance);

        // Remember how many lines we have drawn so far for setting position of the next line.
        let mut line_num = 1;

        for &(ref line, rgb) in legend_text {
            cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);

            cr.move_to(
                upper_left.x + padding_x,
                upper_left.y - padding_y - font_extents.ascent
                    - f64::from(line_num - 1) * font_extents.height,
            );

            cr.show_text(line);
            line_num += 1;
        }
    }

    /// Not recommended to override.
    fn draw_no_data(&self, args: DrawingArgs) {
        const MESSAGE: &str = "No Data";

        let (cr, config) = (args.cr, args.ac.config.borrow());

        self.prepare_to_make_text(args);
        cr.save();

        let ScreenRect {
            lower_left: ScreenCoords { x: xmin, y: ymin },
            upper_right: ScreenCoords { x: xmax, y: ymax },
        } = self.bounding_box_in_screen_coords();

        // Scale the font to fill the view.
        let width = xmax - xmin;
        let text_width = cr.text_extents(MESSAGE).width;
        let ratio = 0.75 * width / text_width;
        self.set_font_size(config.label_font_size * ratio, cr);

        // Calculate the starting position
        let text_extents = cr.text_extents(MESSAGE);
        let height = ymax - ymin;
        let start_y = ymin + (height - text_extents.height) / 2.0;
        let start_x = xmin + (width - text_extents.width) / 2.0;

        // Make a rectangle behind it.
        let font_extents = cr.font_extents();
        let mut rgb = config.background_rgba;
        cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
        cr.rectangle(
            start_x - 0.05 * text_extents.width,
            start_y - font_extents.descent,
            1.1 * text_extents.width,
            font_extents.height,
        );
        cr.fill_preserve();
        rgb = config.label_rgba;
        cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
        cr.stroke();

        // Draw the text.
        cr.move_to(start_x, start_y);
        cr.show_text(MESSAGE);

        cr.restore();
    }

    /***********************************************************************************************
     * Active readout Drawing.
     **********************************************************************************************/
    /// Override to activate the active readout/sampling.
    fn create_active_readout_text(_vals: &DataRow, _ac: &AppContext) -> Vec<(String, Rgba)> {
        vec![]
    }

    /// Override to add overlays other than the active readout, or to create one without text
    /// or that doesn't use pressure as a coordinate, such as the hodograph.
    fn draw_active_readout(&self, args: DrawingArgs) {
        if args.ac.config.borrow().show_active_readout {
            self.draw_active_sample(args);
        }
    }

    /// Not recommended to override, unless you want to create an active readout that doesn't use
    /// pressure as a vertical coord or doesn't use text. Like the Hodograph.
    fn draw_active_sample(&self, args: DrawingArgs) {
        if !self.has_data() {
            return;
        }

        let (ac, cr) = (args.ac, args.cr);

        let vals = if let Some(vals) = ac.get_sample() {
            vals
        } else {
            return;
        };

        let sample_p = if let Some(sample_p) = vals.pressure {
            sample_p
        } else {
            return;
        };

        let lines = Self::create_active_readout_text(&vals, ac);

        if lines.is_empty() {
            return;
        }

        self.draw_sample_line(args, sample_p);

        self.prepare_to_make_text(args);

        let box_rect = self.calculate_active_readout_box(args, &lines, sample_p);

        Self::draw_sample_readout_text_box(&box_rect, cr, ac, &lines);
    }

    /// Not recommended to override.
    fn draw_sample_line(&self, args: DrawingArgs, sample_p: f64) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let bb = self.bounding_box_in_screen_coords();
        let (left, right) = (bb.lower_left.x, bb.upper_right.x);
        let y = convert_pressure_to_y(sample_p);

        let rgba = config.active_readout_line_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(
            cr.device_to_user_distance(config.active_readout_line_width, 0.0)
                .0,
        );
        let start = self.convert_xy_to_screen(XYCoords { x: left, y });
        let end = self.convert_xy_to_screen(XYCoords { x: right, y });
        cr.move_to(start.x, start.y);
        cr.line_to(end.x, end.y);
        cr.stroke();
    }

    /// Not recommended to override.
    fn calculate_active_readout_box(
        &self,
        args: DrawingArgs,
        strings: &[(String, Rgba)],
        sample_p: f64,
    ) -> ScreenRect {
        let cr = args.cr;
        let config = args.ac.config.borrow();

        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;

        let font_extents = cr.font_extents();

        let mut line = String::with_capacity(100);
        for &(ref val, _) in strings.iter() {
            line.push_str(val);

            if !val.ends_with('\n') {
                continue;
            } else {
                let line_extents = cr.text_extents(line.trim());
                if line_extents.width > width {
                    width = line_extents.width;
                }
                height += font_extents.height;

                line.clear();
            }
        }

        let (padding, _) = cr.device_to_user_distance(config.edge_padding, 0.0);

        width += 2.0 * padding;
        height += 2.0 * padding;

        let ScreenCoords { x: mut left, .. } =
            self.convert_device_to_screen(DeviceCoords { col: 5.0, row: 5.0 });
        let ScreenCoords { y: top, .. } = self.convert_xy_to_screen(XYCoords {
            x: 0.0,
            y: convert_pressure_to_y(sample_p),
        });
        let mut bottom = top - height;

        let ScreenCoords { x: xmin, y: ymin } =
            self.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
        let ScreenCoords { x: xmax, y: ymax } =
            self.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });

        // Prevent clipping
        if left < xmin {
            left = xmin;
        }
        if left > xmax - width {
            left = xmax - width;
        }
        if bottom < ymin {
            bottom = ymin;
        }
        if bottom > ymax - height {
            bottom = ymax - height;
        }

        // Keep it on the screen
        let ScreenRect {
            lower_left: ScreenCoords { x: xmin, y: ymin },
            upper_right: ScreenCoords { x: xmax, y: ymax },
        } = self.bounding_box_in_screen_coords();
        if left < xmin {
            left = xmin;
        }
        if left > xmax - width {
            left = xmax - width;
        }
        if bottom < ymin {
            bottom = ymin;
        }
        if bottom > ymax - height {
            bottom = ymax - height;
        }

        let lower_left = ScreenCoords { x: left, y: bottom };
        let upper_right = ScreenCoords {
            x: left + width,
            y: bottom + height,
        };

        ScreenRect {
            lower_left,
            upper_right,
        }
    }

    /// Not recommended to override.
    fn draw_sample_readout_text_box(
        rect: &ScreenRect,
        cr: &Context,
        ac: &AppContext,
        lines: &[(String, Rgba)],
    ) {
        let config = ac.config.borrow();

        let ScreenRect {
            lower_left: ScreenCoords { x: xmin, y: ymin },
            upper_right: ScreenCoords { x: xmax, y: ymax },
        } = *rect;

        let rgba = config.background_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.rectangle(xmin, ymin, xmax - xmin, ymax - ymin);
        cr.fill_preserve();
        let rgba = config.label_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
        cr.stroke();

        let (padding, _) = cr.device_to_user_distance(config.edge_padding, 0.0);

        let font_extents = cr.font_extents();

        let mut lines_drawn = 0.0;
        let mut start_x = xmin + padding;

        for &(ref val, rgba) in lines {
            let show_val = if val.ends_with('\n') {
                val.trim_right()
            } else {
                val
            };

            let text_extents = cr.text_extents(show_val);

            cr.move_to(
                start_x,
                ymax - padding - font_extents.ascent - font_extents.height * lines_drawn,
            );
            cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            cr.show_text(show_val);
            if val.ends_with('\n') {
                lines_drawn += 1.0;
                start_x = xmin + padding;
            } else {
                start_x += text_extents.x_advance;
            }
        }
    }

    /***********************************************************************************************
     *                                     Drawing utilities
     **********************************************************************************************/
    fn set_font_size(&self, size_in_pct: f64, cr: &Context) {
        let height = self.get_device_rect().height();

        let mut font_size = size_in_pct / 100.0 * height;
        font_size = cr.device_to_user_distance(font_size, 0.0).0;

        // Flip the y-coordinate so it displays the font right side up
        cr.set_font_matrix(Matrix {
            xx: 1.0 * font_size,
            yx: 0.0,
            xy: 0.0,
            yy: -1.0 * font_size, // Reflect it to be right side up!
            x0: 0.0,
            y0: 0.0,
        });
    }

    fn draw_tag(&self, text: &str, location: ScreenCoords, color: Rgba, args: DrawingArgs) {
        self.prepare_to_make_text(args);

        let cr = args.cr;
        let config = args.ac.config.borrow();

        // Calculate the box
        let text_extents = cr.text_extents(text);
        let (padding, _) = cr.device_to_user_distance(config.edge_padding, 0.0);

        let width: f64 = text_extents.width + 2.0 * padding;
        let height: f64 = text_extents.height + 2.0 * padding;
        let leader = height * 2.0 / 3.0;
        let home_x = location.x + leader + padding;
        let home_y = location.y - text_extents.height / 2.0;

        // Draw the box
        cr.move_to(location.x, location.y);
        cr.rel_line_to(leader, height / 2.0);
        cr.rel_line_to(width, 0.0);
        cr.rel_line_to(0.0, -height);
        cr.rel_line_to(-width, 0.0);
        cr.rel_line_to(-leader, height / 2.0);
        let rgba = config.background_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.fill_preserve();
        let rgba = color;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
        cr.stroke();

        // Fill with text
        cr.move_to(home_x, home_y);
        cr.show_text(text);
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
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
        self.mark_background_dirty();

        ac.update_all_gui();

        Inhibit(true)
    }

    fn button_press_event(&self, event: &EventButton, _ac: &AppContextPointer) -> Inhibit {
        // Left mouse button
        if event.get_button() == 1 {
            self.set_last_cursor_position(Some(event.get_position().into()));
            self.set_left_button_pressed(true);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn button_release_event(&self, event: &EventButton) -> Inhibit {
        if event.get_button() == 1 {
            self.set_last_cursor_position(None);
            self.set_left_button_pressed(false);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn leave_event(&self, ac: &AppContextPointer) -> Inhibit {
        self.set_last_cursor_position(None);
        ac.set_sample(None);
        ac.update_all_gui();

        Inhibit(false)
    }

    fn mouse_motion_event(
        &self,
        da: &DrawingArea,
        ev: &EventMotion,
        ac: &AppContextPointer,
    ) -> Inhibit {
        da.grab_focus();

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);
                let new_position = DeviceCoords::from(ev.get_position());
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
                self.mark_background_dirty();
                ac.update_all_gui();
            }
        }
        Inhibit(false)
    }

    fn key_press_event(event: &EventKey, ac: &AppContextPointer) -> Inhibit {
        let keyval = event.get_keyval();
        if keyval == keyval_from_name("Right") || keyval == keyval_from_name("KP_Right") {
            ac.display_next();
            Inhibit(true)
        } else if keyval == keyval_from_name("Left") || keyval == keyval_from_name("KP_Left") {
            ac.display_previous();
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn size_allocate_event(&self, da: &DrawingArea) {
        self.update_cache_allocations(da);
    }

    fn configure_event(&self, event: &EventConfigure) -> bool {
        let rect = self.get_device_rect();
        let (width, height) = event.get_size();
        if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON
            || (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
        {
            self.mark_background_dirty();
        }
        false
    }

    /***********************************************************************************************
     * Used a layered cached system for drawing on screen
     **********************************************************************************************/
    fn draw_background_cached(&self, args: DrawingArgs) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if self.is_background_dirty() {
            let tmp_cr = Context::new(&self.get_background_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            let rgba = config.background_rgba;
            tmp_cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.bound_view();

            self.draw_background(tmp_args);

            self.clear_background_dirty();
        }

        cr.set_source_surface(&self.get_background_layer(), 0.0, 0.0);
        cr.paint();
    }

    fn draw_data_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_data_dirty() {
            let tmp_cr = Context::new(&self.get_data_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.draw_data_and_legend(tmp_args);

            self.clear_data_dirty();
        }

        cr.set_source_surface(&self.get_data_layer(), 0.0, 0.0);
        cr.paint();
    }

    fn draw_active_readout_cached(&self, args: DrawingArgs) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_overlay_dirty() {
            let tmp_cr = Context::new(&self.get_overlay_layer());

            // Clear the previous drawing from the cache
            tmp_cr.save();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint();
            tmp_cr.restore();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.draw_active_readout(tmp_args);

            self.clear_overlay_dirty();
        }

        cr.set_source_surface(&self.get_overlay_layer(), 0.0, 0.0);
        cr.paint();
    }
}

trait MasterDrawable: Drawable {
    fn draw_callback(&self, cr: &Context, acp: &AppContextPointer) -> Inhibit {
        let args = DrawingArgs::new(acp, cr);

        self.init_matrix(args);
        self.draw_background_cached(args);
        self.draw_data_cached(args);
        self.draw_active_readout_cached(args);

        Inhibit(false)
    }
}

trait SlaveProfileDrawable: Drawable {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64;
    fn set_translate_y(&self, new_translate: XYCoords);

    fn draw_callback(&self, cr: &Context, acp: &AppContextPointer) -> Inhibit {
        let args = DrawingArgs::new(acp, cr);

        let device_height = self.get_device_rect().height;
        let device_width = self.get_device_rect().width;
        let aspect_ratio = device_height / device_width;

        self.set_zoom_factor(aspect_ratio * self.get_master_zoom(acp));
        self.set_translate_y(acp.skew_t.get_translate());
        self.bound_view();

        self.init_matrix(args);
        self.draw_background_cached(args);
        self.draw_data_cached(args);
        self.draw_active_readout_cached(args);

        Inhibit(false)
    }

    fn draw_dendtritic_snow_growth_zone(&self, args: DrawingArgs) {
        let ac = args.ac;

        if !ac.config.borrow().show_dendritic_zone {
            return;
        }

        if let Some(ref snd) = ac.get_sounding_for_display() {
            let layers = match ::sounding_analysis::layers::dendritic_snow_zone(snd.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().dendritic_zone_rgba;

            self.draw_layers(args, &layers, rgba);
        }
    }

    fn draw_hail_growth_zone(&self, args: DrawingArgs) {
        let ac = args.ac;

        if !ac.config.borrow().show_hail_zone {
            return;
        }

        if let Some(ref snd) = ac.get_sounding_for_display() {
            let layers = match ::sounding_analysis::layers::hail_growth_zone(snd.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().hail_zone_rgba;

            self.draw_layers(args, &layers, rgba);
        }
    }

    fn draw_warm_layer_aloft(&self, args: DrawingArgs) {
        let ac = args.ac;

        if !ac.config.borrow().show_warm_layer_aloft {
            return;
        }

        if let Some(snd) = ac.get_sounding_for_display() {
            let layers = match warm_temperature_layer_aloft(snd.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().warm_layer_rgba;

            self.draw_layers(args, &layers, rgba);

            let layers = match warm_wet_bulb_layer_aloft(snd.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().warm_wet_bulb_aloft_rgba;

            self.draw_layers(args, &layers, rgba);
        }
    }

    fn draw_layers(&self, args: DrawingArgs, layers: &[Layer], color_rgba: (f64, f64, f64, f64)) {
        let cr = args.cr;

        let bb = self.bounding_box_in_screen_coords();
        let (left, right) = (bb.lower_left.x, bb.upper_right.x);

        cr.set_source_rgba(color_rgba.0, color_rgba.1, color_rgba.2, color_rgba.3);

        for layer in layers {
            let bottom_press = if let Some(press) = layer.bottom.pressure {
                press
            } else {
                continue;
            };

            let top_press = if let Some(press) = layer.top.pressure {
                press
            } else {
                continue;
            };

            let mut coords = [
                (left, bottom_press),
                (left, top_press),
                (right, top_press),
                (right, bottom_press),
            ];

            // Convert points to screen coords
            for coord in &mut coords {
                coord.1 = convert_pressure_to_y(coord.1);

                let screen_coords = self.convert_xy_to_screen(XYCoords {
                    x: coord.0,
                    y: coord.1,
                });

                coord.0 = screen_coords.x;
                coord.1 = screen_coords.y;
            }

            let mut coord_iter = coords.iter();
            for coord in coord_iter.by_ref().take(1) {
                cr.move_to(coord.0, coord.1);
            }
            for coord in coord_iter {
                cr.line_to(coord.0, coord.1);
            }

            cr.close_path();
            cr.fill();
        }
    }
}
