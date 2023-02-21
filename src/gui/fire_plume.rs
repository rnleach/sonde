use crate::{
    app::config::{self},
    coords::{ScreenCoords, XYCoords},
    gui::{
        plot_context::{PlotContext, PlotContextExt},
        utility::plot_curve_from_points,
//        Drawable, 
        DrawingArgs,
    },
};
use gtk::cairo::{FontFace, FontSlant, FontWeight};
use metfor::{CelsiusDiff, Quantity};

mod fire_plume_height;
pub use fire_plume_height::FirePlumeContext;

mod fire_plume_energy;
pub use fire_plume_energy::FirePlumeEnergyContext;

fn convert_dt_to_x(dt: CelsiusDiff) -> f64 {
    let min_dt = config::MIN_DELTA_T;
    let max_dt = config::MAX_DELTA_T;

    (dt - min_dt) / (max_dt - min_dt)
}

pub fn convert_x_to_dt(x: f64) -> CelsiusDiff {
    let min_dt = config::MIN_DELTA_T.unpack();
    let max_dt = config::MAX_DELTA_T.unpack();

    CelsiusDiff(x * (max_dt - min_dt) + min_dt)
}

fn convert_xy_to_screen<T>(context: &T, coords: XYCoords) -> ScreenCoords
where
    T: PlotContext,
{
    let translate = context.get_translate();

    // Apply translation first
    let x = coords.x - translate.x;
    let y = coords.y - translate.y;

    // Apply scaling
    // Use factor of two scaling because these are wider than tall
    let x = context.get_zoom_factor() * x * 2.0;
    let y = context.get_zoom_factor() * y;

    ScreenCoords { x, y }
}

fn convert_screen_to_xy<T>(context: &T, coords: ScreenCoords) -> XYCoords
where
    T: PlotContext,
{
    // Screen coords go 0 -> 1 down the y axis and 0 -> aspect_ratio right along the x axis.

    let translate = context.get_translate();

    let x = coords.x / context.get_zoom_factor() / 2.0 + translate.x;
    let y = coords.y / context.get_zoom_factor() + translate.y;
    XYCoords { x, y }
}

fn prepare_to_make_text<T>(context: &T, args: DrawingArgs<'_, '_>)
where
    T: Drawable,
{
    let (cr, config) = (args.cr, args.ac.config.borrow());

    let font_face =
        &FontFace::toy_create(&config.font_name, FontSlant::Normal, FontWeight::Bold).unwrap();
    cr.set_font_face(font_face);

    context.set_font_size(config.label_font_size * 2.0, cr);
}

fn draw_iso_dts<T>(context: &T, config: &config::Config, cr: &gtk::cairo::Context)
where
    T: PlotContextExt,
{
    for pnts in config::FIRE_PLUME_DT_PNTS.iter() {
        let pnts = pnts
            .iter()
            .map(|xy_coords| context.convert_xy_to_screen(*xy_coords));
        plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
    }
}
