use std::{env, error::Error, path::Path};

use sonde_bufr::load_309052_sounding;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("No file name provided!");
        return Ok(());
    }

    let snd = load_309052_sounding(&Path::new(&args[0]))?;

    println!();
    println!("---------- Sounding ---------- ");
    println!("     Source:  {:?}", snd.source_description());
    println!(" Valid Time:  {:?}", snd.valid_time());
    println!("  Lead Time:  {:?}", snd.lead_time());
    println!("StationInfo:  {:?}", snd.station_info());
    println!();
    println!("               mslp:  {:?}", snd.mslp());
    println!("   station pressure:  {:?}", snd.station_pressure());
    println!("surface temperature:  {:?}", snd.sfc_temperature());
    println!("  surface dew point:  {:?}", snd.sfc_dew_point());
    println!("          low cloud:  {:?}", snd.low_cloud());
    println!("          mid cloud:  {:?}", snd.mid_cloud());
    println!("         high cloud:  {:?}", snd.high_cloud());
    println!("      precipitation:  {:?}", snd.precipitation());
    println!("       surface wind:  {:?}", snd.sfc_wind());
    println!();
    for row in snd.top_down().take(10) {
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            row.pressure,
            row.temperature,
            row.wet_bulb,
            row.dew_point,
            row.theta_e,
            row.wind,
            row.pvv,
            row.height,
            row.cloud_fraction
        );
    }
    println!("          :");
    println!("          :");
    println!("          :");

    for row in snd
        .top_down()
        .enumerate()
        .skip_while(|(i, _)| *i < snd.pressure_profile().len() - 11)
        .map(|(_, x)| x)
    {
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            row.pressure,
            row.temperature,
            row.wet_bulb,
            row.dew_point,
            row.theta_e,
            row.wind,
            row.pvv,
            row.height,
            row.cloud_fraction
        );
    }
    println!("------------------------------ ");
    println!();
    println!();

    Ok(())
}

