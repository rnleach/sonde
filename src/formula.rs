#![allow(dead_code)] // for now

/// Potential temperature in Kelvin
pub fn theta_kelvin(pressure_hpa: f64, temperature_c: f64) -> f64 {
    use std::f64;

    (temperature_c + 273.15) * f64::powf(1000.0 / pressure_hpa, 0.286)
}

/// Temperture in C
pub fn temperature_c_from_theta(theta_kelvin: f64, pressure_hpa: f64) -> f64 {
    use std::f64;

    theta_kelvin * f64::powf(pressure_hpa / 1000.0, 0.286) - 273.15
}

/// Get the vapor pressure of water as a function of temperature in hPa
pub fn vapor_pressure_water(temperature_c: f64) -> f64 {
    use std::f64;

    6.11 * f64::powf(10.0, 7.5 * temperature_c / (237.3 + temperature_c))
}

/// Get the mixing ratio in g/kg.
pub fn mixing_ratio(temperature_c: f64, pressure_hpa: f64) -> f64 {
    let vp = vapor_pressure_water(temperature_c);
    621.97 * (vp / (pressure_hpa - vp))
}

/// Given a mixing ratio and pressure, calculate the temperature. The p is in hPa and the mw is in
/// g/kg. Assume 100% rh.
pub fn temperature_from_p_and_saturated_mw(p: f64, mw: f64) -> f64 {
    use std::f64;

    let z = mw * p / 6.11 / 621.97 / (1.0 + mw / 621.97);
    237.5 * f64::log10(z) / (7.5 - f64::log10(z))
}

pub fn theta_e_saturated_kelvin(pressure_hpa: f64, temperature_c: f64) -> f64 {
    use std::f64;

    let theta = theta_kelvin(pressure_hpa, temperature_c);
    let mw = mixing_ratio(temperature_c, pressure_hpa) / 1000.0; // divide by 1000 to get kg/kg

    theta * f64::exp(2.6897e6 * mw / 1005.7 / (temperature_c + 273.15))
}

pub fn celsius_to_f(temperature: f64) -> f64 {
    1.8 * temperature + 32.0
}

/// Bisection algorithm for finding the root of an equation given values bracketing a root. Used
/// when drawing moist adiabats.
pub fn find_root(f: &Fn(f64) -> f64, mut low_val: f64, mut high_val: f64) -> f64 {
    use std::f64;
    const MAX_IT: usize = 50;
    const EPS: f64 = 1.0e-3;

    if low_val > high_val {
        ::std::mem::swap(&mut low_val, &mut high_val);
    }

    let mut f_low = f(low_val);
    // let mut f_high = f(high_val);

    let mut mid_val = (high_val - low_val) / 2.0 + low_val;
    let mut f_mid = f(mid_val);
    for _ in 0..MAX_IT {
        if f_mid * f_low > 0.0 {
            low_val = mid_val;
            f_low = f_mid;
        } else {
            high_val = mid_val;
            // f_high = f_mid;
        }

        if (high_val - low_val).abs() < EPS {
            break;
        }

        mid_val = (high_val - low_val) / 2.0 + low_val;
        f_mid = f(mid_val);
    }

    mid_val
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;

    fn approx_equal(val1: f64, val2: f64) -> bool {
        const EPS: f64 = 1.0e-3;

        (val1 - val2).abs() < EPS
    }

    #[test]
    fn test_theta_kelvin() {
        assert!(approx_equal((32.0 + 273.15), theta_kelvin(1000.0, 32.0)));
    }

    #[test]
    fn test_find_min() {
        assert!(approx_equal(1.0, find_root(&|x| x * x - 1.0, 2.0, 0.0)));
        assert!(approx_equal(-1.0, find_root(&|x| x * x - 1.0, -2.0, 0.0)));
    }
}
