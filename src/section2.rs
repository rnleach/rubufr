use crate::{read_1_octet_u8, types::BufrMessageBuilder};

use super::read_3_octet_usize;
use std::{error::Error, io::Read};

pub(super) fn read_section_2(
    mut f: impl Read,
    builder: &mut BufrMessageBuilder,
) -> Result<(), Box<dyn Error>> {

    let section_size = read_3_octet_usize(&mut f)?
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Section Size Required"))?;

    let _reserved:() = read_1_octet_u8(&mut f)?
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Reserved Octet Required"))
        .and_then(|val| {
            match val {
                0 => Ok(()),
                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Reserved Octet Required To Be 0")),
            }
        })?;

    let mut section_data = vec![];
    f.take(section_size as u64 - 4).read_to_end(&mut section_data)?;

    builder.section_2_data(section_data);

    Ok(())
}
