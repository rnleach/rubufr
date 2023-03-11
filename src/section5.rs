use std::{error::Error, fmt::Display, io::Read};

pub struct Section5 {}

impl Display for Section5 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "END OF BUFR MESSAGE")
    }
}

pub(super) fn read_section_5(mut f: impl Read) -> Result<Section5, Box<dyn Error>> {
    let mut section_end: [u8; 4] = [0; 4];
    f.read_exact(&mut section_end)?;
    let section_end = std::str::from_utf8(&section_end)?;
    assert_eq!(section_end, "7777");

    Ok(Section5 {})
}
