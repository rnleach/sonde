use crate::{
    app::config::{self},
    coords::{ScreenCoords, XYCoords},
    gui::{
        plot_context::{PlotContext, PlotContextExt},
        utility::plot_curve_from_points,
        Drawable, DrawingArgs,
    },
};
use cairo::{FontFace, FontSlant, FontWeight};

mod fire_plume_height;
pub use fire_plume_height::FirePlumeContext;

mod fire_plume_energy;
pub use fire_plume_energy::FirePlumeEnergyContext;

fn convert_fp_to_x(fp: f64) -> f64 {
    let min_fp = config::MIN_FIRE_POWER;
    let max_fp = config::MAX_FIRE_POWER;

    (fp - min_fp) / (max_fp - min_fp)
}

pub fn convert_x_to_fp(x: f64) -> f64 {
    let min_fp = config::MIN_FIRE_POWER;
    let max_fp = config::MAX_FIRE_POWER;

    x * (max_fp - min_fp) + min_fp
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

    let font_face = &FontFace::toy_create(&config.font_name, FontSlant::Normal, FontWeight::Bold);
    cr.set_font_face(font_face);

    context.set_font_size(config.label_font_size * 2.0, cr);
}

fn draw_iso_fps<T>(context: &T, config: &config::Config, cr: &cairo::Context)
where
    T: PlotContextExt,
{
    for pnts in config::FIRE_PLUME_FIRE_POWER_PNTS.iter() {
        let pnts = pnts
            .iter()
            .map(|xy_coords| context.convert_xy_to_screen(*xy_coords));
        plot_curve_from_points(cr, config.background_line_width, config.isobar_rgba, pnts);
    }
}
