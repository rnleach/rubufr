use crate::read_1_octet_u8;

use super::read_3_octet_usize;
use std::{error::Error, fmt::Display, io::Read};

pub struct Section2 {
    section_size: usize,
    section_data: Option<Vec<u8>>,
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
    section_2_present: bool,
) -> Result<Section2, Box<dyn Error>> {
    if section_2_present {
        let section_size = read_3_octet_usize(&mut f)?;

        // TODO: Check if this is zero and return an error if it isn't
        let reserved = read_1_octet_u8(&mut f)?;
        debug_assert_eq!(reserved, 0);

        let mut section_data = vec![];
        f.take(section_size as u64 - 4)
            .read_to_end(&mut section_data)?;
        let section_data = Some(section_data);

        Ok(Section2 {
            section_size,
            section_data,
        })
    } else {
        let section_size = 0;
        let section_data = None;

        Ok(Section2 {
            section_size,
            section_data,
        })
    }
}
