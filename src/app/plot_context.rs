
use cairo::Context;

use coords::{DeviceCoords, ScreenCoords, XYCoords, ScreenRect, XYRect};

use super::AppContext;

pub trait PlotContext {
    /// Get the device width
    fn get_device_width(&self) -> i32;

    /// Set the device width
    fn set_device_width(&mut self, new_width: i32);

    /// Get device height
    fn get_device_height(&self) -> i32;

    /// Set device height
    fn set_device_height(&mut self, new_height: i32);

    /// Get a bounding box in XYCoords around all the data in this plot.
    fn get_xy_envelope(&self) -> XYRect;

    /// Set the bounding box in XYCoords around all the data in this plot.
    fn set_xy_envelope(&mut self, XYRect);

    /// Get zoom factor
    fn get_zoom_factor(&self) -> f64;

    /// Set zoom factor
    fn set_zoom_factor(&mut self, new_zoom_factor: f64);

    /// Get the translation between screen and  `XYCoords`
    fn get_translate(&self) -> XYCoords;

    /// Set the translation between screen and `XYCoords`
    fn set_translate(&mut self, new_translate: XYCoords);

    /// Get whether or not the left mouse button is pressed over this widget.
    fn get_left_button_pressed(&self) -> bool;

    /// Set whether or not the left mouse button is pressed over this widget
    fn set_left_button_pressed(&mut self, pressed: bool);

    /// Get the last position of the cursor over this widget.
    fn get_last_cursor_position(&self) -> Option<DeviceCoords>;

    /// Set the last position of the cursor over this widget.
    fn set_last_cursor_position<T>(&mut self, new_position: T)
    where
        Option<DeviceCoords>: From<T>;

    /// Get the distance used for adding padding around labels in `ScreenCoords`
    fn get_label_padding(&self) -> f64;

    /// Set the distance used for adding padding around labels in `ScreenCoords`
    fn set_label_padding(&mut self, new_padding: f64);

    /// Get the distance using for keeping things too close to the edge of the window in
    ///  `ScreenCoords`
    fn get_edge_padding(&self) -> f64;

    /// Set the distance using for keeping things too close to the edge of the window in
    ///  `ScreenCoords`
    fn set_edge_padding(&mut self, new_padding: f64);

    /// Conversion from (x,y) coords to screen coords
    fn convert_xy_to_screen(&self, coords: XYCoords) -> ScreenCoords {

        let translate = self.get_translate();

        // Apply translation first
        let x = coords.x - translate.x;
        let y = coords.y - translate.y;

        // Apply scaling
        let x = self.get_zoom_factor() * x;
        let y = self.get_zoom_factor() * y;
        ScreenCoords { x, y }
    }

    /// Conversion from device coordinates to `ScreenCoords`
    fn convert_device_to_screen(&self, coords: DeviceCoords) -> ScreenCoords {
        let scale_factor = self.scale_factor();
        ScreenCoords {
            x: coords.col / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            y: -(coords.row / scale_factor) + f64::from(self.get_device_height()) / scale_factor,
        }
    }

    /// This is the scale factor that will be set for the cairo transform matrix.
    ///
    /// By using this scale factor, it makes a distance of 1 in `XYCoords` equal to a distance of
    /// 1 in `ScreenCoords` when the zoom factor is 1.
    fn scale_factor(&self) -> f64 {
        f64::from(::std::cmp::min(
            self.get_device_height(),
            self.get_device_width(),
        ))
    }

    /// Convert device coords to (x,y) coords
    fn convert_device_to_xy(&self, coords: DeviceCoords) -> XYCoords {
        let screen_coords = self.convert_device_to_screen(coords);
        self.convert_screen_to_xy(screen_coords)
    }

    /// Conversion from (x,y) coords to screen coords
    fn convert_screen_to_xy(&self, coords: ScreenCoords) -> XYCoords {
        // Screen coords go 0 -> 1 down the y axis and 0 -> aspect_ratio right along the x axis.

        let translate = self.get_translate();

        let x = coords.x / self.get_zoom_factor() + translate.x;
        let y = coords.y / self.get_zoom_factor() + translate.y;
        XYCoords { x, y }
    }

    /// Get the edges of the X-Y plot area in `ScreenCoords`, may or may not be on the screen.
    fn calculate_plot_edges(&self, cr: &Context, ac: &AppContext) -> ScreenRect {

        let ScreenRect {
            lower_left,
            upper_right,
        } = self.bounding_box_in_screen_coords();
        let ScreenCoords {
            x: mut screen_x_min,
            y: mut screen_y_min,
        } = lower_left;
        let ScreenCoords {
            x: mut screen_x_max,
            y: mut screen_y_max,
        } = upper_right;

        // If screen area is bigger than plot area, labels will be clipped, keep them on the plot
        let ScreenCoords { x: xmin, y: ymin } =
            self.convert_xy_to_screen(XYCoords { x: 0.0, y: 0.0 });
        let ScreenCoords { x: xmax, y: ymax } =
            self.convert_xy_to_screen(XYCoords { x: 1.0, y: 1.0 });

        if xmin > screen_x_min {
            screen_x_min = xmin;
        }
        if xmax < screen_x_max {
            screen_x_max = xmax;
        }
        if ymax < screen_y_max {
            screen_y_max = ymax;
        }
        if ymin > screen_y_min {
            screen_y_min = ymin;
        }

        // Add some padding to keep away from the window edge
        let padding = cr.device_to_user_distance(ac.config.label_padding, 0.0).0;
        screen_x_max -= padding;
        screen_y_max -= padding;
        screen_x_min += padding;
        screen_y_min += padding;

        ScreenRect {
            lower_left: ScreenCoords {
                x: screen_x_min,
                y: screen_y_min,
            },
            upper_right: ScreenCoords {
                x: screen_x_max,
                y: screen_y_max,
            },
        }
    }

    /// Get a bounding box in screen coords
    fn bounding_box_in_screen_coords(&self) -> ScreenRect {
        let lower_left = self.convert_device_to_screen(DeviceCoords {
            col: 0.0,
            row: f64::from(self.get_device_height()),
        });
        let upper_right = self.convert_device_to_screen(DeviceCoords {
            col: f64::from(self.get_device_width()),
            row: 0.0,
        });

        ScreenRect {
            lower_left,
            upper_right,
        }
    }

    /// Left justify the plot in the view if zoomed out, and if zoomed in don't let it view
    /// beyond the edges of the plot.
    fn bound_view(&mut self) {

        let bounds = DeviceCoords {
            col: f64::from(self.get_device_width()),
            row: f64::from(self.get_device_height()),
        };
        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy(DeviceCoords { col: 0.0, row: 0.0 });
        let width = lower_right.x - upper_left.x;
        let height = upper_left.y - lower_right.y;

        let mut translate = self.get_translate();
        if width <= 1.0 {
            if translate.x < 0.0 {
                translate.x = 0.0;
            }
            let max_x = 1.0 - width;
            if translate.x > max_x {
                translate.x = max_x;
            }
        } else {
            translate.x = 0.0;
        }
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
}

pub struct GenericContext {
    // Rectangle that bounds all the values to be plotted in `XYCoords`.
    xy_envelope: XYRect,

    // Standard x-y coords, used for zooming and panning.
    zoom_factor: f64, // Multiply by this after translating
    translate: XYCoords,

    // device dimensions
    device_height: i32,
    device_width: i32,

    // state of input for left button press and panning.
    left_button_pressed: bool,

    // last cursor position in skew_t widget, used for sampling and panning
    last_cursor_position: Option<DeviceCoords>,

    // Distance used for adding padding around labels in `ScreenCoords`
    label_padding: f64,
    // Distance using for keeping things too close to the edge of the window in `ScreenCoords`
    edge_padding: f64,
}

impl GenericContext {
    pub fn new() -> Self {
        GenericContext {
            xy_envelope: XYRect {
                lower_left: XYCoords { x: 0.0, y: 0.0 },
                upper_right: XYCoords { x: 1.0, y: 1.0 },
            },

            // Sounding Area GUI state
            zoom_factor: 1.0,
            translate: XYCoords::origin(),
            device_height: 100,
            device_width: 100,
            last_cursor_position: None,
            left_button_pressed: false,

            // Drawing cache
            edge_padding: 0.0,
            label_padding: 0.0,
        }
    }
}

impl PlotContext for GenericContext {
    fn set_device_width(&mut self, new_width: i32) {
        self.device_width = new_width;
    }

    fn set_device_height(&mut self, new_height: i32) {
        self.device_height = new_height;
    }

    fn get_device_width(&self) -> i32 {
        self.device_width
    }

    fn get_device_height(&self) -> i32 {
        self.device_height
    }

    fn get_xy_envelope(&self) -> XYRect {
        self.xy_envelope
    }

    fn set_xy_envelope(&mut self, new_envelope: XYRect) {
        self.xy_envelope = new_envelope;
    }

    fn get_zoom_factor(&self) -> f64 {
        self.zoom_factor
    }

    fn set_zoom_factor(&mut self, new_zoom_factor: f64) {
        self.zoom_factor = new_zoom_factor;
    }

    fn get_translate(&self) -> XYCoords {
        self.translate
    }

    fn set_translate(&mut self, new_translate: XYCoords) {
        self.translate = new_translate;
    }

    fn get_left_button_pressed(&self) -> bool {
        self.left_button_pressed
    }

    fn set_left_button_pressed(&mut self, pressed: bool) {
        self.left_button_pressed = pressed;
    }

    fn get_last_cursor_position(&self) -> Option<DeviceCoords> {
        self.last_cursor_position
    }

    fn set_last_cursor_position<T>(&mut self, new_position: T)
    where
        Option<DeviceCoords>: From<T>,
    {
        self.last_cursor_position = Option::from(new_position);
    }

    fn get_label_padding(&self) -> f64 {
        self.label_padding
    }

    fn set_label_padding(&mut self, new_padding: f64) {
        self.label_padding = new_padding;
    }

    fn get_edge_padding(&self) -> f64 {
        self.edge_padding
    }

    fn set_edge_padding(&mut self, new_padding: f64) {
        self.edge_padding = new_padding;
    }
}

pub trait HasGenericContext {
    fn get_generic_context(&self) -> &GenericContext;
    fn get_generic_context_mut(&mut self) -> &mut GenericContext;
}

impl<T> PlotContext for T
where
    T: HasGenericContext,
{
    fn set_device_width(&mut self, new_width: i32) {
        self.get_generic_context_mut().set_device_width(new_width);
    }

    fn set_device_height(&mut self, new_height: i32) {
        self.get_generic_context_mut().set_device_height(new_height);
    }

    fn get_device_width(&self) -> i32 {
        self.get_generic_context().get_device_width()
    }

    fn get_device_height(&self) -> i32 {
        self.get_generic_context().get_device_height()
    }

    fn get_xy_envelope(&self) -> XYRect {
        self.get_generic_context().get_xy_envelope()
    }

    fn set_xy_envelope(&mut self, new_envelope: XYRect) {
        self.get_generic_context_mut().set_xy_envelope(new_envelope);
    }

    fn get_zoom_factor(&self) -> f64 {
        self.get_generic_context().get_zoom_factor()
    }

    fn set_zoom_factor(&mut self, new_zoom_factor: f64) {
        self.get_generic_context_mut().set_zoom_factor(
            new_zoom_factor,
        );
    }

    fn get_translate(&self) -> XYCoords {
        self.get_generic_context().get_translate()
    }

    fn set_translate(&mut self, new_translate: XYCoords) {
        self.get_generic_context_mut().set_translate(new_translate);
    }

    fn get_left_button_pressed(&self) -> bool {
        self.get_generic_context().get_left_button_pressed()
    }

    fn set_left_button_pressed(&mut self, pressed: bool) {
        self.get_generic_context_mut().set_left_button_pressed(
            pressed,
        );
    }

    fn get_last_cursor_position(&self) -> Option<DeviceCoords> {
        self.get_generic_context().get_last_cursor_position()
    }

    fn set_last_cursor_position<U>(&mut self, new_position: U)
    where
        Option<DeviceCoords>: From<U>,
    {
        self.get_generic_context_mut().set_last_cursor_position(
            new_position,
        );
    }

    fn get_label_padding(&self) -> f64 {
        self.get_generic_context().get_label_padding()
    }

    fn set_label_padding(&mut self, new_padding: f64) {
        self.get_generic_context_mut().set_label_padding(
            new_padding,
        );
    }

    fn get_edge_padding(&self) -> f64 {
        self.get_generic_context().get_edge_padding()
    }

    fn set_edge_padding(&mut self, new_padding: f64) {
        self.get_generic_context_mut().set_edge_padding(new_padding);
    }
}
