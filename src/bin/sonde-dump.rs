use std::{env, error::Error, iter::zip};

use chrono::NaiveDate;
use optional::{none, Optioned};

use sonde_bufr::{read_bufr_message, scan_to_bufr_start, Structure, Group, Replication};

use metfor::{Kelvin, Celsius, HectoPascal, Meters, MetersPSec, Knots, WindSpdDir};
use sounding_analysis::{Sounding, StationInfo};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("No file name provided!");
        return Ok(());
    }

    let f = std::fs::File::open(&args[0])?;
    let mut f = std::io::BufReader::new(f);

    loop {

        if let Err(e) = scan_to_bufr_start(&mut f) {
            println!("End of file or error: {}", e);
            break;
        }

        let bufr = read_bufr_message(&mut f)?;

        let mut station = StationInfo::new();
        let mut snd = Sounding::new();

        let mut pres: Vec<Optioned<HectoPascal>> = vec![];
        let mut temp: Vec<Optioned<Celsius>> = vec![];
        let mut dewp: Vec<Optioned<Celsius>> = vec![];
        let mut hgt: Vec<Optioned<Meters>> = vec![];
        let mut dir: Vec<Optioned<f64>> = vec![];
        let mut spd: Vec<Optioned<Knots>> = vec![];

        for structure in bufr.get_elements() {

            // Find the sounding format I'm looking for.
            if let Structure::Group(grp) = structure && grp.code() == "309052" {
                
                // Iterate the members of the Group
                for structure in grp.items() {

                    // A nested group with location & station info
                    if let Structure::Group(grp) = structure {
                        match grp.code() {

                            "301111" => station = extract_station_id_from_group(grp, station),
                            "301113" => snd = extract_time_info_from_group(grp, snd),
                            "301114" => station = extract_station_location_from_group(grp, station),
                            _ => {},
                        }
                    } else if let Structure::Replication(rep) = structure {
                        if rep.len() > pres.capacity() {
                            pres.reserve(rep.len());
                            temp.reserve(rep.len());
                            dewp.reserve(rep.len());
                            hgt.reserve(rep.len());
                            dir.reserve(rep.len());
                            spd.reserve(rep.len());
                        }

                        extract_data_rows(rep, &mut pres, &mut temp, &mut dewp, &mut hgt, &mut dir, &mut spd);
                    }
                }
            }
        }

        let wnd: Vec<Optioned<WindSpdDir<Knots>>> = zip(dir, spd)
            .map(|(d, s): (Optioned<f64>, Optioned<Knots>)| {
                s.and_then(|ss| d.map(|dd|  WindSpdDir { speed: ss, direction: dd }).into())
            })
            .collect();

        snd = snd.with_station_info(station);
        snd = snd.with_pressure_profile(pres);
        snd = snd.with_temperature_profile(temp);
        snd = snd.with_dew_point_profile(dewp);
        snd = snd.with_height_profile(hgt);
        snd = snd.with_wind_profile(wnd);


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
            println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                row.pressure, row.temperature, row.wet_bulb, row.dew_point, row.theta_e, row.wind, row.pvv,
                row.height, row.cloud_fraction);
        }
        println!("          :");
        println!("          :");
        println!("          :");

        for row in snd.top_down().enumerate().skip_while(|(i, _)| *i < snd.pressure_profile().len() - 11).map(|(_, x)| x) {
            println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                row.pressure, row.temperature, row.wet_bulb, row.dew_point, row.theta_e, row.wind, row.pvv,
                row.height, row.cloud_fraction);
        }
        println!("------------------------------ ");
        println!();
        println!();

    }

    Ok(())
}

fn extract_data_rows(
    rep: &Replication,
    pres: &mut Vec<Optioned<HectoPascal>>,
    temp: &mut Vec<Optioned<Celsius>>,
    dewp: &mut Vec<Optioned<Celsius>>,
    hgt: &mut Vec<Optioned<Meters>>,
    dir: &mut Vec<Optioned<f64>>,
    spd: &mut Vec<Optioned<Knots>>,
) {

    for structure in rep.items() {
        if let Structure::Group(grp) = structure && grp.code() == "303054" {
            for structure in grp.items() {
                if let Structure::Element(el) = structure {
                    match el.code() {
                        "007004" => pres.push(el.get_f64_val().map(|x| x / 100.0).map(HectoPascal).into()),
                        "010009" => hgt.push(el.get_f64_val().map(Meters).into()),
                        "011001" => dir.push(el.get_f64_val().into()),
                        "011002" => spd.push(el.get_f64_val().map(MetersPSec).map(Knots::from).into()),
                        "012101" => temp.push(el.get_f64_val().map(Kelvin).map(Celsius::from).into()),
                        "012103" => dewp.push(el.get_f64_val().map(Kelvin).map(Celsius::from).into()),
                        _ => {},
                    }
                }
            }
        }
    }
}

fn extract_station_id_from_group(grp: &Group, mut station: StationInfo) -> StationInfo {
    debug_assert_eq!(grp.code(), "301111");

    for structure in grp.items() {
        if let Structure::Element(el) = structure && el.code() == "001011" {
            station = station.with_station_id(el.get_str_val().map(String::from));
            break;
        }
    }

    station
}

fn extract_station_location_from_group(grp: &Group, mut station: StationInfo) -> StationInfo {
    debug_assert_eq!(grp.code(), "301114");

    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut elev: Optioned<Meters> = none();

    for structure in grp.items() {
        match structure {
            Structure::Element(el) if el.code() == "007030" || el.code() == "007007" => {
                if elev.is_none() && el.get_f64_val().is_some() {
                    elev = el.get_f64_val().map(Meters).into();
                }
            },

            Structure::Group(grp) if grp.code() == "301021" => {
                for structure in grp.items() {
                    match structure {
                        Structure::Element(el) if el.code() == "005001" => lat = el.get_f64_val(),
                        Structure::Element(el) if el.code() == "006001" => lon = el.get_f64_val(),
                        _ => {},
                    }
                }
            },

            _ => {},
        }
    }

    station = station.with_lat_lon(lat.zip(lon));
    station = station.with_elevation(elev);
    station
}

fn extract_time_info_from_group(grp: &Group, mut snd: Sounding) -> Sounding {
    debug_assert_eq!(grp.code(), "301113");

    let mut year: i32 = 0;
    let mut month: u32 = 0;
    let mut day: u32 = 0;
    let mut hour: u32 = 0;
    let mut minute: u32 = 0;
    let mut second: u32 = 0;

    for structure in grp.items() {
        if let Structure::Group(grp) = structure && grp.code() == "301011" {
            for structure in grp.items() {
                match structure {
                    Structure::Element(el) if el.code() == "004001" => year = el.get_i32_val().unwrap_or(0),
                    Structure::Element(el) if el.code() == "004002" => month = el.get_u32_val().unwrap_or(0),
                    Structure::Element(el) if el.code() == "004003" => day = el.get_u32_val().unwrap_or(0),
                    _ => {},
                }
            }
        } else if let Structure::Group(grp) = structure && grp.code() == "301013" {
            for structure in grp.items() {
                match structure {
                    Structure::Element(el) if el.code() == "004004" => hour = el.get_u32_val().unwrap_or(0),
                    Structure::Element(el) if el.code() == "004005" => minute = el.get_u32_val().unwrap_or(0),
                    Structure::Element(el) if el.code() == "004006" => second = el.get_u32_val().unwrap_or(0),
                    _ => {},
                }
            }
        }
    }

    if let Some(vt) = NaiveDate::from_ymd_opt(year, month, day).and_then(|d| d.and_hms_opt(hour, minute, second)) {
        snd = snd.with_valid_time(vt);
    }

    snd
}

