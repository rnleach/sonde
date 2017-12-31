use gui::DrawingArgs;

mod background;
mod labeling;
mod sample_readout;
mod temperature_profile;
mod wind_profile;

pub fn draw_background(args: DrawingArgs) {
    
    background::draw_background_fill(args);
    background::draw_background_lines(args);
    labeling::draw_background_labels(args);
}

pub fn draw_data(args: DrawingArgs) {
    draw_temperature_profiles(args);
    draw_wind_profile(args);
}

fn draw_temperature_profiles(args: DrawingArgs) {
    let config = args.ac.config.borrow();

    use self::temperature_profile::TemperatureType::{DewPoint, DryBulb, WetBulb};

    if config.show_wet_bulb {
        temperature_profile::draw_temperature_profile(WetBulb, args);
    }

    if config.show_dew_point {
        temperature_profile::draw_temperature_profile(DewPoint, args);
    }

    if config.show_temperature {
        temperature_profile::draw_temperature_profile(DryBulb, args);
    }
}

fn draw_wind_profile(args: DrawingArgs) {
    if args.ac.config.borrow().show_wind_profile {
        wind_profile::draw_wind_profile(args);
    }
}

pub fn draw_overlays(args: DrawingArgs) {
    if args.ac.config.borrow().show_active_readout {
        sample_readout::draw_active_sample(args);
    }
}
