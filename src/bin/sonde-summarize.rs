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

    loop {
        let header = match scan_to_bufr_start(&mut f) {
            Ok(h) => h,
            Err(e) => {
                println!("End of file or error: {}", e);
                break;
            }
        };

        match String::from_utf8(header) {
            Ok(h) => println!("Header:\n\t{}", h),
            Err(_) => println!("Header content is not a UTF-8 string."),
        }

        let bufr = read_bufr_message(&mut f)?;

        println!("BUFR Summary:\n{}", &bufr);
    }

    Ok(())
}
