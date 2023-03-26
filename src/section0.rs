use crate::{read_1_octet_u8, read_3_octet_usize, types::BufrMessageBuilder};
use std::{error::Error, io::Read};

pub(super) fn read_section_0(
    mut f: impl Read,
    builder: &mut BufrMessageBuilder,
) -> Result<(), Box<dyn Error>> {
    let mut bufr_name: [u8; 4] = [0; 4];
    f.read_exact(&mut bufr_name)?;
    let bufr_name = std::str::from_utf8(&bufr_name)?;
    assert_eq!(bufr_name, "BUFR");

    let _message_size = read_3_octet_usize(&mut f)?;
    let bufr_version = read_1_octet_u8(&mut f)?;

    builder.bufr_version(bufr_version);

    Ok(())
}
