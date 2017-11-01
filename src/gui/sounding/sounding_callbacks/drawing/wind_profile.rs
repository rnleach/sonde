use cairo::Context;

use app::AppContext;
use coords::{ScreenCoords, TPCoords, ScreenRect, Rect};

pub fn draw_wind_profile(cr: &Context, ac: &AppContext) {

    let snd = if let Some(snd) = ac.get_sounding_for_display() {
        snd
    } else {
        return;
    };

    let barb_config = WindBarbConfig::init(cr, ac);
    let barb_data = gather_wind_data(&snd, &barb_config);
    let barb_data = filter_wind_data(barb_data, ac);

    let rgba = ac.config.wind_rgba;
    cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    cr.set_line_width(
        cr.device_to_user_distance(ac.config.wind_barb_line_width, 0.0)
            .0,
    );

    for bdata in barb_data.into_iter() {
        bdata.draw(cr);
    }
}

fn gather_wind_data(
    snd: &::sounding_base::Sounding,
    barb_config: &WindBarbConfig,
) -> Vec<WindBarbData> {

    let dir = &snd.direction;
    let spd = &snd.speed;
    let pres = &snd.pressure;

    izip!(pres, dir, spd)
        .filter_map(|tuple| {
            let (p, d, s) = tuple;
            if p.as_option().is_some() && d.as_option().is_some() && s.as_option().is_some() {
                Some((p.unwrap(), d.unwrap(), s.unwrap()))
            } else {
                None
            }
        })
        .map(|tuple| {
            let (p, d, s) = tuple;
            WindBarbData::create(p, d, s, &barb_config)
        })
        .collect()
}

fn filter_wind_data(barb_data: Vec<WindBarbData>, ac: &AppContext) -> Vec<WindBarbData> {
    // Remove overlapping barbs, or barbs not on the screen
    let mut keepers: Vec<WindBarbData> = vec![];
    let screen_box = ac.bounding_box_in_screen_coords();
    let mut last_added_bbox: ScreenRect = ScreenRect {
        lower_left: ScreenCoords {
            x: ::std::f64::MAX,
            y: ::std::f64::MAX,
        },
        upper_right: ScreenCoords {
            x: ::std::f64::MAX,
            y: ::std::f64::MAX,
        },
    };
    for bdata in barb_data.into_iter() {
        let bbox = bdata.bounding_box();
        if !bbox.inside(&screen_box) || bbox.overlaps(&last_added_bbox) {
            continue;
        }
        last_added_bbox = bbox;
        keepers.push(bdata);
    }

    keepers
}

struct WindBarbConfig<'a> {
    shaft_length: f64,
    barb_length: f64,
    pennant_width: f64,
    xcoord: f64,
    dot_size: f64,
    ac: &'a AppContext,
}

impl<'a, 'b> WindBarbConfig<'a> {
    fn init(cr: &'b Context, ac: &'a AppContext) -> Self {
        let (shaft_length, barb_length) = cr.device_to_user_distance(
            ac.config.wind_barb_shaft_length,
            -ac.config.wind_barb_barb_length,
        );

        let (dot_size, pennant_width) = cr.device_to_user_distance(
            ac.config.wind_barb_dot_radius,
            -ac.config.wind_barb_pennant_width,
        );
        let padding = ac.edge_padding;
        let ScreenRect {
            lower_left: ScreenCoords { x: _xmin, y: _ymin },
            upper_right: ScreenCoords { x: xmax, y: _ymax },
        } = ac.bounding_box_in_screen_coords();
        let xcoord = xmax - padding - shaft_length;

        WindBarbConfig {
            shaft_length,
            barb_length,
            pennant_width,
            xcoord,
            dot_size,
            ac,
        }
    }
}

struct WindBarbData {
    center: ScreenCoords,
    shaft_end: ScreenCoords,
    num_pennants: usize,
    pennant_coords: [(ScreenCoords, ScreenCoords, ScreenCoords); 5],
    num_barbs: usize,
    barb_coords: [(ScreenCoords, ScreenCoords); 5],
    direction_radians: f64,
    point_radius: f64,
}

impl WindBarbData {
    fn create(pressure: f64, direction: f64, speed: f64, barb_config: &WindBarbConfig) -> Self {

        let center = get_wind_barb_center(pressure, barb_config.xcoord, barb_config.ac);

        // Convert angle to traditional XY coordinate plane
        let direction_radians = ::std::f64::consts::FRAC_PI_2 - direction.to_radians();

        let dx = barb_config.shaft_length * direction_radians.cos();
        let dy = barb_config.shaft_length * direction_radians.sin();

        let shaft_end = ScreenCoords {
            x: center.x + dx,
            y: center.y + dy,
        };

        let mut rounded_speed = (speed / 10.0 * 2.0).round() / 2.0 * 10.0;
        let mut num_pennants = 0;
        while rounded_speed >= 50.0 {
            num_pennants += 1;
            rounded_speed -= 50.0;
        }

        let mut num_barbs = 0;
        while rounded_speed >= 10.0 {
            num_barbs += 1;
            rounded_speed -= 10.0;
        }

        let mut pennant_coords = [(
            ScreenCoords::origin(),
            ScreenCoords::origin(),
            ScreenCoords::origin(),
        ); 5];

        for i in 0..num_pennants {
            if i >= pennant_coords.len() {
                break;
            }

            let mut pos = shaft_end;
            pos.x -= (i + 1) as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= (i + 1) as f64 * barb_config.pennant_width * direction_radians.sin();
            let pnt1 = pos;

            pos.x += barb_config.pennant_width * direction_radians.cos();
            pos.y += barb_config.pennant_width * direction_radians.sin();
            let pnt2 = pos;

            let point_angle = direction_radians - ::std::f64::consts::FRAC_PI_2;
            pos.x += barb_config.barb_length * point_angle.cos();
            pos.y += barb_config.barb_length * point_angle.sin();
            let pnt3 = pos;

            pennant_coords[i] = (pnt1, pnt2, pnt3);
        }

        let mut barb_coords = [(ScreenCoords::origin(), ScreenCoords::origin()); 5];

        for i in 0..num_barbs {
            if i >= barb_coords.len() {
                break;
            }

            let mut pos = shaft_end;
            pos.x -= num_pennants as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= num_pennants as f64 * barb_config.pennant_width * direction_radians.sin();

            pos.x -= i as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= i as f64 * barb_config.pennant_width * direction_radians.sin();
            let pnt1 = pos;

            let point_angle = direction_radians - ::std::f64::consts::FRAC_PI_2;
            pos.x += barb_config.barb_length * point_angle.cos();
            pos.y += barb_config.barb_length * point_angle.sin();
            let pnt2 = pos;

            barb_coords[i] = (pnt1, pnt2);
        }

        // Add half barb if needed
        if rounded_speed >= 5.0 && num_barbs < barb_coords.len() {

            let mut pos = shaft_end;
            pos.x -= num_pennants as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= num_pennants as f64 * barb_config.pennant_width * direction_radians.sin();

            pos.x -= num_barbs as f64 * barb_config.pennant_width * direction_radians.cos();
            pos.y -= num_barbs as f64 * barb_config.pennant_width * direction_radians.sin();
            let pnt1 = pos;

            let point_angle = direction_radians - ::std::f64::consts::FRAC_PI_2;
            pos.x += barb_config.barb_length * point_angle.cos() / 2.0;
            pos.y += barb_config.barb_length * point_angle.sin() / 2.0;
            let pnt2 = pos;

            barb_coords[num_barbs] = (pnt1, pnt2);

            num_barbs += 1;
        }

        let point_radius = barb_config.dot_size;

        WindBarbData {
            center,
            shaft_end,
            num_pennants,
            pennant_coords,
            num_barbs,
            barb_coords,
            direction_radians,
            point_radius,
        }
    }

    fn bounding_box(&self) -> ScreenRect {
        let mut bbox = ScreenRect {
            lower_left: ScreenCoords {
                x: self.center.x - self.point_radius,
                y: self.center.y - self.point_radius,
            },
            upper_right: ScreenCoords {
                x: self.center.x + self.point_radius,
                y: self.center.y + self.point_radius,
            },
        };

        bbox.expand_to_fit(self.shaft_end);
        for i in 0..self.num_pennants {
            if i >= self.pennant_coords.len() {
                break;
            }
            bbox.expand_to_fit(self.pennant_coords[i].2);
        }
        for i in 0..self.num_barbs {
            if i >= self.barb_coords.len() {
                break;
            }
            bbox.expand_to_fit(self.barb_coords[i].1);
        }

        bbox
    }

    fn draw(&self, cr: &Context) {
        // Assume color and line width are already taken care of.
        cr.arc(
            self.center.x,
            self.center.y,
            self.point_radius,
            0.0,
            2.0 * ::std::f64::consts::PI,
        );
        cr.fill();

        cr.move_to(self.center.x, self.center.y);
        cr.line_to(self.shaft_end.x, self.shaft_end.y);
        cr.stroke();

        for (i, &(pnt1, pnt2, pnt3)) in self.pennant_coords.iter().enumerate() {
            if i >= self.num_pennants {
                break;
            }
            cr.move_to(pnt1.x, pnt1.y);
            cr.line_to(pnt2.x, pnt2.y);
            cr.line_to(pnt3.x, pnt3.y);
            cr.close_path();
            cr.fill();
        }

        for (i, &(pnt1, pnt2)) in self.barb_coords.iter().enumerate() {
            if i >= self.num_barbs {
                break;
            }
            cr.move_to(pnt1.x, pnt1.y);
            cr.line_to(pnt2.x, pnt2.y);
            cr.stroke();
        }
    }
}

fn get_wind_barb_center(pressure: f64, xcenter: f64, ac: &AppContext) -> ScreenCoords {

    let ScreenCoords { x: _, y: yc } = ac.convert_tp_to_screen(TPCoords {
        temperature: 0.0,
        pressure,
    });

    ScreenCoords { x: xcenter, y: yc }
}
