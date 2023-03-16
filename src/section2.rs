use crate::{read_1_octet_u8, types::BufrMessageBuilder};

use super::read_3_octet_usize;
use std::{error::Error, fmt::Display, io::Read};

pub struct Section2 {
    section_size: usize,
    pub(crate) section_data: Option<Vec<u8>>,
}

impl Display for Section2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, " Section Size: {}", self.section_size)?;
        writeln!(
            f,
            "Bytes of Data: {}",
            self.section_data.as_ref().map(|v| v.len()).unwrap_or(0)
        )?;

        Ok(())
    }
}

pub(super) fn read_section_2(
    mut f: impl Read,
    builder: &mut BufrMessageBuilder,
) -> Result<(), Box<dyn Error>> {
    let section_size = read_3_octet_usize(&mut f)?;

    // TODO: Check if this is zero and return an error if it isn't
    let reserved = read_1_octet_u8(&mut f)?;
    debug_assert_eq!(reserved, 0);

    let mut section_data = vec![];
    f.take(section_size as u64 - 4)
        .read_to_end(&mut section_data)?;

    builder.section_2_data(section_data);

    Ok(())
}
