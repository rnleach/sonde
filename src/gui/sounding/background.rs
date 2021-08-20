use super::SkewTContext;
use crate::{
    app::config::{self},
    coords::TPCoords,
    gui::DrawingArgs,
};
use metfor::{Celsius, CelsiusDiff, HectoPascal, Quantity};

impl SkewTContext {
    pub fn draw_clear_background(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.background_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        const MINT: Celsius = Celsius(-160.0);
        const MAXT: Celsius = Celsius(100.0);
        self.draw_temperature_band(MINT, MAXT, args);
    }

    pub fn draw_temperature_banding(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.background_band_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        let mut start_line = -160i32;
        while start_line < 100 {
            let t1 = Celsius(f64::from(start_line));
            let t2 = t1 + CelsiusDiff(10.0);

            self.draw_temperature_band(t1, t2, args);

            start_line += 20;
        }
    }

    pub fn draw_hail_growth_zone(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.hail_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
        self.draw_temperature_band(Celsius(-30.0), Celsius(-10.0), args);
    }

    pub fn draw_dendtritic_growth_zone(&self, args: DrawingArgs<'_, '_>) {
        let (cr, config) = (args.cr, args.ac.config.borrow());

        let rgba = config.dendritic_zone_rgba;
        cr.set_source_rgba(rgba.0, rgba.1, rgba.2, rgba.3);

        self.draw_temperature_band(Celsius(-18.0), Celsius(-12.0), args);
    }

    fn draw_temperature_band(&self, cold_t: Celsius, warm_t: Celsius, args: DrawingArgs<'_, '_>) {
        let cr = args.cr;

        // Assume color has already been set up for us.

        const MAXP: HectoPascal = config::MAXP;
        const MINP: HectoPascal = config::MINP;

        let mut coords = [
            (warm_t.unpack(), MAXP.unpack()),
            (warm_t.unpack(), MINP.unpack()),
            (cold_t.unpack(), MINP.unpack()),
            (cold_t.unpack(), MAXP.unpack()),
        ];

        // Convert points to screen coords
        for coord in &mut coords {
            let screen_coords = self.convert_tp_to_screen(TPCoords {
                temperature: Celsius(coord.0),
                pressure: HectoPascal(coord.1),
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
