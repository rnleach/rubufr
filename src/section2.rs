use crate::{read_1_octet_u8, types::BufrMessageBuilder};

use super::read_3_octet_usize;
use std::{error::Error, io::Read};

pub(super) fn read_section_2(
    mut f: impl Read,
    builder: &mut BufrMessageBuilder,
) -> Result<(), Box<dyn Error>> {
    // TODO Return error if unable to read section size.
    let section_size = read_3_octet_usize(&mut f)?.unwrap();

    // TODO: Check if this is zero and return an error if it isn't
    // TODO: If this shows up as missing, return an error.
    let reserved = read_1_octet_u8(&mut f)?.unwrap();
    debug_assert_eq!(reserved, 0);

    let mut section_data = vec![];
    f.take(section_size as u64 - 4)
        .read_to_end(&mut section_data)?;

    builder.section_2_data(section_data);

    Ok(())
}
