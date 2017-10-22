#![allow(dead_code)] // for now

/// Potential temperature in Kelvin
pub fn theta_kelvin(pressure_hpa: f32, temperature_c: f32) -> f32 {
    use std::f32;

    (temperature_c + 273.15) * f32::powf(1000.0 / pressure_hpa, 0.286)
}

/// Temperture in C
pub fn temperature_c_from_theta(theta_kelvin: f32, pressure_hpa: f32) -> f32 {
    use std::f32;

    theta_kelvin * f32::powf(pressure_hpa / 1000.0, 0.286) - 273.15
}

/// Get the vapor pressure of water as a function of temperature in hPa
pub fn vapor_pressure_water(temperature_c: f32) -> f32 {
    use std::f32;

    6.11 * f32::powf(10.0, 7.5 * temperature_c / (237.3 + temperature_c))
}

/// Get the mixing ratio in g/kg.
pub fn mixing_ratio(temperature_c: f32, pressure_hpa: f32) -> f32 {
    let vp = vapor_pressure_water(temperature_c);
    621.97 * (vp / (pressure_hpa - vp))
}

/// Given a mixing ratio and pressure, calculate the temperature. The p is in hPa and the mw is in
/// g/kg. Assume 100% rh.
pub fn temperature_from_p_and_saturated_mw(p: f32, mw: f32) -> f32 {
    use std::f32;

    let z = mw * p / 6.11 / 621.97 / (1.0 + mw / 621.97);
    237.5 * f32::log10(z) / (7.5 - f32::log10(z))
}

pub fn theta_e_saturated_kelvin(pressure_hpa: f32, temperature_c: f32) -> f32 {
    use std::f32;

    let theta = theta_kelvin(pressure_hpa, temperature_c);
    let mw = mixing_ratio(temperature_c, pressure_hpa) / 1000.0; // divide by 1000 to get kg/kg

    theta * f32::exp(2.6897e6 * mw / 1005.7 / (temperature_c + 273.15))
}
