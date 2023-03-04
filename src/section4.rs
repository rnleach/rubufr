use super::read_3_octet_usize;
use std::{error::Error, fmt::Display, io::Read};

pub struct Section4 {}

impl Display for Section4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

pub(super) fn read_section_4(mut f: impl Read) -> Result<Section4, Box<dyn Error>> {
    //let section_size = read_3_octet_usize(&mut f)?;
    Ok(Section4 {})
}
