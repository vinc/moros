use crate::api::clock;
use crate::api::process;

use alloc::format;
use core::f64::consts::PI;

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() < 2 {
        eprintln!("Usage: <longitude> [<timestamp>]");
        return Err(process::EXIT_FAILURE);
    }

    let longitude = args[1].parse().expect("Could not parse longitude");

    let timestamp = if args.len() == 3 {
        args[2].parse().expect("Could not parse timestamp")
    } else {
        clock::realtime()
    };

    let t = geotime(longitude, timestamp);
    let t = libm::floor(100.0 * t) / 100.0; // Avoid rounding up 99.996 to 100.00
    println!("{}", format!("{:05.2}", t).replace(".", ":"));

    Ok(())
}

pub fn geotime(longitude: f64, timestamp: f64) -> f64 {
    let days = libm::floor(libm::fmod(timestamp / 86400.0, 365.2425));
    let hours = libm::floor(libm::fmod(timestamp, 86400.0) / 3600.0);

    // Equation of time (https://www.esrl.noaa.gov/gmd/grad/solcalc/solareqns.PDF)
    let y = (2.0 * PI / 365.0) * (days + (hours - 12.0) / 24.0);
    let eot = 60.0 * 229.18 * (
        0.000075 +
        0.001868 * libm::cos(1.0 * y) -
        0.032077 * libm::sin(1.0 * y) -
        0.014615 * libm::cos(2.0 * y) -
        0.040849 * libm::sin(2.0 * y)
    );

    let seconds = libm::fmod(timestamp, 86400.0) + (longitude * 86400.0 / 360.0) + eot;
    
    libm::fmod(100.0 * seconds / 86400.0, 100.0)
}
