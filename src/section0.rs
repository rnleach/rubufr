use super::{read_1_octet_u8, read_3_octet_usize};
use std::{error::Error, fmt::Display, io::Read};

pub struct Section0 {
    message_size: usize,
    bufr_version: u8,
}

impl Display for Section0 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "  BUFR version: {}", self.bufr_version)?;
        writeln!(f, "Message Length: {}", self.message_size)?;

        Ok(())
    }
}

pub(super) fn read_section_0(mut f: impl Read) -> Result<Section0, Box<dyn Error>> {
    let mut bufr_name: [u8; 4] = [0; 4];
    f.read_exact(&mut bufr_name)?;
    let bufr_name = std::str::from_utf8(&bufr_name)?;
    assert_eq!(bufr_name, "BUFR");

    let message_size = read_3_octet_usize(&mut f)?;
    let bufr_version = read_1_octet_u8(&mut f)?;

    // TODO: create error and return it if BUFR version not 4

    Ok(Section0 {
        message_size,
        bufr_version,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_section_0() {
        const TEST_SIZE: usize = 42;
        const TEST_VERSION: u8 = 4;

        let mut message: Vec<u8> = "BUFR".as_bytes().to_vec();
        message.extend_from_slice(&TEST_SIZE.to_be_bytes()[5..8]);
        message.push(TEST_VERSION);

        let mut message = Cursor::new(message);

        let sec0 = read_section_0(&mut message).unwrap();
        assert_eq!(sec0.message_size, TEST_SIZE);
        assert_eq!(sec0.bufr_version, TEST_VERSION);
    }
}
