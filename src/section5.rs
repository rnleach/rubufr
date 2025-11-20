use std::{error::Error, io::Read};

pub struct Section5 {}

pub(super) fn read_section_5(mut f: impl Read) -> Result<Section5, Box<dyn Error>> {
    let mut section_end: [u8; 4] = [0; 4];
    f.read_exact(&mut section_end)?;
    let section_end = std::str::from_utf8(&section_end)?;

    // TODO: Check this value and return an error if it fails.
    assert_eq!(section_end, "7777");

    Ok(Section5 {})
}
