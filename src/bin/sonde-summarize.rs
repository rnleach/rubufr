use sonde_bufr::{read_bufr_message, scan_to_bufr_start};
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("No file name provided!");
        return Ok(());
    }

    let f = std::fs::File::open(&args[0])?;
    let mut f = std::io::BufReader::new(f);
    scan_to_bufr_start(&mut f)?;
    let bufr = read_bufr_message(&mut f)?;

    println!("{}", &bufr);

    Ok(())
}
