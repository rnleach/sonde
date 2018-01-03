use std::cell::{Cell, RefCell};

use cairo::{Context, Format, ImageSurface, Matrix, MatrixTrait, Operator};
use gtk::DrawingArea;
use gtk::prelude::*;

use app::AppContextPointer;
use coords::{DeviceCoords, DeviceRect, Rect, ScreenCoords, ScreenRect, XYCoords, XYRect};
use gui::{AppContext, DrawingArgs};

pub trait PlotContext {
    /// Set the width and height of the plot in device
    fn set_device_rect(&self, rect: DeviceRect);

    /// Get the device dimensions
    fn get_device_rect(&self) -> DeviceRect;

    /// Get a bounding box in XYCoords around all the data in this plot.
    fn get_xy_envelope(&self) -> XYRect;

    /// Set the bounding box in XYCoords around all the data in this plot.
    fn set_xy_envelope(&self, XYRect);

    /// Get zoom factor
    fn get_zoom_factor(&self) -> f64;

    /// Set zoom factor
    fn set_zoom_factor(&self, new_zoom_factor: f64);

    /// Get the translation between screen and  `XYCoords`
    fn get_translate(&self) -> XYCoords;

    /// Set the translation between screen and `XYCoords`
    fn set_translate(&self, new_translate: XYCoords);

    /// Get whether or not the left mouse button is pressed over this widget.
    fn get_left_button_pressed(&self) -> bool;

    /// Set whether or not the left mouse button is pressed over this widget
    fn set_left_button_pressed(&self, pressed: bool);

    /// Get the last position of the cursor over this widget.
    fn get_last_cursor_position(&self) -> Option<DeviceCoords>;

    /// Set the last position of the cursor over this widget.
    fn set_last_cursor_position<T>(&self, new_position: T)
    where
        Option<DeviceCoords>: From<T>;

    /// Get the matrix used to transform `ScreenCoords` to `DeviceCoords`
    fn get_matrix(&self) -> Matrix;

    /// Set the matrix used to transform `ScreenCoords` to `DeviceCoords`
    fn set_matrix(&self, matrix: Matrix);

    fn mark_background_dirty(&self);
    fn clear_background_dirty(&self);
    fn is_background_dirty(&self) -> bool;

    fn mark_data_dirty(&self);
    fn clear_data_dirty(&self);
    fn is_data_dirty(&self) -> bool;

    fn mark_overlay_dirty(&self);
    fn clear_overlay_dirty(&self);
    fn is_overlay_dirty(&self) -> bool;

    fn get_background_layer(&self) -> ImageSurface;
    fn set_background_layer(&self, new_surface: ImageSurface);
    fn get_data_layer(&self) -> ImageSurface;
    fn set_data_layer(&self, new_surface: ImageSurface);
    fn get_overlay_layer(&self) -> ImageSurface;
    fn set_overlay_layer(&self, new_surface: ImageSurface);
}

pub trait PlotContextExt: PlotContext {
    fn update_cache_allocations(&self, da: &DrawingArea) {
        // Mark everything as dirty
        self.mark_background_dirty(); // Marks everything

        // Get the size
        let (width, height) = (da.get_allocation().width, da.get_allocation().height);

        // Make the new allocations
        self.set_background_layer(ImageSurface::create(Format::ARgb32, width, height).unwrap());
        self.set_data_layer(ImageSurface::create(Format::ARgb32, width, height).unwrap());
        self.set_overlay_layer(ImageSurface::create(Format::ARgb32, width, height).unwrap());
    }

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
        let device_rect = self.get_device_rect();
        let height = device_rect.height();
        let upper_left = device_rect.upper_left;
        ScreenCoords {
            x: (coords.col - upper_left.col) / scale_factor,
            // Flip y coordinate vertically and translate so origin is upper left corner.
            y: -((coords.row - upper_left.row) / scale_factor) + height / scale_factor,
        }
    }

    /// This is the scale factor that will be set for the cairo transform matrix.
    ///
    /// By using this scale factor, it makes a distance of 1 in `XYCoords` equal to a distance of
    /// 1 in `ScreenCoords` when the zoom factor is 1.
    fn scale_factor(&self) -> f64 {
        let device_rect = self.get_device_rect();
        if device_rect.width() < device_rect.height() {
            device_rect.width()
        } else {
            device_rect.height()
        }
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
        let padding = cr.device_to_user_distance(ac.config.borrow().label_padding, 0.0)
            .0;
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
        let device_rect = self.get_device_rect();

        let lower_left = self.convert_device_to_screen(DeviceCoords {
            col: device_rect.upper_left.col,
            row: device_rect.height + device_rect.upper_left.row,
        });
        let upper_right = self.convert_device_to_screen(DeviceCoords {
            col: device_rect.upper_left.col + device_rect.width,
            row: device_rect.upper_left.row,
        });

        ScreenRect {
            lower_left,
            upper_right,
        }
    }

    /// Left justify the plot in the view if zoomed out, and if zoomed in don't let it view
    /// beyond the edges of the plot.
    fn bound_view(&self) {
        let device_rect = self.get_device_rect();

        let bounds = DeviceCoords {
            col: device_rect.width,
            row: device_rect.height,
        };

        let lower_right = self.convert_device_to_xy(bounds);
        let upper_left = self.convert_device_to_xy(device_rect.upper_left);
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

    /// Zoom in the most possible while still keeping the whole envelope in view.
    fn zoom_to_envelope(&self) {
        use std::f64;

        let xy_envelope = self.get_xy_envelope();

        let lower_left = xy_envelope.lower_left;
        self.set_translate(lower_left);

        let width = xy_envelope.upper_right.x - xy_envelope.lower_left.x;
        let height = xy_envelope.upper_right.y - xy_envelope.lower_left.y;

        let width_scale = 1.0 / width;
        let height_scale = 1.0 / height;

        self.set_zoom_factor(f64::min(width_scale, height_scale));

        self.bound_view();
    }

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
}

pub struct GenericContext {
    // Area to draw in
    device_rect: Cell<DeviceRect>,

    // Rectangle that bounds all the values to be plotted in `XYCoords`.
    xy_envelope: Cell<XYRect>,

    // Standard x-y coords, used for zooming and panning.
    zoom_factor: Cell<f64>, // Multiply by this after translating
    translate: Cell<XYCoords>,

    // state of input for left button press and panning.
    left_button_pressed: Cell<bool>,

    // last cursor position in skew_t widget, used for sampling and panning
    last_cursor_position: Cell<Option<DeviceCoords>>,

    matrix: Cell<Matrix>,

    dirty_background: Cell<bool>,
    background_layer: RefCell<ImageSurface>,

    dirty_data: Cell<bool>,
    data_layer: RefCell<ImageSurface>,

    dirty_overlay: Cell<bool>,
    overlay_layer: RefCell<ImageSurface>,
}

impl GenericContext {
    pub fn new() -> Self {
        GenericContext {
            device_rect: Cell::new(DeviceRect {
                upper_left: DeviceCoords { row: 0.0, col: 0.0 },
                width: 1.0,
                height: 1.0,
            }),
            xy_envelope: Cell::new(XYRect {
                lower_left: XYCoords { x: 0.0, y: 0.0 },
                upper_right: XYCoords { x: 1.0, y: 1.0 },
            }),

            // Sounding Area GUI state
            zoom_factor: Cell::new(1.0),
            translate: Cell::new(XYCoords::origin()),
            last_cursor_position: Cell::new(None),
            left_button_pressed: Cell::new(false),

            matrix: Cell::new(Matrix::identity()),

            dirty_background: Cell::new(true),
            background_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),

            dirty_data: Cell::new(true),
            data_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),

            dirty_overlay: Cell::new(true),
            overlay_layer: RefCell::new(ImageSurface::create(Format::ARgb32, 5, 5).unwrap()),
        }
    }
}

pub trait HasGenericContext {
    fn get_generic_context(&self) -> &GenericContext;
}

impl<T> PlotContext for T
where
    T: HasGenericContext,
{
    fn set_device_rect(&self, rect: DeviceRect) {
        self.get_generic_context().device_rect.set(rect)
    }

    fn get_device_rect(&self) -> DeviceRect {
        self.get_generic_context().device_rect.get()
    }

    fn get_xy_envelope(&self) -> XYRect {
        self.get_generic_context().xy_envelope.get()
    }

    fn set_xy_envelope(&self, new_envelope: XYRect) {
        self.get_generic_context().xy_envelope.set(new_envelope);
    }

    fn get_zoom_factor(&self) -> f64 {
        self.get_generic_context().zoom_factor.get()
    }

    fn set_zoom_factor(&self, new_zoom_factor: f64) {
        self.get_generic_context().zoom_factor.set(new_zoom_factor);
    }

    fn get_translate(&self) -> XYCoords {
        self.get_generic_context().translate.get()
    }

    fn set_translate(&self, new_translate: XYCoords) {
        self.get_generic_context().translate.set(new_translate);
    }

    fn get_left_button_pressed(&self) -> bool {
        self.get_generic_context().left_button_pressed.get()
    }

    fn set_left_button_pressed(&self, pressed: bool) {
        self.get_generic_context().left_button_pressed.set(pressed);
    }

    fn get_last_cursor_position(&self) -> Option<DeviceCoords> {
        self.get_generic_context().last_cursor_position.get()
    }

    fn set_last_cursor_position<U>(&self, new_position: U)
    where
        Option<DeviceCoords>: From<U>,
    {
        self.get_generic_context()
            .last_cursor_position
            .set(Option::from(new_position));
    }

    fn get_matrix(&self) -> Matrix {
        self.get_generic_context().matrix.get()
    }

    fn set_matrix(&self, matrix: Matrix) {
        self.get_generic_context().matrix.set(matrix);
    }

    fn mark_background_dirty(&self) {
        self.get_generic_context().dirty_background.set(true);
        self.get_generic_context().dirty_data.set(true);
        self.get_generic_context().dirty_overlay.set(true);
    }

    fn clear_background_dirty(&self) {
        self.get_generic_context().dirty_background.set(false);
    }

    fn is_background_dirty(&self) -> bool {
        self.get_generic_context().dirty_background.get()
    }

    fn mark_data_dirty(&self) {
        self.get_generic_context().dirty_background.set(true);
        self.get_generic_context().dirty_data.set(true);
    }

    fn clear_data_dirty(&self) {
        self.get_generic_context().dirty_data.set(false);
    }

    fn is_data_dirty(&self) -> bool {
        self.get_generic_context().dirty_data.get()
    }

    fn mark_overlay_dirty(&self) {
        self.get_generic_context().dirty_overlay.set(true);
    }

    fn clear_overlay_dirty(&self) {
        self.get_generic_context().dirty_overlay.set(false);
    }

    fn is_overlay_dirty(&self) -> bool {
        self.get_generic_context().dirty_overlay.get()
    }

    fn get_background_layer(&self) -> ImageSurface {
        self.get_generic_context().background_layer.borrow().clone()
    }

    fn set_background_layer(&self, new_surface: ImageSurface) {
        *self.get_generic_context().background_layer.borrow_mut() = new_surface;
    }

    fn get_data_layer(&self) -> ImageSurface {
        self.get_generic_context().data_layer.borrow().clone()
    }

    fn set_data_layer(&self, new_surface: ImageSurface) {
        *self.get_generic_context().data_layer.borrow_mut() = new_surface;
    }

    fn get_overlay_layer(&self) -> ImageSurface {
        self.get_generic_context().overlay_layer.borrow().clone()
    }

    fn set_overlay_layer(&self, new_surface: ImageSurface) {
        *self.get_generic_context().overlay_layer.borrow_mut() = new_surface;
    }
}

pub trait Drawable: PlotContext + PlotContextExt {
    fn set_up_drawing_area(da: &DrawingArea, acp: &AppContextPointer);
    fn draw_background(&self, args: DrawingArgs);
    fn draw_data(&self, args: DrawingArgs);
    fn draw_overlays(&self, args: DrawingArgs);

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

            self.draw_data(tmp_args);

            self.clear_data_dirty();
        }

        cr.set_source_surface(&self.get_data_layer(), 0.0, 0.0);
        cr.paint();
    }

    fn draw_overlay_cached(&self, args: DrawingArgs) {
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

            self.draw_overlays(tmp_args);

            self.clear_overlay_dirty();
        }

        cr.set_source_surface(&self.get_overlay_layer(), 0.0, 0.0);
        cr.paint();
    }
}
