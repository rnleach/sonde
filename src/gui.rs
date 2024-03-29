//! Module for the GUI components of the application.

use crate::{
    app::{config::Rgba, sample::Sample, AppContext, AppContextPointer},
    coords::{
        convert_pressure_to_y, DeviceCoords, DeviceRect, Rect, ScreenCoords, ScreenRect, XYCoords,
    },
    errors::SondeError,
};
use gtk::{
    cairo::{Context, FontExtents, FontFace, FontSlant, FontWeight, Matrix, Operator},
    glib::Propagation,
    prelude::*,
    DrawingArea, EventControllerMotion,
};
use metfor::{HectoPascal, Quantity};
use sounding_analysis::{
    self, freezing_levels, warm_temperature_layer_aloft, warm_wet_bulb_layer_aloft,
    wet_bulb_zero_levels, DataRow, Layer,
};

mod control_area;
mod fire_plume;
mod hodograph;
mod indexes_area;
mod main_window;
mod plot_context;
pub mod profiles;
mod provider_data;
mod sounding;
mod text_area;
mod utility;

pub use self::fire_plume::{FirePlumeContext, FirePlumeEnergyContext};
pub use self::hodograph::HodoContext;
pub use self::plot_context::{PlotContext, PlotContextExt};
pub use self::sounding::SkewTContext;
pub use self::text_area::update_text_highlight;

use self::utility::{plot_curve_from_points, DrawingArgs};

pub fn initialize(app: &AppContextPointer) -> Result<(), SondeError> {
    sounding::SkewTContext::set_up_drawing_area(app)?;
    hodograph::HodoContext::set_up_drawing_area(app)?;
    fire_plume::FirePlumeContext::set_up_drawing_area(app)?;
    fire_plume::FirePlumeEnergyContext::set_up_drawing_area(app)?;
    control_area::set_up_control_area(app)?;
    text_area::set_up_text_area(app)?;
    profiles::initialize_profiles(app)?;
    indexes_area::set_up_indexes_area(app)?;
    provider_data::set_up_provider_text_area(app)?;
    main_window::set_up_main_window(app)?;

    Ok(())
}

pub fn draw_all(app: &AppContext) {
    const DRAWING_AREAS: [&str; 4] = [
        "skew_t",
        "hodograph_area",
        "fire_plume_height_area",
        "fire_plume_energy_area",
    ];

    for &da in &DRAWING_AREAS {
        if let Ok(da) = app.fetch_widget::<DrawingArea>(da) {
            da.queue_draw();
        }
    }

    profiles::draw_profiles(app);
}

pub fn update_text_views(app: &AppContext) {
    self::text_area::update_text_area(app);
    self::text_area::update_text_highlight(app);
    self::indexes_area::update_indexes_area(app);
    self::provider_data::update_text_area(app);
}

trait Drawable: PlotContext + PlotContextExt {
    /***********************************************************************************************
     * Initialization
     **********************************************************************************************/
    /// Required to implement.
    fn set_up_drawing_area(acp: &AppContextPointer) -> Result<(), SondeError>;

    /// Not recommended to override.
    fn init_matrix(&self, args: DrawingArgs<'_, '_>) {
        let cr = args.cr;

        cr.save().unwrap();

        let (x1, y1, x2, y2) = cr.clip_extents().unwrap();
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
        cr.transform(Matrix::new(
            1.0,
            0.0,
            0.0,
            -1.0,
            0.0,
            device_rect.height / scale_factor,
        ));

        self.set_matrix(cr.matrix());
        cr.restore().unwrap();
    }

    /// Not recommended to override.
    fn prepare_to_make_text(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let font_face =
            &FontFace::toy_create(&config.font_name, FontSlant::Normal, FontWeight::Bold).unwrap();
        cr.set_font_face(font_face);

        self.set_font_size(config.label_font_size, cr);
    }

    /***********************************************************************************************
     * Background Drawing.
     **********************************************************************************************/
    /// Override for background fill.
    fn draw_background_fill(&self, _args: DrawingArgs<'_, '_>) {}

    /// Override for background lines.
    fn draw_background_lines(&self, _args: DrawingArgs<'_, '_>) {}

    /// Override for background labels.
    fn collect_labels(&self, _args: DrawingArgs<'_, '_>) -> Vec<(String, ScreenRect)> {
        vec![]
    }

    /// Not recommended to override.
    fn draw_background_labels(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        if config.show_labels {
            let labels = self.collect_labels(args);
            let padding = cr
                .device_to_user_distance(config.label_padding, 0.0)
                .unwrap()
                .0;

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
                cr.fill().unwrap();

                // Setup label colors
                rgba = config.label_rgba;
                cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
                cr.move_to(lower_left.x, lower_left.y);
                cr.show_text(&label).unwrap();
            }
        }
    }

    /// Not recommended to override.
    fn draw_background(&self, args: DrawingArgs<'_, '_>) {
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
    fn draw_data(&self, args: DrawingArgs<'_, '_>);

    /// Not recommended to override
    fn draw_data_and_legend(&self, args: DrawingArgs<'_, '_>) {
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
    fn draw_legend(&self, args: DrawingArgs<'_, '_>) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if !ac.plottable() {
            return;
        }

        let mut upper_left = self.convert_device_to_screen(self.get_device_rect().upper_left);

        let padding = cr
            .device_to_user_distance(config.edge_padding, 0.0)
            .unwrap()
            .0;
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

        let font_extents = cr.font_extents().unwrap();

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
        args: DrawingArgs<'_, '_>,
        font_extents: &FontExtents,
        legend_text: &[(String, Rgba)],
    ) -> (f64, f64) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let mut box_width: f64 = 0.0;
        let mut box_height: f64 = 0.0;

        for (line, _) in legend_text {
            let extents = cr.text_extents(line).unwrap();
            if extents.width() > box_width {
                box_width = extents.width();
            }
            box_height += font_extents.height();
        }

        // Add padding last
        let (padding_x, padding_y) = cr
            .device_to_user_distance(config.edge_padding, -config.edge_padding)
            .unwrap();
        let padding_x = f64::max(padding_x, font_extents.max_x_advance());

        // Add room for the last line's descent and padding
        box_height += f64::max(font_extents.descent(), padding_y);
        box_height += padding_y;
        box_width += 2.0 * padding_x;

        (box_width, box_height)
    }

    /// Not recommended to override.
    fn draw_legend_rectangle(args: DrawingArgs<'_, '_>, screen_rect: &ScreenRect) {
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
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).unwrap().0);
        cr.stroke_preserve().unwrap();
        let rgba = config.background_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.fill().unwrap();
    }

    /// Not recommended to override.
    fn draw_legend_text(
        args: DrawingArgs<'_, '_>,
        upper_left: &ScreenCoords,
        font_extents: &FontExtents,
        legend_text: &[(String, Rgba)],
    ) {
        let (config, cr) = (args.ac.config.borrow(), args.cr);

        let (padding_x, padding_y) = cr
            .device_to_user_distance(config.edge_padding, -config.edge_padding)
            .unwrap();
        let padding_x = f64::max(padding_x, font_extents.max_x_advance());

        // Remember how many lines we have drawn so far for setting position of the next line.
        let mut line_num = 1;

        for &(ref line, rgb) in legend_text {
            cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);

            cr.move_to(
                upper_left.x + padding_x,
                upper_left.y
                    - padding_y
                    - font_extents.ascent()
                    - f64::from(line_num - 1) * font_extents.height(),
            );

            cr.show_text(line).unwrap();
            line_num += 1;
        }
    }

    /// Not recommended to override.
    fn draw_no_data(&self, args: DrawingArgs<'_, '_>) {
        const MESSAGE: &str = "No Data";

        let (cr, config) = (args.cr, args.ac.config.borrow());

        self.prepare_to_make_text(args);
        cr.save().unwrap();

        let ScreenRect {
            lower_left: ScreenCoords { x: xmin, y: ymin },
            upper_right: ScreenCoords { x: xmax, y: ymax },
        } = self.get_plot_area();

        // Scale the font to fill the view.
        let width = xmax - xmin;
        let text_width = cr.text_extents(MESSAGE).unwrap().width();
        let ratio = 0.75 * width / text_width;
        self.set_font_size(config.label_font_size * ratio, cr);

        // Calculate the starting position
        let text_extents = cr.text_extents(MESSAGE).unwrap();
        let height = ymax - ymin;
        let start_y = ymin + (height - text_extents.height()) / 2.0;
        let start_x = xmin + (width - text_extents.width()) / 2.0;

        // Make a rectangle behind it.
        let font_extents = cr.font_extents().unwrap();
        let mut rgb = config.background_rgba;
        cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
        cr.rectangle(
            start_x - 0.05 * text_extents.width(),
            start_y - font_extents.descent(),
            1.1 * text_extents.width(),
            font_extents.height(),
        );
        cr.fill_preserve().unwrap();
        rgb = config.label_rgba;
        cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).unwrap().0);
        cr.stroke().unwrap();

        // Draw the text.
        cr.move_to(start_x, start_y);
        cr.show_text(MESSAGE).unwrap();

        cr.restore().unwrap();
    }

    /***********************************************************************************************
     * Active readout Drawing.
     **********************************************************************************************/
    /// Override to activate the active readout/sampling.
    fn create_active_readout_text(_vals: &Sample, _ac: &AppContext) -> Vec<(String, Rgba)> {
        vec![]
    }

    /// Override to add overlays other than the active readout, or to create one without text
    /// or that doesn't use pressure as a coordinate, such as the hodograph.
    fn draw_active_readout(&self, args: DrawingArgs<'_, '_>) {
        if args.ac.config.borrow().show_active_readout {
            self.draw_active_sample(args);
        }
    }

    /// Not recommended to override, unless you want to create an active readout that doesn't use
    /// pressure as a vertical coord or doesn't use text. Like the Hodograph.
    fn draw_active_sample(&self, args: DrawingArgs<'_, '_>) {
        if !self.has_data() {
            return;
        }

        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let vals = ac.get_sample();

        let sample_p = if let Some(sample_p) = match *vals {
            Sample::Sounding { data, .. } => data.pressure.into_option(),
            Sample::FirePlume { parcel_low, .. } => Some(parcel_low.pressure),
            Sample::None => None,
        } {
            sample_p
        } else {
            return;
        };

        if config.show_active_readout_line {
            self.draw_sample_line(args, sample_p);
        }

        if config.show_active_readout_text {
            let lines = Self::create_active_readout_text(&vals, ac);

            if lines.is_empty() {
                return;
            }

            self.prepare_to_make_text(args);

            let box_rect = self.calculate_active_readout_box(args, &lines, sample_p);

            Self::draw_sample_readout_text_box(&box_rect, cr, ac, &lines);
        }
    }

    /// Not recommended to override.
    fn draw_sample_line(&self, args: DrawingArgs<'_, '_>, sample_p: HectoPascal) {
        let (ac, cr) = (args.ac, args.cr);
        let config = ac.config.borrow();

        let bb = self.get_plot_area();
        let (left, right) = (bb.lower_left.x, bb.upper_right.x);
        let y = convert_pressure_to_y(sample_p);

        let rgba = config.active_readout_line_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(
            cr.device_to_user_distance(config.active_readout_line_width, 0.0)
                .unwrap()
                .0,
        );
        let start = self.convert_xy_to_screen(XYCoords { x: left, y });
        let end = self.convert_xy_to_screen(XYCoords { x: right, y });
        cr.move_to(start.x, start.y);
        cr.line_to(end.x, end.y);
        cr.stroke().unwrap();
    }

    /// Not recommended to override.
    fn calculate_active_readout_box(
        &self,
        args: DrawingArgs<'_, '_>,
        strings: &[(String, Rgba)],
        sample_p: HectoPascal,
    ) -> ScreenRect {
        let cr = args.cr;
        let config = args.ac.config.borrow();

        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;

        let font_extents = cr.font_extents().unwrap();

        let mut line = String::with_capacity(100);
        for (val, _) in strings.iter() {
            line.push_str(val);

            if !val.ends_with('\n') {
                continue;
            } else {
                let line_extents = cr.text_extents(line.trim()).unwrap();
                if line_extents.width() > width {
                    width = line_extents.width();
                }
                height += font_extents.height();

                line.clear();
            }
        }

        let (padding, _) = cr
            .device_to_user_distance(config.edge_padding, 0.0)
            .unwrap();

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
        } = self.get_plot_area();
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
        cr.fill_preserve().unwrap();
        let rgba = config.label_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).unwrap().0);
        cr.stroke().unwrap();

        let (padding, _) = cr
            .device_to_user_distance(config.edge_padding, 0.0)
            .unwrap();

        let font_extents = cr.font_extents().unwrap();

        let mut lines_drawn = 0.0;
        let mut start_x = xmin + padding;

        for &(ref val, rgba) in lines {
            let show_val = if val.ends_with('\n') {
                val.trim_end()
            } else {
                val
            };

            let text_extents = cr.text_extents(show_val).unwrap();

            cr.move_to(
                start_x,
                ymax - padding - font_extents.ascent() - font_extents.height() * lines_drawn,
            );
            cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            cr.show_text(show_val).unwrap();
            if val.ends_with('\n') {
                lines_drawn += 1.0;
                start_x = xmin + padding;
            } else {
                start_x += text_extents.x_advance();
            }
        }
    }

    /***********************************************************************************************
     *                                     Drawing utilities
     **********************************************************************************************/
    fn set_font_size(&self, size_in_pct: f64, cr: &Context) {
        let height = self.get_device_rect().height();

        let mut font_size = size_in_pct / 100.0 * height;
        font_size = cr.device_to_user_distance(font_size, 0.0).unwrap().0;

        // Flip the y-coordinate so it displays the font right side up
        cr.set_font_matrix(Matrix::new(
            1.0 * font_size,
            0.0,
            0.0,
            -1.0 * font_size, // Reflect it to be right side up!
            0.0,
            0.0,
        ));
    }

    fn draw_tag(
        &self,
        text: &str,
        mut location: ScreenCoords,
        color: Rgba,
        args: DrawingArgs<'_, '_>,
    ) {
        self.prepare_to_make_text(args);

        let cr = args.cr;
        let config = args.ac.config.borrow();

        // Calculate the box
        let text_extents = cr.text_extents(text).unwrap();
        let (padding, _) = cr
            .device_to_user_distance(config.edge_padding, 0.0)
            .unwrap();

        let width: f64 = text_extents.width() + 2.0 * padding;
        let height: f64 = text_extents.height() + 2.0 * padding;
        let leader = height * 2.0 / 3.0;
        let mut home_x = location.x + leader + padding;
        let mut home_y = location.y - text_extents.height() / 2.0;

        // Make adjustments to keep it on screen
        let overflow = location.x + width + leader - 1.0;
        if overflow > 0.0 {
            location.x -= overflow;
            home_x -= overflow;
        }

        let overflow = location.y - height / 2.0;
        if overflow < 0.0 {
            location.y -= overflow;
            home_y -= overflow;
        }

        // Draw the box
        cr.move_to(location.x, location.y);
        cr.rel_line_to(leader, height / 2.0);
        cr.rel_line_to(width, 0.0);
        cr.rel_line_to(0.0, -height);
        cr.rel_line_to(-width, 0.0);
        cr.rel_line_to(-leader, height / 2.0);
        let fg_rgba = color;
        let bg_rgba = config.background_rgba;
        cr.set_source_rgba(bg_rgba.0, bg_rgba.1, bg_rgba.2, fg_rgba.3);
        cr.fill_preserve().unwrap();
        cr.set_source_rgba(fg_rgba.0, fg_rgba.1, fg_rgba.2, fg_rgba.3);
        cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).unwrap().0);
        cr.stroke().unwrap();

        // Fill with text
        cr.move_to(home_x, home_y);
        cr.show_text(text).unwrap();
    }

    fn draw_point(location: ScreenCoords, color: Rgba, args: DrawingArgs<'_, '_>) {
        let cr = args.cr;

        let pnt_size = cr.device_to_user_distance(5.0, 0.0).unwrap().0;

        cr.set_source_rgba(color.0, color.1, color.2, color.3);
        cr.arc(
            location.x,
            location.y,
            pnt_size,
            0.0,
            2.0 * ::std::f64::consts::PI,
        );
        cr.fill().unwrap();
    }

    /***********************************************************************************************
     * Events
     **********************************************************************************************/
    /// Handles zooming from the mouse wheel. Connected to the scroll-event signal.
    fn scroll_event(&self, dy: f64, ac: &AppContextPointer) -> Propagation {
        const DELTA_SCALE: f64 = 1.05;

        if let Some(pos) = self.get_last_cursor_position() {
            let pos = self.convert_device_to_xy(pos);

            let old_zoom = self.get_zoom_factor();
            let mut new_zoom = old_zoom;

            if dy > 0.0 {
                new_zoom /= DELTA_SCALE;
            } else if dy < 0.0 {
                new_zoom *= DELTA_SCALE;
            }

            let mut translate = self.get_translate();
            translate = XYCoords {
                x: pos.x - old_zoom / new_zoom * (pos.x - translate.x),
                y: pos.y - old_zoom / new_zoom * (pos.y - translate.y),
            };

            self.zoom(translate, new_zoom);

            draw_all(ac);
            text_area::update_text_highlight(ac);

            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    }

    fn left_button_press_event(&self, position: (f64, f64), _ac: &AppContextPointer) {
        self.set_last_cursor_position(Some(position.into()));
        self.set_left_button_pressed(true);
    }

    fn right_button_release_event(&self, _position: (f64, f64), _ac: &AppContextPointer) {
        // For showing optional context menu
    }

    fn left_button_release_event(&self, _position: (f64, f64), _ac: &AppContextPointer) {
        self.set_last_cursor_position(None);
        self.set_left_button_pressed(false);
    }

    fn enter_event(&self, _ac: &AppContextPointer) {}

    fn leave_event(&self, ac: &AppContextPointer) {
        self.set_last_cursor_position(None);
        ac.set_sample(Sample::None);

        draw_all(ac);
        text_area::update_text_highlight(ac);
    }

    fn mouse_motion_event(
        &self,
        controller: &EventControllerMotion,
        new_position: (f64, f64),
        ac: &AppContextPointer,
    ) {
        let da: DrawingArea = controller.widget().downcast().unwrap();
        da.grab_focus();

        let position = DeviceCoords::from(new_position);

        if self.get_left_button_pressed() {
            if let Some(last_position) = self.get_last_cursor_position() {
                let old_position = self.convert_device_to_xy(last_position);

                let position = self.convert_device_to_xy(position);
                let delta = (position.x - old_position.x, position.y - old_position.y);
                let mut translate = self.get_translate();
                translate.x -= delta.0;
                translate.y -= delta.1;
                self.set_translate(translate);
                self.bound_view();
                self.mark_background_dirty();

                draw_all(ac);
                text_area::update_text_highlight(ac);
            }
        }

        self.set_last_cursor_position(Some(position));
    }

    fn key_press_event(keyval: gtk::gdk::Key, ac: &AppContextPointer) -> Propagation {
        use gtk::gdk::Key;

        if keyval == Key::KP_Right || keyval == Key::Right {
            ac.display_next();
            Propagation::Stop
        } else if keyval == Key::KP_Left || keyval == Key::Left {
            ac.display_previous();
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    }

    fn size_allocate_event(&self, da: &DrawingArea) {
        self.update_cache_allocations(da);
    }

    fn resize_event(&self, width: i32, height: i32, ac: &AppContextPointer) -> bool {
        let rect = self.get_device_rect();
        if (rect.width - f64::from(width)).abs() < ::std::f64::EPSILON
            || (rect.height - f64::from(height)).abs() < ::std::f64::EPSILON
        {
            ac.mark_background_dirty();
        }
        false
    }

    /***********************************************************************************************
     * Used a layered cached system for drawing on screen
     **********************************************************************************************/
    fn draw_background_cached(&self, args: DrawingArgs<'_, '_>) {
        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        if self.is_background_dirty() {
            let tmp_cr = Context::new(&self.get_background_layer()).unwrap();

            // Clear the previous drawing from the cache
            tmp_cr.save().unwrap();
            let rgba = config.background_rgba;
            tmp_cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint().unwrap();
            tmp_cr.restore().unwrap();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.bound_view();

            self.clip(&tmp_cr);

            self.draw_background(tmp_args);

            self.clear_background_dirty();
        }

        cr.set_source_surface(&self.get_background_layer(), 0.0, 0.0)
            .unwrap();
        cr.paint().unwrap();
    }

    fn draw_data_cached(&self, args: DrawingArgs<'_, '_>) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_data_dirty() {
            let tmp_cr = Context::new(&self.get_data_layer()).unwrap();

            // Clear the previous drawing from the cache
            tmp_cr.save().unwrap();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint().unwrap();
            tmp_cr.restore().unwrap();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.clip(&tmp_cr);

            self.draw_data_and_legend(tmp_args);

            self.clear_data_dirty();
        }

        cr.set_source_surface(&self.get_data_layer(), 0.0, 0.0)
            .unwrap();
        cr.paint().unwrap();
    }

    fn draw_active_readout_cached(&self, args: DrawingArgs<'_, '_>) {
        let (ac, cr) = (args.ac, args.cr);

        if self.is_overlay_dirty() {
            let tmp_cr = Context::new(&self.get_overlay_layer()).unwrap();

            // Clear the previous drawing from the cache
            tmp_cr.save().unwrap();
            tmp_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            tmp_cr.set_operator(Operator::Source);
            tmp_cr.paint().unwrap();
            tmp_cr.restore().unwrap();
            tmp_cr.transform(self.get_matrix());
            let tmp_args = DrawingArgs { cr: &tmp_cr, ac };

            self.clip(&tmp_cr);

            self.draw_active_readout(tmp_args);

            self.clear_overlay_dirty();
        }

        cr.set_source_surface(&self.get_overlay_layer(), 0.0, 0.0)
            .unwrap();
        cr.paint().unwrap();
    }

    fn clip(&self, cr: &Context) {
        let clip_area = self.get_plot_area();
        cr.rectangle(
            clip_area.min_x(),
            clip_area.min_y(),
            clip_area.width(),
            clip_area.height(),
        );
        cr.clip();
    }
}

trait MasterDrawable: Drawable {
    fn draw_callback(&self, cr: &Context, acp: &AppContextPointer) -> Propagation {
        let args = DrawingArgs::new(acp, cr);
        self.init_matrix(args);

        self.draw_background_cached(args);
        self.draw_data_cached(args);
        self.draw_active_readout_cached(args);

        Propagation::Proceed
    }
}

trait SlaveProfileDrawable: Drawable {
    fn get_master_zoom(&self, acp: &AppContextPointer) -> f64;
    fn set_translate_y(&self, new_translate: XYCoords);

    fn draw_callback(&self, cr: &Context, acp: &AppContextPointer) -> Propagation {
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

        Propagation::Proceed
    }

    fn draw_dendritic_snow_growth_zone(&self, args: DrawingArgs<'_, '_>) {
        let ac = args.ac;

        if !ac.config.borrow().show_dendritic_zone {
            return;
        }

        if let Some(anal) = ac.get_sounding_for_display() {
            let anal = anal.borrow();
            let layers = match sounding_analysis::dendritic_snow_zone(anal.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().dendritic_zone_rgba;

            self.draw_layers(args, &layers, rgba);
        }
    }

    fn draw_hail_growth_zone(&self, args: DrawingArgs<'_, '_>) {
        let ac = args.ac;

        if !ac.config.borrow().show_hail_zone {
            return;
        }

        if let Some(anal) = ac.get_sounding_for_display() {
            let anal = anal.borrow();
            let layers = match sounding_analysis::hail_growth_zone(anal.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().hail_zone_rgba;

            self.draw_layers(args, &layers, rgba);
        }
    }

    fn draw_warm_layer_aloft(&self, args: DrawingArgs<'_, '_>) {
        let ac = args.ac;

        if !ac.config.borrow().show_warm_layer_aloft {
            return;
        }

        if let Some(anal) = ac.get_sounding_for_display() {
            let anal = anal.borrow();
            let layers = match warm_temperature_layer_aloft(anal.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().warm_layer_rgba;

            self.draw_layers(args, &layers, rgba);

            let layers = match warm_wet_bulb_layer_aloft(anal.sounding()) {
                Ok(layers) => layers,
                Err(_) => return,
            };

            let rgba = ac.config.borrow().warm_wet_bulb_aloft_rgba;

            self.draw_layers(args, &layers, rgba);
        }
    }

    fn draw_freezing_levels(&self, args: DrawingArgs<'_, '_>) {
        let ac = args.ac;

        if !ac.config.borrow().show_freezing_line {
            return;
        }

        if let Some(anal) = ac.get_sounding_for_display() {
            let anal = anal.borrow();
            let levels = match freezing_levels(anal.sounding()) {
                Ok(levels) => levels,
                Err(_) => return,
            };

            let config = ac.config.borrow();
            let rgba = config.freezing_line_color;
            let line_width = config.freezing_line_width;

            self.draw_levels(args, &levels, rgba, line_width);
        }
    }

    fn draw_wet_bulb_zero_levels(&self, args: DrawingArgs<'_, '_>) {
        let ac = args.ac;

        if !ac.config.borrow().show_wet_bulb_zero_line {
            return;
        }

        if let Some(anal) = ac.get_sounding_for_display() {
            let anal = anal.borrow();
            let levels = match wet_bulb_zero_levels(anal.sounding()) {
                Ok(levels) => levels,
                Err(_) => return,
            };

            let config = ac.config.borrow();
            let rgba = config.wet_bulb_zero_line_color;
            let line_width = config.wet_bulb_zero_line_width;

            self.draw_levels(args, &levels, rgba, line_width);
        }
    }

    fn draw_layers(&self, args: DrawingArgs<'_, '_>, layers: &[Layer], color_rgba: Rgba) {
        let cr = args.cr;

        let bb = self.get_plot_area();
        let (left, right) = (bb.lower_left.x, bb.upper_right.x);

        cr.set_source_rgba(color_rgba.0, color_rgba.1, color_rgba.2, color_rgba.3);

        for layer in layers {
            let bottom_press = if let Some(press) = layer.bottom.pressure.into_option() {
                press
            } else {
                continue;
            };

            let top_press = if let Some(press) = layer.top.pressure.into_option() {
                press
            } else {
                continue;
            };

            let mut coords = [
                (left, bottom_press.unpack()),
                (left, top_press.unpack()),
                (right, top_press.unpack()),
                (right, bottom_press.unpack()),
            ];

            // Convert points to screen coords
            for coord in &mut coords {
                coord.1 = convert_pressure_to_y(HectoPascal(coord.1));

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
            cr.fill().unwrap();
        }
    }

    fn draw_levels(
        &self,
        args: DrawingArgs<'_, '_>,
        levels: &[DataRow],
        color_rgba: Rgba,
        line_width: f64,
    ) {
        let cr = args.cr;

        let bb = self.get_plot_area();
        let (left, right) = (bb.lower_left.x, bb.upper_right.x);

        cr.set_source_rgba(color_rgba.0, color_rgba.1, color_rgba.2, color_rgba.3);

        for level in levels {
            let press = if let Some(press) = level.pressure.into_option() {
                press
            } else {
                continue;
            };

            let mut coords = [(left, press.unpack()), (right, press.unpack())];

            // Convert points to screen coords
            for coord in &mut coords {
                coord.1 = convert_pressure_to_y(HectoPascal(coord.1));

                let screen_coords = self.convert_xy_to_screen(XYCoords {
                    x: coord.0,
                    y: coord.1,
                });

                coord.0 = screen_coords.x;
                coord.1 = screen_coords.y;
            }

            plot_curve_from_points(
                cr,
                line_width,
                color_rgba,
                coords.iter().map(|&(x, y)| ScreenCoords { x, y }),
            );
        }
    }
}
