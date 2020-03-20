use super::super::SkewTContext;
use crate::{
    analysis::{Intensity, PrecipTypeAlgorithm},
    app::config::{Rgba, BLUE, GREEN, RED},
    coords::ScreenCoords,
    gui::{Drawable, DrawingArgs, PlotContextExt},
};
use sounding_analysis::{self, PrecipType};

const PRECIP_BOX_SIZE: f64 = 0.07;

impl SkewTContext {
    pub fn draw_precip_icon(
        &self,
        algo: PrecipTypeAlgorithm,
        box_num: u8,
        args: DrawingArgs<'_, '_>,
    ) {
        use PrecipTypeAlgorithm::*;

        let (ac, cr, config) = (args.ac, args.cr, args.ac.config.borrow());

        let (wx_symbol_code, method_str) = if let Some(vals) = match algo {
            Model => ac
                .get_sounding_for_display()
                .and_then(|anal| anal.borrow().provider_precip_type())
                .map(|code| (code, "--Model--")),
            Bourgouin => ac
                .get_sounding_for_display()
                .and_then(|anal| anal.borrow().bourgouin_precip_type())
                .map(|code| (code, "Bourgouin")),
            NSSL => std::option::Option::None,
        } {
            vals
        } else {
            return;
        };

        if wx_symbol_code == PrecipType::None {
            return;
        }

        self.prepare_to_make_text(args);

        let padding = cr.device_to_user_distance(config.edge_padding, 0.0).0;
        let text_extents = cr.text_extents(method_str);
        let mut width = PRECIP_BOX_SIZE;
        if width < text_extents.width + 2.0 * padding {
            width = text_extents.width + 2.0 * padding;
        }
        let height = PRECIP_BOX_SIZE + 2.0 * padding + text_extents.height;

        let mut box_area = self.get_plot_area();
        box_area.lower_left.x += PRECIP_BOX_SIZE;
        box_area.upper_right.x = box_area.lower_left.x + width;
        box_area.lower_left.y += PRECIP_BOX_SIZE + (2.0 * padding + height) * box_num as f64;
        box_area.upper_right.y = box_area.lower_left.y + height;

        Self::draw_legend_rectangle(args, &box_area);

        let box_center = ScreenCoords {
            x: box_area.lower_left.x + width / 2.0,
            y: box_area.lower_left.y + PRECIP_BOX_SIZE / 2.0 + text_extents.height + 2.0 * padding,
        };

        cr.move_to(box_center.x, box_center.y);
        use PrecipType::*;
        match wx_symbol_code {
            // Drizzle
            LightDrizzle => draw_point_symbol(cr, Intensity::Light, GREEN, draw_drizzle_comma),
            ModerateDrizzle => {
                draw_point_symbol(cr, Intensity::Moderate, GREEN, draw_drizzle_comma)
            }
            HeavyDrizzle => draw_point_symbol(cr, Intensity::Heavy, GREEN, draw_drizzle_comma),
            LightFreezingDrizzle => {
                draw_freezing_liquid_precip(cr, Intensity::Light, draw_drizzle_comma)
            }
            ModerateFreezingDrizzle => {
                draw_freezing_liquid_precip(cr, Intensity::Moderate, draw_drizzle_comma)
            }
            HeavyFreezingDrizzle => {
                draw_freezing_liquid_precip(cr, Intensity::Heavy, draw_drizzle_comma)
            }

            // Rain
            LightRain => draw_point_symbol(cr, Intensity::Light, GREEN, draw_rain_dot),
            ModerateRain => draw_point_symbol(cr, Intensity::Moderate, GREEN, draw_rain_dot),
            HeavyRain => draw_point_symbol(cr, Intensity::Heavy, GREEN, draw_rain_dot),
            LightFreezingRain => draw_freezing_liquid_precip(cr, Intensity::Light, draw_rain_dot),
            ModerateFreezingRain => {
                draw_freezing_liquid_precip(cr, Intensity::Moderate, draw_rain_dot)
            }
            HeavyFreezingRain => draw_freezing_liquid_precip(cr, Intensity::Heavy, draw_rain_dot),
            LightRainAndSnow => draw_mixed_rain_snow(cr, Intensity::Light),
            ModerateRainAndSnow => draw_mixed_rain_snow(cr, Intensity::Moderate),

            // Ice precipitation
            LightSnow => draw_point_symbol(cr, Intensity::Light, BLUE, draw_snowflake),
            ModerateSnow => draw_point_symbol(cr, Intensity::Moderate, BLUE, draw_snowflake),
            HeavySnow => draw_point_symbol(cr, Intensity::Heavy, BLUE, draw_snowflake),
            LightIcePellets => draw_ice_pellets(cr),
            ModerateIcePellets => draw_point_symbol(cr, Intensity::Moderate, RED, draw_ice_pellet),
            HeavyIcePellets => draw_point_symbol(cr, Intensity::Heavy, RED, draw_ice_pellet),

            // Showers
            LightRainShowers => draw_showers(cr, Intensity::Light, GREEN, draw_rain_dot),
            ModerateRainShowers => draw_showers(cr, Intensity::Moderate, GREEN, draw_rain_dot),
            HeavyRainShowers => draw_showers(cr, Intensity::Heavy, GREEN, draw_rain_dot),
            LightSnowShowers => draw_showers(cr, Intensity::Light, BLUE, draw_snowflake),
            ModerateSnowShowers => draw_showers(cr, Intensity::Moderate, BLUE, draw_snowflake),
            HeavySnowShowers => draw_showers(cr, Intensity::Heavy, BLUE, draw_snowflake),

            _ => draw_red_x(cr),
        }

        let mut text_home = ScreenCoords {
            x: box_area.lower_left.x + padding,
            y: box_area.lower_left.y + padding,
        };

        let slack = width - text_extents.width - 2.0 * padding;
        if slack > 0.0 {
            text_home.x += slack / 2.0;
        }

        let rgb = config.label_rgba;
        cr.set_source_rgba(rgb.0, rgb.1, rgb.2, rgb.3);
        cr.set_line_width(cr.device_to_user_distance(2.0, 0.0).0);
        cr.move_to(text_home.x, text_home.y);
        cr.show_text(method_str);

        cr.move_to(
            box_area.lower_left.x,
            box_area.lower_left.y + text_extents.height + 2.0 * padding,
        );
        cr.rel_line_to(width, 0.0);
        cr.stroke();
    }
}
fn draw_point_symbol<F: Fn(&cairo::Context, f64)>(
    cr: &cairo::Context,
    inten: Intensity,
    color: Rgba,
    draw_func: F,
) {
    const GRID_SIZE: f64 = PRECIP_BOX_SIZE / 5.0;
    const PNT_SIZE: f64 = GRID_SIZE / 2.0; // divide by 2.0 for radius
    const A: f64 = std::f64::consts::SQRT_2 * GRID_SIZE;

    cr.set_source_rgba(color.0, color.1, color.2, color.3);

    let (x, y) = cr.get_current_point();

    match inten {
        Intensity::Light => {
            cr.move_to(x - A / 2.0, y);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x + A / 2.0, y);
            draw_func(cr, PNT_SIZE);
        }
        Intensity::Moderate => {
            const H_SQ: f64 = 3.0 * A * A / 4.0;
            let h = H_SQ.sqrt();
            let yt = (A * A + 4.0 * H_SQ) / (8.0 * h);

            cr.move_to(x, y + yt);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x + A / 2.0, y - h + yt);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x - A / 2.0, y - h + yt);
            draw_func(cr, PNT_SIZE);
        }
        Intensity::Heavy => {
            cr.move_to(x, y + GRID_SIZE);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x, y - GRID_SIZE);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x + GRID_SIZE, y);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x - GRID_SIZE, y);
            draw_func(cr, PNT_SIZE);
        }
    }
}

fn draw_showers<F: Fn(&cairo::Context, f64)>(
    cr: &cairo::Context,
    inten: Intensity,
    color: Rgba,
    draw_func: F,
) {
    const GRID_SIZE: f64 = PRECIP_BOX_SIZE / 5.0;
    const PNT_SIZE: f64 = GRID_SIZE / 2.0 / 1.5; // divide by 2.0 for radius
    const A: f64 = std::f64::consts::SQRT_2 * GRID_SIZE / 1.5;

    const H: f64 = 2.5 * GRID_SIZE;
    const DROP: f64 = 0.35 * GRID_SIZE;
    const HPRIME: f64 = H - DROP;
    const HALF_WIDTH: f64 = GRID_SIZE;
    const HALF_WIDTH_PRIME: f64 = HPRIME / H * HALF_WIDTH;

    cr.set_source_rgba(color.0, color.1, color.2, color.3);
    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);

    // Draw the triangle
    let (x, y) = cr.get_current_point();
    let y = y + 0.7 * GRID_SIZE;

    cr.move_to(x - HALF_WIDTH, y);
    cr.line_to(x + HALF_WIDTH, y);
    cr.line_to(x, y - H);
    cr.close_path();
    cr.stroke();

    match inten {
        Intensity::Light => {}
        Intensity::Moderate | Intensity::Heavy => {
            cr.move_to(x - HALF_WIDTH_PRIME, y - DROP);
            cr.line_to(x + HALF_WIDTH_PRIME, y - DROP);
            cr.stroke();
        }
    }

    // Draw the points
    match inten {
        Intensity::Light | Intensity::Moderate => {
            // draw a dot
            cr.move_to(x, y + A);
            draw_func(cr, PNT_SIZE);
        }
        Intensity::Heavy => {
            //draw 2 dots
            cr.move_to(x - A / 2.0, y + A);
            draw_func(cr, PNT_SIZE);
            cr.move_to(x + A / 2.0, y + A);
            draw_func(cr, PNT_SIZE);
        }
    }
}

fn draw_mixed_rain_snow(cr: &cairo::Context, inten: Intensity) {
    const GRID_SIZE: f64 = PRECIP_BOX_SIZE / 5.0;
    const PNT_SIZE: f64 = GRID_SIZE / 2.0; // divide by 2.0 for radius
    const A: f64 = std::f64::consts::SQRT_2 * GRID_SIZE;

    let (x, y) = cr.get_current_point();

    match inten {
        Intensity::Light => {
            cr.set_source_rgba(GREEN.0, GREEN.1, GREEN.2, GREEN.3);
            cr.move_to(x, y + A / 2.0);
            draw_rain_dot(cr, PNT_SIZE);

            cr.set_source_rgba(BLUE.0, BLUE.1, BLUE.2, BLUE.3);
            cr.move_to(x, y - A / 2.0);
            draw_snowflake(cr, PNT_SIZE);
        }
        Intensity::Moderate | Intensity::Heavy => {
            cr.set_source_rgba(GREEN.0, GREEN.1, GREEN.2, GREEN.3);
            draw_rain_dot(cr, PNT_SIZE);

            cr.set_source_rgba(BLUE.0, BLUE.1, BLUE.2, BLUE.3);
            cr.move_to(x, y + A);
            draw_snowflake(cr, PNT_SIZE);
            cr.move_to(x, y - A);
            draw_snowflake(cr, PNT_SIZE);
        }
    }
}

fn draw_freezing_liquid_precip<F: Fn(&cairo::Context, f64)>(
    cr: &cairo::Context,
    intensity: Intensity,
    draw_func: F,
) {
    use std::f64::consts::PI;

    const PNT_SIZE: f64 = PRECIP_BOX_SIZE / 7.0 / 2.0; // divide by 2.0 for radius

    if intensity == Intensity::Heavy {
        cr.rel_move_to(0.0, -0.5 * PRECIP_BOX_SIZE / 5.0);
    }

    let (mut x, mut y) = cr.get_current_point();
    x -= PRECIP_BOX_SIZE / 5.0;

    cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
    cr.move_to(x, y);
    draw_func(cr, PNT_SIZE);

    // Draw the almost infinity line.
    let radius = PRECIP_BOX_SIZE / 5.0 / 1.2;
    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);
    cr.arc_negative(x, y, radius, 5.0 * PI / 4.0, 9.0 * PI / 4.0);
    x += PRECIP_BOX_SIZE / 5.0 * 2.0;
    cr.arc(x, y, radius, 5.0 * PI / 4.0, 9.0 * PI / 4.0);
    cr.stroke();

    if intensity == Intensity::Moderate || intensity == Intensity::Heavy {
        cr.move_to(x, y);
        draw_func(cr, PNT_SIZE);
    }

    if intensity == Intensity::Heavy {
        x -= PRECIP_BOX_SIZE / 5.0;
        y += PRECIP_BOX_SIZE / 5.0 * 1.5;
        cr.move_to(x, y);
        draw_func(cr, PNT_SIZE);
    }
}

fn draw_ice_pellets(cr: &cairo::Context) {
    use std::f64::consts::PI;
    const PNT_SIZE: f64 = PRECIP_BOX_SIZE / 7.0 / 2.0; // divide by 2.0 for radius
    #[allow(non_snake_case)]
    let TRIANGLE_HEIGHT: f64 = PRECIP_BOX_SIZE * 3.0 * 3.0f64.sqrt() / 10.0;
    const TRIANGLE_WIDTH: f64 = 3.0 * PRECIP_BOX_SIZE / 5.0;
    #[allow(non_snake_case)]
    let Y: f64 = (TRIANGLE_WIDTH * TRIANGLE_WIDTH / 4.0 + TRIANGLE_HEIGHT * TRIANGLE_HEIGHT)
        / (2.0 * TRIANGLE_HEIGHT);

    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);

    cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
    cr.rel_move_to(0.0, TRIANGLE_HEIGHT / 2.0 - Y);
    let (x, y) = cr.get_current_point();
    cr.arc(x, y, PNT_SIZE, 0.0, 2.0 * PI);
    cr.fill();

    cr.move_to(x, y);
    cr.rel_move_to(0.0, Y);
    cr.rel_line_to(TRIANGLE_WIDTH / 2.0, -TRIANGLE_HEIGHT);
    cr.rel_line_to(-TRIANGLE_WIDTH, 0.0);
    cr.close_path();
    cr.stroke();
}

fn draw_red_x(cr: &cairo::Context) {
    cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
    cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
    cr.rel_move_to(-PRECIP_BOX_SIZE / 2.0, -PRECIP_BOX_SIZE / 2.0);
    cr.rel_line_to(PRECIP_BOX_SIZE, PRECIP_BOX_SIZE);
    cr.rel_move_to(-PRECIP_BOX_SIZE, 0.0);
    cr.rel_line_to(PRECIP_BOX_SIZE, -PRECIP_BOX_SIZE);
    cr.stroke();
}

fn draw_rain_dot(cr: &cairo::Context, pnt_size: f64) {
    use std::f64::consts::PI;

    let (x, y) = cr.get_current_point();
    cr.arc(x, y, pnt_size, 0.0, 2.0 * PI);
    cr.fill();
}

fn draw_drizzle_comma(cr: &cairo::Context, pnt_size: f64) {
    use std::f64::consts::PI;
    let comma_size = 2.0 * pnt_size / 3.0;

    cr.set_line_width(cr.device_to_user_distance(3.0, 0.0).0);
    let (x, y) = cr.get_current_point();
    cr.arc(x, y, comma_size, 0.0, 2.0 * PI);
    cr.fill_preserve();
    cr.stroke();

    cr.arc(
        x - comma_size,
        y - comma_size / 5.0,
        2.0 * comma_size,
        -PI / 2.5,
        0.0,
    );
    cr.stroke();
}

fn draw_snowflake(cr: &cairo::Context, _pnt_size: f64) {
    const ANGLE: f64 = std::f64::consts::PI * 2.0 / 5.0;
    const A: f64 = PRECIP_BOX_SIZE / 5.0 / 2.0;

    cr.set_line_width(cr.device_to_user_distance(2.5, 0.0).0);

    cr.save();

    let (x, y) = cr.get_current_point();
    cr.translate(x, y);

    cr.rel_line_to(0.0, A);
    for _ in 0..5 {
        cr.rel_move_to(0.0, -A);
        cr.rotate(ANGLE);
        cr.rel_line_to(0.0, A);
    }

    cr.stroke();
    cr.restore();
}

fn draw_ice_pellet(cr: &cairo::Context, _pnt_size: f64) {
    use std::f64::consts::PI;
    const GRID_SIZE: f64 = PRECIP_BOX_SIZE / 5.0;
    const PNT_SIZE: f64 = GRID_SIZE / 2.0 / 2.0; // divide by 2.0 for radius, and 2 again for small

    #[allow(non_snake_case)]
    let TRIANGLE_HEIGHT: f64 = 1.2 * GRID_SIZE * 3.0f64.sqrt() / 2.0;
    const TRIANGLE_WIDTH: f64 = 1.2 * GRID_SIZE;
    #[allow(non_snake_case)]
    let Y: f64 = (TRIANGLE_WIDTH * TRIANGLE_WIDTH / 4.0 + TRIANGLE_HEIGHT * TRIANGLE_HEIGHT)
        / (2.0 * TRIANGLE_HEIGHT);

    cr.set_line_width(cr.device_to_user_distance(1.2, 0.0).0);

    cr.save();

    let (x, y) = cr.get_current_point();
    cr.translate(x, y);

    cr.rel_move_to(0.0, TRIANGLE_HEIGHT / 2.0 - Y);
    let (x, y) = cr.get_current_point();
    cr.arc(x, y, PNT_SIZE, 0.0, 2.0 * PI);
    cr.fill();

    cr.move_to(x, y);
    cr.rel_move_to(0.0, Y);
    cr.rel_line_to(TRIANGLE_WIDTH / 2.0, -TRIANGLE_HEIGHT);
    cr.rel_line_to(-TRIANGLE_WIDTH, 0.0);
    cr.close_path();
    cr.stroke();
    cr.restore();
}
